#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import hashlib
import json
import os
import statistics
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from datetime import datetime, timezone
from functools import lru_cache
from pathlib import Path

try:
    import tomllib as toml_loader  # type: ignore[attr-defined]
except ModuleNotFoundError:
    try:
        import tomli as toml_loader  # type: ignore[no-redef]
    except ModuleNotFoundError:
        toml_loader = None


REPO_ROOT = Path(__file__).resolve().parents[2]
LOCAL_RESULTS_ROOT: Path | None = None
LOCAL_CACHE_MIRROR_ROOT: Path | None = None
PUBLISHED_DOSSIER_NAME = "benchmark.md"
LEGACY_PUBLISHED_DOSSIER_NAME = "lunarc.md"


@lru_cache(maxsize=1)
def load_benchmark_workspace_config() -> dict:
    path = REPO_ROOT / "configs" / "bench" / "workspace.toml"
    if not path.is_file() or toml_loader is None:
        return {}
    with path.open("rb") as handle:
        return toml_loader.load(handle)


@lru_cache(maxsize=1)
def load_benchmark_publication_config() -> dict:
    path = REPO_ROOT / "configs" / "bench" / "publication.toml"
    if not path.is_file() or toml_loader is None:
        return {}
    with path.open("rb") as handle:
        return toml_loader.load(handle)


def _workspace_path(section: str, key: str) -> Path:
    value = load_benchmark_workspace_config().get(section, {}).get(key)
    if isinstance(value, str) and value.strip():
        return Path(value).expanduser()
    raise SystemExit(
        "missing benchmark workspace path contract: "
        f"[{section}].{key} in configs/bench/workspace.toml"
    )


def _workspace_string(section: str, key: str) -> str:
    value = load_benchmark_workspace_config().get(section, {}).get(key)
    if isinstance(value, str) and value.strip():
        return value.strip()
    raise SystemExit(
        "missing benchmark workspace string contract: "
        f"[{section}].{key} in configs/bench/workspace.toml"
    )


def _workspace_template(section: str, key: str) -> str:
    value = load_benchmark_workspace_config().get("artifacts", {}).get(section, {}).get(key)
    if isinstance(value, str) and value.strip():
        return value
    raise SystemExit(
        "missing benchmark artifact template contract: "
        f"[artifacts.{section}].{key} in configs/bench/workspace.toml"
    )


def _workspace_layout_template(section: str, key: str) -> str:
    value = load_benchmark_workspace_config().get("layout", {}).get(section, {}).get(key)
    if isinstance(value, str) and value.strip():
        return value
    raise SystemExit(
        "missing benchmark layout template contract: "
        f"[layout.{section}].{key} in configs/bench/workspace.toml"
    )


def _expand_workspace_template(template: str, values: dict[str, str]) -> Path:
    return Path(template.format(**values))


def benchmark_local_results_root() -> Path:
    if LOCAL_RESULTS_ROOT is not None:
        return LOCAL_RESULTS_ROOT
    return _workspace_path("local", "results_root")


def benchmark_local_cache_mirror_root() -> Path:
    if LOCAL_CACHE_MIRROR_ROOT is not None:
        return LOCAL_CACHE_MIRROR_ROOT
    return _workspace_path("local", "cache_mirror_root")


def benchmark_local_extra_data_root() -> Path:
    return _workspace_path("local", "extra_data_root")


def benchmark_local_reference_root() -> Path:
    return _workspace_path("local", "reference_root")


def benchmark_remote_repo_root() -> Path:
    return _workspace_path("remote", "repo_root")


def benchmark_remote_frontend_root() -> Path:
    return benchmark_remote_repo_root().parent


def benchmark_remote_ssh_host() -> str:
    return _workspace_string("remote", "ssh_host")


def benchmark_remote_corpus_root() -> Path:
    return _workspace_path("remote", "corpus_root")


def benchmark_remote_cache_root() -> Path:
    return _workspace_path("remote", "cache_root")


def benchmark_remote_results_root() -> Path:
    return _workspace_path("remote", "results_root")


def benchmark_remote_results_legacy_root() -> Path:
    return _workspace_path("remote", "results_legacy_root")


def benchmark_remote_extra_data_root() -> Path:
    return _workspace_path("remote", "extra_data_root")


def benchmark_remote_containers_root() -> Path:
    return _workspace_path("remote", "containers_root")


def benchmark_remote_reference_root() -> Path:
    return _workspace_path("remote", "reference_root")


def published_dossier_path(stage_docs_root: Path) -> Path:
    return stage_docs_root / PUBLISHED_DOSSIER_NAME


def resolve_existing_dossier_path(stage_docs_root: Path) -> Path:
    preferred = published_dossier_path(stage_docs_root)
    if preferred.is_file():
        return preferred
    legacy = stage_docs_root / LEGACY_PUBLISHED_DOSSIER_NAME
    if legacy.is_file():
        return legacy
    return preferred


def parse_corpus_report_args(
    *,
    description: str,
    docs_root: str,
) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default=docs_root,
        help="Directory where summary artifacts should be written.",
    )
    return parser.parse_args()


def parse_corpus_briefing_args(
    *,
    description: str,
    docs_root: str,
) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=description)
    parser.add_argument(
        "--docs-root",
        default=docs_root,
        help="Directory that contains summary.json and sample_results.csv.",
    )
    return parser.parse_args()


@dataclass(frozen=True)
class CorpusBriefingRuntime:
    docs_root: Path
    summary: dict
    sample_rows: list[dict]


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_csv_rows(path: Path) -> list[dict]:
    with path.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def find_cohort_entry(
    rows: list[dict],
    *,
    tool: str,
    dimension: str,
    cohort: str,
) -> dict | None:
    for row in rows:
        if (
            row.get("tool") == tool
            and row.get("dimension") == dimension
            and row.get("cohort") == cohort
        ):
            return row
    return None


def percentile(values: list[float], fraction: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = round((len(ordered) - 1) * fraction)
    return float(ordered[index])


def fmt_runtime(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}"


def fmt_fraction(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.1%}"


def fmt_value(value: float | int | None) -> str:
    if value is None:
        return "n/a"
    numeric = float(value)
    if numeric.is_integer():
        return str(int(numeric))
    return f"{numeric:.3f}"


def fmt_metric(value: float | int | None) -> str:
    if value is None:
        return "n/a"
    return f"{float(value):.3f}"


def fmt_csv_value(value: object) -> object:
    if isinstance(value, float):
        return f"{value:.6f}"
    return value


def optional_float(value: object) -> float | None:
    if value in {"", None}:
        return None
    return float(value)


def optional_int(value: object) -> int | None:
    if value in {"", None}:
        return None
    return int(value)


def write_csv_artifact(
    path: Path,
    rows: list[dict],
    *,
    fieldnames: list[str] | None = None,
    empty_error: str = "cannot write empty csv artifact: {path}",
) -> None:
    if not rows:
        raise SystemExit(empty_error.format(path=path))
    selected_fieldnames = fieldnames or list(rows[0].keys())
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=selected_fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(
                {
                    key: fmt_csv_value(row.get(key))
                    for key in selected_fieldnames
                }
            )


def publish_corpus_briefing_artifacts(
    docs_root: Path,
    *,
    markdown: str,
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outlier_rows: list[dict],
    runtime_fieldnames: list[str] | None = None,
    cohort_fieldnames: list[str] | None = None,
    outlier_fieldnames: list[str] | None = None,
    empty_error: str = "cannot write empty csv artifact: {path}",
) -> None:
    write_csv_artifact(
        docs_root / "tool_runtime_summary.csv",
        runtime_rows,
        fieldnames=runtime_fieldnames,
        empty_error=empty_error,
    )
    write_csv_artifact(
        docs_root / "cohort_runtime_summary.csv",
        cohort_rows,
        fieldnames=cohort_fieldnames,
        empty_error=empty_error,
    )
    write_csv_artifact(
        docs_root / "sample_runtime_outliers.csv",
        outlier_rows,
        fieldnames=outlier_fieldnames,
        empty_error=empty_error,
    )
    (docs_root / PUBLISHED_DOSSIER_NAME).write_text(markdown, encoding="utf-8")


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


def resolve_bowtie2_index_prefix(path: Path) -> Path:
    resolved = path.expanduser().resolve()
    if not resolved.is_dir():
        return resolved
    prefixes = sorted(
        {
            candidate.name.removesuffix(".1.bt2")
            for candidate in resolved.glob("*.1.bt2")
            if candidate.is_file()
            and not candidate.name.endswith(".rev.1.bt2")
        }
        | {
            candidate.name.removesuffix(".1.bt2l")
            for candidate in resolved.glob("*.1.bt2l")
            if candidate.is_file()
            and not candidate.name.endswith(".rev.1.bt2l")
        }
    )
    if not prefixes:
        raise FileNotFoundError(
            f"missing Bowtie2 index prefix under directory: {resolved}"
        )
    if len(prefixes) != 1:
        raise ValueError(
            f"ambiguous Bowtie2 index prefixes under directory {resolved}: {prefixes!r}"
        )
    return resolved / prefixes[0]


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


def resolve_artifact_lineage_json(path: Path) -> Path | None:
    resolved = path.expanduser().resolve()
    candidates: list[Path] = []
    if resolved.is_dir():
        candidates.append(resolved / "lineage.json")
    else:
        candidates.append(resolved.parent / "lineage.json")
    for candidate in candidates:
        if candidate.is_file():
            return candidate
    return None


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


@dataclass(frozen=True)
class StageRunRootCandidate:
    source: str
    path: Path


@dataclass(frozen=True)
class StageRunRootSelection:
    selected: StageRunRootCandidate
    newest_available: StageRunRootCandidate | None


@dataclass(frozen=True)
class CorpusReportRuntime:
    repo_root: Path
    corpus_root: Path
    run_root: Path
    docs_root: Path
    local_results_root: Path
    run_manifest: dict


def corpus_01_publication_contract(stage_id: str) -> CorpusBenchmarkContract:
    for row in load_benchmark_publication_config().get("corpus_01", {}).get("contracts", []):
        if str(row.get("stage_id")) != stage_id:
            continue
        return CorpusBenchmarkContract(
            stage_id=str(row["stage_id"]),
            scenario_id=str(row["scenario_id"]),
            tools=[str(tool) for tool in row["tools"]],
            sample_scope=str(row.get("sample_scope", "full")),
        )
    raise SystemExit(
        "missing benchmark publication contract: "
        f"stage_id={stage_id} in configs/bench/publication.toml"
    )


DETECT_ADAPTERS_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.detect_adapters"
)


PROFILE_READS_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.profile_reads")


PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.profile_read_lengths"
)


PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.profile_overrepresented_sequences"
)


MERGE_PAIRS_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.merge_pairs")


REPORT_QC_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.report_qc")


TRIM_POLYG_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.trim_polyg_tails"
)


TRIM_READS_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.trim_reads")


TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.trim_terminal_damage"
)


NORMALIZE_PRIMERS_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.normalize_primers"
)


FILTER_READS_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.filter_reads")


DEPLETE_RRNA_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.deplete_rrna")


REMOVE_DUPLICATES_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.remove_duplicates"
)


FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.filter_low_complexity"
)


DEPLETE_HOST_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.deplete_host")


DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.deplete_reference_contaminants"
)


CORRECT_ERRORS_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.correct_errors"
)


EXTRACT_UMIS_BENCHMARK_CONTRACT = corpus_01_publication_contract("fastq.extract_umis")


SCREEN_TAXONOMY_BENCHMARK_CONTRACT = corpus_01_publication_contract(
    "fastq.screen_taxonomy"
)


CORPUS_01_PUBLICATION_CONTRACTS = [
    CorpusBenchmarkContract(
        stage_id=str(row["stage_id"]),
        scenario_id=str(row["scenario_id"]),
        tools=[str(tool) for tool in row["tools"]],
        sample_scope=str(row.get("sample_scope", "full")),
    )
    for row in load_benchmark_publication_config()
    .get("corpus_01", {})
    .get("contracts", [])
]


CORPUS_01_PUBLICATION_EXCLUSIONS = [
    CorpusBenchmarkExclusion(
        stage_id=str(row["stage_id"]),
        reason=str(row["reason"]),
    )
    for row in load_benchmark_publication_config()
    .get("corpus_01", {})
    .get("exclusions", [])
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


CORPUS_01_STAGE_TARGET_STEMS = {
    "fastq.validate_reads": "validate",
    "fastq.trim_polyg_tails": "trim-polyg",
    "fastq.trim_reads": "trim-reads",
    "fastq.trim_terminal_damage": "trim-terminal-damage",
    "fastq.detect_adapters": "detect-adapters",
    "fastq.profile_reads": "profile-reads",
    "fastq.profile_read_lengths": "profile-read-lengths",
    "fastq.profile_overrepresented_sequences": "profile-overrepresented",
    "fastq.filter_low_complexity": "filter-low-complexity",
    "fastq.filter_reads": "filter-reads",
    "fastq.remove_duplicates": "remove-duplicates",
    "fastq.merge_pairs": "merge",
    "fastq.report_qc": "report-qc",
    "fastq.normalize_primers": "normalize-primers",
    "fastq.deplete_rrna": "deplete-rrna",
    "fastq.deplete_host": "deplete-host",
    "fastq.deplete_reference_contaminants": "deplete-reference-contaminants",
    "fastq.screen_taxonomy": "screen-taxonomy",
    "fastq.correct_errors": "correct-errors",
    "fastq.extract_umis": "extract-umis",
}


def corpus_01_make_target_stem(stage_id: str) -> str:
    try:
        return CORPUS_01_STAGE_TARGET_STEMS[stage_id]
    except KeyError as err:
        raise KeyError(f"missing corpus-01 make target stem for stage {stage_id}") from err


def corpus_01_make_run_target(stage_id: str) -> str:
    return f"_benchmark-{corpus_01_make_target_stem(stage_id)}-corpus-01"


def corpus_01_make_report_target(stage_id: str) -> str:
    return f"{corpus_01_make_run_target(stage_id)}-report"


def trim_reads_benchmark_defaults() -> dict:
    return {
        "min_length": None,
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
        "musket_kmer_budget": 536_870_912,
        "genome_size": 3_200_000_000,
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


def _configured_stage_run_root(root: Path, template_key: str, corpus_id: str, stage_id: str) -> Path:
    relative_root = _expand_workspace_template(
        _workspace_layout_template("stage_runs", template_key),
        {
            "corpus_id": corpus_id,
            "stage_id": stage_id,
        },
    )
    return root / relative_root


def default_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return _configured_stage_run_root(
        benchmark_remote_results_root(),
        "remote_results_template",
        corpus_root.name,
        stage_id,
    )


def default_local_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return _configured_stage_run_root(
        benchmark_local_cache_mirror_root(),
        "local_cache_results_template",
        corpus_root.name,
        stage_id,
    )


def legacy_local_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return _configured_stage_run_root(
        benchmark_local_results_root(),
        "local_archive_results_template",
        corpus_root.name,
        stage_id,
    )


def configured_stage_run_roots(corpus_root: Path, stage_id: str) -> list[StageRunRootCandidate]:
    return [
        StageRunRootCandidate(
            source="local-cache-mirror",
            path=default_local_results_stage_root(corpus_root, stage_id),
        ),
        StageRunRootCandidate(
            source="local-results-root",
            path=legacy_local_results_stage_root(corpus_root, stage_id),
        ),
        StageRunRootCandidate(
            source="remote-results-root",
            path=default_results_stage_root(corpus_root, stage_id),
        ),
    ]


def infer_results_archive_root(path: Path) -> Path | None:
    resolved = path.expanduser().resolve()
    for index, part in enumerate(resolved.parts):
        if part.startswith("corpus_") and index > 0:
            return Path(*resolved.parts[:index])
    return None


def _path_is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def _workspace_cache_root_for_output(out_root: Path) -> Path | None:
    resolved = out_root.expanduser().resolve()
    remote_cache_root = benchmark_remote_cache_root().resolve()
    remote_results_roots = [
        benchmark_remote_results_root().resolve(),
        benchmark_remote_results_legacy_root().resolve(),
    ]
    local_results_root = benchmark_local_results_root().resolve()
    local_cache_mirror_root = benchmark_local_cache_mirror_root().resolve()

    if _path_is_under(resolved, remote_cache_root) or any(
        _path_is_under(resolved, root) for root in remote_results_roots
    ):
        return remote_cache_root
    if _path_is_under(resolved, local_results_root) or _path_is_under(
        resolved, local_cache_mirror_root
    ):
        return local_cache_mirror_root
    for candidate in (resolved, *resolved.parents):
        if candidate.name == ".cache":
            return candidate
    return None


def benchmark_runtime_env(out_root: Path) -> dict[str, str]:
    env = os.environ.copy()
    cache_root = _workspace_cache_root_for_output(out_root)
    if cache_root is None:
        return env
    env["BIJUX_CACHE_ROOT"] = str(cache_root)
    env["XDG_CACHE_HOME"] = str(cache_root)
    if cache_root.name == ".cache":
        env["BIJUX_HPC_ROOT"] = str(cache_root.parent)
    return env


def default_extra_data_root(out_root: Path) -> Path:
    cache_root = _workspace_cache_root_for_output(out_root)
    if cache_root is not None:
        if cache_root.resolve() == benchmark_remote_cache_root().resolve():
            return benchmark_remote_extra_data_root()
        return benchmark_local_extra_data_root()
    results_archive_root = infer_results_archive_root(out_root)
    if results_archive_root is not None:
        return results_archive_root / "extra-data"
    return benchmark_local_extra_data_root()


def default_host_reference_index_root(
    out_root: Path,
    *,
    reference_catalog_id: str,
    reference_index_backend: str,
) -> Path:
    relative_root = _expand_workspace_template(
        _workspace_template("fastq_deplete_host", "reference_index_template"),
        {
            "reference_catalog_id": reference_catalog_id,
            "reference_index_backend": reference_index_backend,
        },
    )
    return default_extra_data_root(out_root) / relative_root


def default_screen_taxonomy_database_root(
    out_root: Path,
    *,
    database_namespace: str,
    database_scope: str,
    database_artifact_id: str,
) -> Path:
    relative_root = _expand_workspace_template(
        _workspace_template("fastq_screen_taxonomy", "database_root_template"),
        {
            "database_namespace": database_namespace,
            "database_scope": database_scope,
            "database_artifact_id": database_artifact_id,
        },
    )
    return default_extra_data_root(out_root) / relative_root


def preferred_report_run_root(corpus_root: Path, stage_id: str) -> Path:
    return select_stage_run_root(corpus_root, stage_id).selected.path


def resolve_corpus_briefing_runtime(args: argparse.Namespace) -> CorpusBriefingRuntime:
    docs_root = Path(args.docs_root).resolve()
    return CorpusBriefingRuntime(
        docs_root=docs_root,
        summary=load_json(docs_root / "summary.json"),
        sample_rows=load_csv_rows(docs_root / "sample_results.csv"),
    )


def resolve_corpus_report_runtime(
    args: argparse.Namespace,
    *,
    stage_id: str,
) -> CorpusReportRuntime:
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser()
        if args.run_root
        else preferred_report_run_root(corpus_root, stage_id)
    )
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)
    return CorpusReportRuntime(
        repo_root=repo_root,
        corpus_root=corpus_root,
        run_root=run_root,
        docs_root=docs_root,
        local_results_root=run_root.parents[2],
        run_manifest=load_json(run_root / "run_manifest.json"),
    )


def _parse_utc_timestamp(raw: str | None) -> datetime | None:
    if not raw:
        return None
    normalized = raw.strip().replace("Z", "+00:00")
    if not normalized:
        return None
    try:
        parsed = datetime.fromisoformat(normalized)
    except ValueError:
        return None
    if parsed.tzinfo is None:
        return parsed.replace(tzinfo=timezone.utc)
    return parsed.astimezone(timezone.utc)


def run_root_freshness_timestamp(run_root: Path) -> datetime | None:
    manifest_path = run_root / "run_manifest.json"
    if manifest_path.is_file():
        manifest = load_json(manifest_path)
        for key in (
            "completed_at_utc",
            "generated_at_utc",
            "finished_at_utc",
            "started_at_utc",
        ):
            parsed = _parse_utc_timestamp(str(manifest.get(key, "") or ""))
            if parsed is not None:
                return parsed
    return None


def run_root_observed_timestamp(run_root: Path) -> datetime | None:
    manifest_timestamp = run_root_freshness_timestamp(run_root)
    if manifest_timestamp is not None:
        return manifest_timestamp
    if run_root.exists():
        return datetime.fromtimestamp(run_root.stat().st_mtime, tz=timezone.utc)
    return None


def select_stage_run_root(corpus_root: Path, stage_id: str) -> StageRunRootSelection:
    candidates = configured_stage_run_roots(corpus_root, stage_id)
    existing_candidates = [candidate for candidate in candidates if candidate.path.is_dir()]
    if not existing_candidates:
        return StageRunRootSelection(selected=candidates[0], newest_available=None)

    freshest_candidate = existing_candidates[0]
    freshest_timestamp = run_root_freshness_timestamp(freshest_candidate.path)
    for candidate in existing_candidates[1:]:
        candidate_timestamp = run_root_freshness_timestamp(candidate.path)
        if candidate_timestamp is None:
            continue
        if freshest_timestamp is None or candidate_timestamp > freshest_timestamp:
            freshest_candidate = candidate
            freshest_timestamp = candidate_timestamp
    return StageRunRootSelection(
        selected=freshest_candidate,
        newest_available=freshest_candidate,
    )


def localize_results_path(path_str: str, local_results_root: Path) -> Path:
    path = Path(path_str)
    if path.exists():
        return path
    cache_mirror_root = benchmark_local_cache_mirror_root()
    bijux_dna_results_root = (
        cache_mirror_root / "bijux-dna-results"
        if local_results_root.name == "results"
        and local_results_root.parent == cache_mirror_root
        else local_results_root
    )
    root_mappings = [
        ("/results/", local_results_root),
        ("/bijux-dna-results/", bijux_dna_results_root),
        ("/extra-data/", benchmark_local_extra_data_root()),
        ("/reference/", benchmark_local_reference_root()),
    ]
    for marker, mapped_root in root_mappings:
        if marker in path_str:
            return mapped_root / path_str.split(marker, 1)[1]
    return path


def localize_workspace_path(path_str: str, local_results_root: Path) -> Path:
    path = Path(path_str)
    if path.exists():
        return path

    cache_mirror_root = benchmark_local_cache_mirror_root()
    remote_root_mappings = [
        (benchmark_remote_results_root(), cache_mirror_root / "results"),
        (benchmark_remote_results_legacy_root(), cache_mirror_root / "bijux-dna-results"),
        (benchmark_remote_extra_data_root(), cache_mirror_root / "extra-data"),
        (benchmark_remote_reference_root(), cache_mirror_root / "reference"),
    ]
    for remote_root, mapped_root in remote_root_mappings:
        if not str(remote_root):
            continue
        try:
            rel = path.relative_to(remote_root)
        except ValueError:
            continue
        return mapped_root / rel
    return localize_results_path(path_str, local_results_root)


def localize_manifest_paths(
    run_manifest: dict,
    local_results_root: Path,
    *,
    keys: list[str],
) -> dict[str, str]:
    localized: dict[str, str] = {}
    for key in keys:
        value = str(run_manifest.get(key, "") or "").strip()
        if not value:
            continue
        localized[key] = str(localize_workspace_path(value, local_results_root))
    return localized


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
    if toml_loader is not None:
        with path.open("rb") as handle:
            return toml_loader.load(handle)
    return parse_simple_toml(path)


def discover_normalized_samples(
    corpus_root: Path,
    *,
    expected_total: int | None = None,
) -> list[dict]:
    normalized = corpus_root / "normalized"
    if not normalized.is_dir():
        raise SystemExit(f"missing normalized corpus directory: {normalized}")
    target_total = (
        corpus_01_expected_sample_total()
        if expected_total is None
        else expected_total
    )

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
    if len(samples) != target_total:
        raise SystemExit(
            f"expected {target_total} normalized samples for corpus-01, found {len(samples)}"
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


def corpus_01_expected_sample_total(repo_root: Path = REPO_ROOT) -> int:
    return sum(expected_cohort_counts(load_corpus_spec(repo_root)).values())


def full_scope_publication_stage_ids() -> list[str]:
    return [
        contract.stage_id
        for contract in CORPUS_01_PUBLICATION_CONTRACTS
        if contract.sample_scope == "full"
    ]


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
    stage_id: str | None = None,
    expected_total: int | None = None,
) -> dict[str, dict]:
    target_total = (
        expected_total
        if expected_total is not None
        else sum(expected_cohort_counts(spec).values())
    )
    candidate_stage_ids = (
        [stage_id]
        if stage_id is not None
        else full_scope_publication_stage_ids()
    )
    errors: list[str] = []
    for candidate_stage_id in candidate_stage_ids:
        sample_results = (
            repo_root
            / "docs"
            / "benchmark"
            / candidate_stage_id
            / "corpus-01"
            / "sample_results.csv"
        )
        if not sample_results.is_file():
            errors.append(f"missing {sample_results}")
            continue

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

        if len(metadata_by_sample) != target_total:
            errors.append(
                "published sample metadata drift: "
                f"expected {target_total} unique samples in {sample_results}, "
                f"found {len(metadata_by_sample)}"
            )
            continue

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
            errors.append(
                "published corpus metadata drift: "
                f"expected {dict(sorted(expected_counts.items()))}, "
                f"found {dict(sorted(actual_counts.items()))} in {sample_results}"
            )
            continue
        return metadata_by_sample

    raise SystemExit(
        "missing local corpus metadata and published fallback: "
        + "; ".join(errors)
    )


def resolve_corpus_metadata(
    repo_root: Path,
    corpus_root: Path,
    spec: dict,
    *,
    expected_sample_ids: list[str],
    fallback_stage_id: str | None = None,
) -> dict[str, dict]:
    if (corpus_root / "normalized").is_dir() and (corpus_root / "MANIFEST.json").is_file():
        full_corpus_total = sum(expected_cohort_counts(spec).values())
        return validate_corpus_contract(
            corpus_root,
            spec,
            discover_normalized_samples(corpus_root, expected_total=full_corpus_total),
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


def benchmark_manifest_sample_ids(run_manifest: dict) -> list[str]:
    sample_ids: list[str] = []
    seen: set[str] = set()
    for run in run_manifest.get("runs", []):
        sample_id = str(run.get("sample_id", "")).strip()
        if not sample_id:
            raise SystemExit("benchmark run manifest drift: missing sample_id in runs[]")
        if sample_id in seen:
            continue
        seen.add(sample_id)
        sample_ids.append(sample_id)
    return sample_ids


def benchmark_applicable_sample_ids(
    contract: CorpusBenchmarkContract,
    run_manifest: dict,
    metadata_by_sample: dict[str, dict],
) -> list[str]:
    sample_ids = benchmark_manifest_sample_ids(run_manifest)
    if contract.sample_scope == "full":
        return sample_ids
    if contract.sample_scope == "paired":
        return [
            sample_id
            for sample_id in sample_ids
            if metadata_by_sample[sample_id]["layout"] == "pe"
        ]
    raise SystemExit(
        f"unsupported corpus benchmark sample_scope for {contract.stage_id}: "
        f"{contract.sample_scope}"
    )


def benchmark_applicable_runs(
    contract: CorpusBenchmarkContract,
    run_manifest: dict,
    metadata_by_sample: dict[str, dict],
) -> list[dict]:
    applicable_sample_ids = set(
        benchmark_applicable_sample_ids(contract, run_manifest, metadata_by_sample)
    )
    return [
        run
        for run in run_manifest.get("runs", [])
        if str(run.get("sample_id", "")).strip() in applicable_sample_ids
    ]


def benchmark_manifest_failure_count(runs: list[dict]) -> int:
    failures = 0
    for run in runs:
        if int(run.get("exit_code") or 0) != 0:
            failures += 1
    return failures


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


def registry_all_tools_for_stage(repo_root: Path, stage_id: str) -> list[str]:
    output = run_repo_command(
        repo_root,
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


def registry_contract_is_available(repo_root: Path) -> bool:
    return (repo_root / "Cargo.toml").is_file() and (
        repo_root / "configs" / "ci" / "registry" / "tool_registry.toml"
    ).is_file()


@lru_cache(maxsize=None)
def _cached_registry_tools_for_stage(
    repo_root_str: str,
    stage_id: str,
    kind: str,
    scenario_id: str,
) -> tuple[str, ...]:
    repo_root = Path(repo_root_str)
    return tuple(
        registry_tools_for_stage(
            repo_root,
            stage_id,
            kind,
            scenario_id=scenario_id or None,
        )
    )


def resolve_benchmark_tool_roster(
    repo_root: Path,
    stage_id: str,
    scenario_id: str | None,
    fallback: list[str],
) -> tuple[list[str], str | None]:
    if not registry_contract_is_available(repo_root):
        return list(fallback), None
    try:
        tools = list(
            _cached_registry_tools_for_stage(
                str(repo_root.resolve()),
                stage_id,
                "benchmark",
                scenario_id or "",
            )
        )
    except SystemExit as err:
        return list(fallback), str(err)
    return tools, None


@lru_cache(maxsize=None)
def _cached_stage_toolset(repo_root_str: str, stage_id: str) -> tuple[str, ...]:
    repo_root = Path(repo_root_str)
    return tuple(registry_all_tools_for_stage(repo_root, stage_id))


def resolve_stage_toolset(
    repo_root: Path,
    stage_id: str,
    fallback: list[str],
) -> tuple[list[str], str | None]:
    if not registry_contract_is_available(repo_root):
        return list(fallback), None
    try:
        tools = list(_cached_stage_toolset(str(repo_root.resolve()), stage_id))
    except SystemExit as err:
        return list(fallback), str(err)
    return tools, None
