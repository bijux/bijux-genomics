#!/usr/bin/env python3
from __future__ import annotations

import csv
import hashlib
import json
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:
    tomllib = None


LOCAL_RESULTS_ROOT = Path("/Users/bijan/bijux/bijux-dna-results")


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def _prefix_bundle_members(path: Path) -> list[Path]:
    if path.exists():
        return [path]
    parent = path.parent
    if not parent.is_dir():
        return []
    return sorted(
        candidate
        for candidate in parent.glob(f"{path.name}*")
        if candidate.is_file() or candidate.is_dir()
    )


def artifact_bundle_exists(path: Path) -> bool:
    return bool(_prefix_bundle_members(path))


def artifact_bundle_size_bytes(path: Path) -> int:
    total = 0
    for member in _prefix_bundle_members(path):
        if member.is_file():
            total += member.stat().st_size
            continue
        for nested in sorted(candidate for candidate in member.rglob("*") if candidate.is_file()):
            total += nested.stat().st_size
    return total


def sha256_artifact_bundle(path: Path) -> str:
    members = _prefix_bundle_members(path)
    if not members:
        raise FileNotFoundError(f"missing artifact bundle: {path}")

    digest = hashlib.sha256()
    for member in members:
        if member.is_file():
            digest.update(member.name.encode("utf-8"))
            digest.update(b"\0file\0")
            digest.update(sha256_file(member).encode("utf-8"))
            continue

        for nested in sorted(member.rglob("*")):
            if nested == member:
                continue
            relative = nested.relative_to(member.parent)
            digest.update(relative.as_posix().encode("utf-8"))
            if nested.is_dir():
                digest.update(b"\0dir\0")
                continue
            digest.update(b"\0file\0")
            digest.update(sha256_file(nested).encode("utf-8"))
    return digest.hexdigest()


@dataclass(frozen=True)
class CorpusBenchmarkContract:
    stage_id: str
    scenario_id: str
    tools: list[str]
    sample_scope: str = "full"


@dataclass(frozen=True)
class CorpusBenchmarkExclusion:
    stage_id: str
    reason: str


@dataclass(frozen=True)
class ReportQcContributorContract:
    stage_id: str
    tool_id: str
    artifact_id: str
    artifact_role: str
    relative_path: str

    @property
    def contributor_id(self) -> str:
        return f"{self.stage_id}.{self.tool_id}"

    @property
    def artifact_name(self) -> str:
        return f"{self.stage_id}.tool.{self.tool_id}.{self.artifact_id}"


DETECT_ADAPTERS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.detect_adapters",
    scenario_id="detect_adapters_fairness",
    tools=["fastqc"],
)


PROFILE_READS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.profile_reads",
    scenario_id="profile_reads_fairness",
    tools=["seqkit_stats"],
)


PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.profile_read_lengths",
    scenario_id="read_length_fairness",
    tools=["seqkit_stats"],
)


PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.profile_overrepresented_sequences",
    scenario_id="overrepresented_sequence_fairness",
    tools=["fastqc", "fastq_scan", "seqkit"],
)


MERGE_PAIRS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.merge_pairs",
    scenario_id="merge_fairness",
    tools=["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch"],
    sample_scope="paired",
)


REPORT_QC_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.report_qc",
    scenario_id="qc_aggregation_fairness",
    tools=["multiqc"],
)


TRIM_POLYG_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.trim_polyg_tails",
    scenario_id="polyg_trim_fairness",
    tools=["bbduk", "fastp"],
)


TRIM_READS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.trim_reads",
    scenario_id="trim_fairness",
    tools=[
        "adapterremoval",
        "atropos",
        "bbduk",
        "cutadapt",
        "fastp",
        "prinseq",
        "seqkit",
        "trim_galore",
        "trimmomatic",
    ],
)


TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.trim_terminal_damage",
    scenario_id="terminal_damage_fairness",
    tools=["adapterremoval", "cutadapt", "seqkit"],
)


NORMALIZE_PRIMERS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.normalize_primers",
    scenario_id="primer_normalization_fairness",
    tools=["cutadapt"],
)


FILTER_READS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.filter_reads",
    scenario_id="filter_fairness",
    tools=["bbduk", "fastp", "prinseq", "seqkit"],
)


DEPLETE_RRNA_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.deplete_rrna",
    scenario_id="rrna_depletion_fairness",
    tools=["sortmerna"],
)


REMOVE_DUPLICATES_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.remove_duplicates",
    scenario_id="dedup_fairness",
    tools=["clumpify", "fastuniq"],
    sample_scope="paired",
)


FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.filter_low_complexity",
    scenario_id="low_complexity_fairness",
    tools=["bbduk", "prinseq"],
)


DEPLETE_HOST_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.deplete_host",
    scenario_id="host_depletion_fairness",
    tools=["bowtie2"],
)


DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.deplete_reference_contaminants",
    scenario_id="contaminant_depletion_fairness",
    tools=["bowtie2"],
)


CORRECT_ERRORS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.correct_errors",
    scenario_id="correction_fairness",
    tools=["bayeshammer", "lighter", "musket", "rcorrector"],
    sample_scope="paired",
)


EXTRACT_UMIS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.extract_umis",
    scenario_id="umi_extraction_fairness",
    tools=["umi_tools"],
    sample_scope="paired",
)


SCREEN_TAXONOMY_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.screen_taxonomy",
    scenario_id="screen_fairness",
    tools=["centrifuge", "kaiju", "kraken2", "krakenuniq"],
)


CORPUS_01_PUBLICATION_CONTRACTS = [
    CorpusBenchmarkContract(
        stage_id="fastq.validate_reads",
        scenario_id="validation_fairness",
        tools=["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"],
    ),
    DETECT_ADAPTERS_BENCHMARK_CONTRACT,
    PROFILE_READS_BENCHMARK_CONTRACT,
    PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT,
    PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT,
    NORMALIZE_PRIMERS_BENCHMARK_CONTRACT,
    TRIM_POLYG_BENCHMARK_CONTRACT,
    TRIM_READS_BENCHMARK_CONTRACT,
    FILTER_READS_BENCHMARK_CONTRACT,
    FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT,
    DEPLETE_RRNA_BENCHMARK_CONTRACT,
    MERGE_PAIRS_BENCHMARK_CONTRACT,
    REMOVE_DUPLICATES_BENCHMARK_CONTRACT,
    DEPLETE_HOST_BENCHMARK_CONTRACT,
    DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT,
    CORRECT_ERRORS_BENCHMARK_CONTRACT,
    EXTRACT_UMIS_BENCHMARK_CONTRACT,
    SCREEN_TAXONOMY_BENCHMARK_CONTRACT,
    TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT,
    REPORT_QC_BENCHMARK_CONTRACT,
]


CORPUS_01_PUBLICATION_EXCLUSIONS = [
    CorpusBenchmarkExclusion(
        stage_id="fastq.index_reference",
        reason=(
            "Reference-index benchmarking measures bundle build throughput rather than "
            "sample-cohort execution on corpus-01 FASTQ inputs."
        ),
    ),
    CorpusBenchmarkExclusion(
        stage_id="fastq.cluster_otus",
        reason=(
            "OTU clustering is amplicon-specific and does not fit the human whole-genome "
            "FASTQ cohort contract used by corpus-01."
        ),
    ),
    CorpusBenchmarkExclusion(
        stage_id="fastq.normalize_abundance",
        reason=(
            "Abundance normalization benchmarks require derived abundance tables rather than "
            "the raw FASTQ corpus-01 publication surface."
        ),
    ),
]


REPORT_QC_CONTRIBUTOR_CONTRACTS = [
    ReportQcContributorContract(
        stage_id="fastq.validate_reads",
        tool_id="fastqvalidator",
        artifact_id="validation_report",
        artifact_role="report_json",
        relative_path="validation.json",
    ),
    ReportQcContributorContract(
        stage_id="fastq.validate_reads",
        tool_id="fastqvalidator",
        artifact_id="validated_reads_manifest",
        artifact_role="summary_json",
        relative_path="validated_reads_manifest.json",
    ),
    ReportQcContributorContract(
        stage_id="fastq.detect_adapters",
        tool_id="fastqc",
        artifact_id="report_json",
        artifact_role="report_json",
        relative_path="adapter_report.json",
    ),
    ReportQcContributorContract(
        stage_id="fastq.detect_adapters",
        tool_id="fastqc",
        artifact_id="adapter_evidence_dir",
        artifact_role="stage_report",
        relative_path="fastqc",
    ),
    ReportQcContributorContract(
        stage_id="fastq.profile_reads",
        tool_id="seqkit_stats",
        artifact_id="qc_json",
        artifact_role="metrics_json",
        relative_path="qc.json",
    ),
    ReportQcContributorContract(
        stage_id="fastq.profile_read_lengths",
        tool_id="seqkit_stats",
        artifact_id="length_distribution_json",
        artifact_role="metrics_json",
        relative_path="length_distribution.json",
    ),
]


def trim_reads_benchmark_defaults() -> dict:
    return {
        "min_length": 30,
        "quality_cutoff": None,
        "n_policy": "retain",
        "adapter_policy": "none",
        "polyx_policy": "none",
        "contaminant_policy": "none",
        "adapter_bank_preset": None,
        "polyx_preset": None,
        "contaminant_preset": None,
    }


def trim_terminal_damage_benchmark_defaults() -> dict:
    return {
        "damage_mode": "ancient",
        "execution_policy": "explicit_terminal_trim",
        "trim_5p_bases": 2,
        "trim_3p_bases": 2,
    }


def filter_low_complexity_benchmark_defaults() -> dict:
    return {
        "entropy_threshold": 0.55,
        "polyx_threshold": None,
    }


def filter_reads_benchmark_defaults() -> dict:
    return {
        "threads": 8,
        "max_n": 0,
        "max_n_fraction": None,
        "max_n_count": 3,
        "low_complexity_threshold": 20.0,
        "entropy_threshold": 18.0,
        "kmer_ref": None,
        "polyx_policy": "trim",
    }


def remove_duplicates_benchmark_defaults() -> dict:
    return {
        "dedup_mode": "exact",
        "keep_order": True,
    }


def normalize_primers_benchmark_defaults() -> dict:
    return {
        "primer_set_id": "16S_universal_v1",
        "orientation_policy": "normalize_to_forward_primer",
        "max_mismatch_rate": 0.10,
        "min_overlap_bp": 10,
        "strict_5p_anchor": True,
        "allow_iupac_codes": True,
    }


def deplete_rrna_benchmark_defaults() -> dict:
    return {
        "threads": 4,
        "rrna_bundle_id": "sortmerna_v4_3_default_db",
        "min_identity": 0.95,
    }


def deplete_host_benchmark_defaults() -> dict:
    return {
        "threads": 8,
        "reference_catalog_id": "host_reference",
        "reference_index_backend": "bowtie2_build",
        "host_identity_threshold": 0.95,
        "retain_unmapped_only": True,
    }


def deplete_reference_contaminants_benchmark_defaults() -> dict:
    return {
        "threads": 8,
        "reference_catalog_id": "contaminant_reference",
        "reference_index_backend": "bowtie2_build",
        "decoy_mode": "phix_and_spikeins",
    }


def correct_errors_benchmark_defaults() -> dict:
    return {
        "threads": 8,
        "quality_encoding": "phred33",
        "kmer_size": None,
        "genome_size": None,
        "max_memory_gb": None,
        "trusted_kmer_artifact": None,
        "conservative_mode": False,
    }


def extract_umis_benchmark_defaults() -> dict:
    return {
        "threads": 4,
        "umi_pattern": "NNNNNNNN",
        "allow_missing_umi_headers": True,
    }


def screen_taxonomy_benchmark_defaults() -> dict:
    return {
        "threads": 8,
        "database_catalog_id": "taxonomy_reference",
        "database_artifact_id": "taxonomy_db",
        "database_namespace": "read_screening",
        "database_scope": "read_screening",
        "minimum_confidence": None,
        "emit_unclassified": True,
    }


def merge_pairs_benchmark_defaults() -> dict:
    return {
        "merge_overlap": None,
        "min_length": None,
        "unmerged_read_policy": "emit_unmerged_pairs",
    }


def default_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return corpus_root.parent / "results" / corpus_root.name / stage_id / "lunarc"


def default_local_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return LOCAL_RESULTS_ROOT / corpus_root.name / stage_id / "lunarc"


def preferred_report_run_root(corpus_root: Path, stage_id: str) -> Path:
    local_root = default_local_results_stage_root(corpus_root, stage_id)
    remote_root = default_results_stage_root(corpus_root, stage_id)
    if local_root.exists() or not remote_root.exists():
        return local_root
    return remote_root


def localize_results_path(path_str: str, local_results_root: Path) -> Path:
    path = Path(path_str)
    if path.exists():
        return path
    marker = "/results/"
    if marker not in path_str:
        return path
    return local_results_root / path_str.split(marker, 1)[1]


def validate_benchmark_layout(corpus_root: Path, out_root: Path) -> None:
    try:
        out_root.relative_to(corpus_root)
    except ValueError:
        pass
    else:
        raise SystemExit(
            "benchmark output root must not live under the corpus data tree: "
            f"{out_root}. Use {corpus_root.parent / 'results' / corpus_root.name} instead."
        )

    legacy_benchmarks_root = corpus_root / "benchmarks"
    if legacy_benchmarks_root.exists():
        raise SystemExit(
            "legacy benchmark outputs still exist under the corpus data tree: "
            f"{legacy_benchmarks_root}. Mirror them locally, delete the embedded benchmark tree, "
            f"and keep active outputs under {corpus_root.parent / 'results' / corpus_root.name}."
        )


def stage_run_dir_name(stage_id: str) -> str:
    domain, stage = stage_id.split(".", 1)
    if domain != "fastq":
        raise SystemExit(f"unsupported corpus benchmark domain for {stage_id}")
    return stage.replace(".", "_")


def benchmark_sample_root(run_root: Path, stage_id: str, sample_id: str) -> Path:
    return run_root / "bench" / stage_run_dir_name(stage_id) / sample_id


def benchmark_tool_root(
    run_root: Path,
    stage_id: str,
    sample_id: str,
    tool_id: str,
) -> Path:
    return benchmark_sample_root(run_root, stage_id, sample_id) / "tools" / tool_id


def parse_simple_toml(path: Path) -> dict:
    root: dict = {}
    samples: list[dict] = []
    current_sample: dict | None = None
    for raw_line in path.read_text(encoding="utf-8").splitlines():
        line = raw_line.split("#", 1)[0].strip()
        if not line:
            continue
        if line == "[[samples]]":
            current_sample = {}
            samples.append(current_sample)
            continue
        key, raw_value = [part.strip() for part in line.split("=", 1)]
        if raw_value.startswith('"') and raw_value.endswith('"'):
            value = raw_value[1:-1]
        elif raw_value.isdigit():
            value = int(raw_value)
        else:
            raise SystemExit(f"unsupported TOML value in {path}: {raw_value}")
        target = current_sample if current_sample is not None else root
        target[key] = value
    root["samples"] = samples
    return root


def load_corpus_spec(repo_root: Path) -> dict:
    path = repo_root / "configs" / "runtime" / "corpora" / "corpus-01.toml"
    if tomllib is not None:
        with path.open("rb") as handle:
            return tomllib.load(handle)
    return parse_simple_toml(path)


def discover_normalized_samples(
    corpus_root: Path,
    *,
    expected_total: int = 20,
) -> list[dict]:
    normalized = corpus_root / "normalized"
    if not normalized.is_dir():
        raise SystemExit(f"missing normalized corpus directory: {normalized}")

    sample_ids: set[str] = set()
    for path in normalized.glob("sample_*_R1.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R1.fastq.gz"))
    for path in normalized.glob("sample_*_R2.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R2.fastq.gz"))

    samples: list[dict] = []
    for sample_id in sorted(sample_ids):
        r1 = normalized / f"{sample_id}_R1.fastq.gz"
        r2 = normalized / f"{sample_id}_R2.fastq.gz"
        if not r1.is_file():
            raise SystemExit(f"missing R1 for sample {sample_id}: {r1}")
        samples.append(
            {
                "sample_id": sample_id,
                "r1": r1,
                "r2": r2 if r2.is_file() else None,
                "layout": "pe" if r2.is_file() else "se",
            }
        )
    if len(samples) != expected_total:
        raise SystemExit(
            f"expected {expected_total} normalized samples for corpus-01, found {len(samples)}"
        )
    return samples


def sample_accession_map(corpus_root: Path, spec: dict) -> dict[str, dict]:
    manifest = load_json(corpus_root / "MANIFEST.json")
    manifest_files = manifest["files"]
    hash_to_accessions: dict[str, set[str]] = defaultdict(set)
    for relative_path, digest in manifest_files.items():
        parts = Path(relative_path).parts
        if len(parts) >= 2 and parts[0] == "raw":
            hash_to_accessions[digest].add(parts[1])

    spec_by_accession = {sample["accession"]: sample for sample in spec["samples"]}
    sample_map: dict[str, dict] = {}
    for relative_path, digest in manifest_files.items():
        path = Path(relative_path)
        if len(path.parts) != 2 or path.parts[0] != "normalized":
            continue
        filename = path.name
        if filename.endswith("_R1.fastq.gz"):
            sample_id = filename.removesuffix("_R1.fastq.gz")
        elif filename.endswith("_R2.fastq.gz"):
            sample_id = filename.removesuffix("_R2.fastq.gz")
        else:
            continue
        accessions = sorted(hash_to_accessions.get(digest, set()))
        if len(accessions) != 1:
            raise SystemExit(
                f"expected one accession for {relative_path}, found {accessions or 'none'}"
            )
        accession = accessions[0]
        sample_map.setdefault(sample_id, {}).update(spec_by_accession[accession])
        sample_map[sample_id]["accession"] = accession
    return sample_map


def expected_cohort_counts(spec: dict) -> dict[str, int]:
    return {
        "ancient_se": int(spec["target_ancient_se"]),
        "ancient_pe": int(spec["target_ancient_pe"]),
        "modern_se": int(spec["target_modern_se"]),
        "modern_pe": int(spec["target_modern_pe"]),
    }


def validate_corpus_contract(
    corpus_root: Path,
    spec: dict,
    samples: list[dict],
) -> dict[str, dict]:
    metadata_by_sample = sample_accession_map(corpus_root, spec)
    actual_counts: dict[str, int] = defaultdict(int)

    for sample in samples:
        sample_id = sample["sample_id"]
        metadata = metadata_by_sample.get(sample_id)
        if metadata is None:
            raise SystemExit(f"missing accession metadata for normalized sample {sample_id}")
        cohort = f"{metadata['era']}_{metadata['layout']}"
        actual_counts[cohort] += 1

    expected_counts = expected_cohort_counts(spec)
    if dict(sorted(actual_counts.items())) != dict(sorted(expected_counts.items())):
        raise SystemExit(
            "corpus-01 cohort contract drift: "
            f"expected {dict(sorted(expected_counts.items()))}, "
            f"found {dict(sorted(actual_counts.items()))}"
        )
    return metadata_by_sample


def load_published_sample_metadata(
    repo_root: Path,
    spec: dict,
    *,
    stage_id: str = "fastq.validate_reads",
    expected_total: int = 20,
) -> dict[str, dict]:
    sample_results = (
        repo_root / "docs" / "benchmark" / stage_id / "corpus-01" / "sample_results.csv"
    )
    if not sample_results.is_file():
        raise SystemExit(
            "missing local corpus metadata and published fallback: "
            f"{sample_results}"
        )

    metadata_by_sample: dict[str, dict] = {}
    with sample_results.open("r", encoding="utf-8", newline="") as handle:
        reader = csv.DictReader(handle)
        for row in reader:
            sample_id = row.get("sample_id", "").strip()
            if not sample_id:
                continue
            metadata = {
                "accession": row.get("accession", "").strip(),
                "era": row.get("era", "").strip(),
                "layout": row.get("layout", "").strip(),
                "study_accession": row.get("study_accession", "").strip(),
                "size_band": row.get("size_band", "").strip(),
            }
            if sample_id in metadata_by_sample:
                if metadata_by_sample[sample_id] != metadata:
                    raise SystemExit(
                        "published sample metadata drift: "
                        f"inconsistent rows for {sample_id} in {sample_results}"
                    )
                continue
            metadata_by_sample[sample_id] = metadata

    if len(metadata_by_sample) != expected_total:
        raise SystemExit(
            "published sample metadata drift: "
            f"expected {expected_total} unique samples in {sample_results}, "
            f"found {len(metadata_by_sample)}"
        )

    expected_counts = expected_cohort_counts(spec)
    actual_counts: dict[str, int] = defaultdict(int)
    for sample_id, metadata in sorted(metadata_by_sample.items()):
        accession = metadata.get("accession")
        era = metadata.get("era")
        layout = metadata.get("layout")
        if not accession or not era or not layout:
            raise SystemExit(
                "published sample metadata drift: "
                f"missing accession/era/layout for {sample_id} in {sample_results}"
            )
        actual_counts[f"{era}_{layout}"] += 1

    if dict(sorted(actual_counts.items())) != dict(sorted(expected_counts.items())):
        raise SystemExit(
            "published corpus metadata drift: "
            f"expected {dict(sorted(expected_counts.items()))}, "
            f"found {dict(sorted(actual_counts.items()))}"
        )

    return metadata_by_sample


def resolve_corpus_metadata(
    repo_root: Path,
    corpus_root: Path,
    spec: dict,
    *,
    expected_sample_ids: list[str],
    fallback_stage_id: str = "fastq.validate_reads",
) -> dict[str, dict]:
    if (corpus_root / "normalized").is_dir() and (corpus_root / "MANIFEST.json").is_file():
        return validate_corpus_contract(
            corpus_root,
            spec,
            discover_normalized_samples(corpus_root, expected_total=len(expected_sample_ids)),
        )

    metadata_by_sample = load_published_sample_metadata(
        repo_root,
        spec,
        stage_id=fallback_stage_id,
        expected_total=sum(expected_cohort_counts(spec).values()),
    )
    missing_samples = sorted(set(expected_sample_ids) - set(metadata_by_sample))
    if missing_samples:
        raise SystemExit(
            "published sample metadata drift: "
            f"missing rows for samples {missing_samples}"
        )
    return metadata_by_sample


def select_paired_samples(
    spec: dict,
    samples: list[dict],
    metadata_by_sample: dict[str, dict],
) -> list[dict]:
    paired_samples = [
        sample
        for sample in samples
        if metadata_by_sample[sample["sample_id"]]["layout"] == "pe"
    ]
    actual_counts: dict[str, int] = defaultdict(int)
    for sample in paired_samples:
        metadata = metadata_by_sample[sample["sample_id"]]
        actual_counts[f"{metadata['era']}_{metadata['layout']}"] += 1

    expected_counts = {
        "ancient_pe": int(spec["target_ancient_pe"]),
        "modern_pe": int(spec["target_modern_pe"]),
    }
    if dict(sorted(actual_counts.items())) != dict(sorted(expected_counts.items())):
        raise SystemExit(
            "paired corpus contract drift: "
            f"expected {dict(sorted(expected_counts.items()))}, "
            f"found {dict(sorted(actual_counts.items()))}"
        )
    expected_total = sum(expected_counts.values())
    if len(paired_samples) != expected_total:
        raise SystemExit(
            f"expected {expected_total} paired normalized samples for corpus-01, "
            f"found {len(paired_samples)}"
        )
    return paired_samples


def normalize_tool_csv(raw: str) -> list[str]:
    tools: list[str] = []
    seen: set[str] = set()
    for tool in (entry.strip() for entry in raw.split(",")):
        if not tool or tool in seen:
            continue
        seen.add(tool)
        tools.append(tool)
    if not tools:
        raise SystemExit("tool roster must not be empty")
    return tools


def run_repo_command(repo_root: Path, *args: str) -> str:
    completed = subprocess.run(
        list(args),
        cwd=repo_root,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        stderr = completed.stderr.strip()
        stdout = completed.stdout.strip()
        detail = stderr or stdout or f"exit code {completed.returncode}"
        raise SystemExit(f"command failed: {' '.join(args)}: {detail}")
    return completed.stdout.strip()


def registry_tools_for_stage(
    repo_root: Path,
    stage_id: str,
    kind: str,
    *,
    scenario_id: str | None = None,
) -> list[str]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "registry",
        "list-tools",
        "--stage",
        stage_id,
        "--kind",
        kind,
    ]
    if scenario_id:
        command.extend(["--scenario", scenario_id])
    output = run_repo_command(
        repo_root,
        *command,
    )
    return normalize_tool_csv(output)


def require_canonical_tool_roster(
    repo_root: Path,
    stage_id: str,
    tools: list[str],
    *,
    scenario_id: str | None = None,
    kind: str = "benchmark",
) -> list[str]:
    expected = registry_tools_for_stage(
        repo_root,
        stage_id,
        kind,
        scenario_id=scenario_id,
    )
    return require_exact_tool_roster(stage_id, tools, expected)


def require_exact_tool_roster(
    stage_id: str,
    tools: list[str],
    expected: list[str],
) -> list[str]:
    expected_set = set(expected)
    tools_set = set(tools)
    if tools_set != expected_set:
        raise SystemExit(
            f"{stage_id} corpus benchmark requires the full governed tool roster "
            f"{expected}, received {tools}"
        )
    return [tool for tool in expected if tool in tools_set]
