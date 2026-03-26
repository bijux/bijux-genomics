#!/usr/bin/env python3
from __future__ import annotations

import json
import subprocess
from collections import defaultdict
from dataclasses import dataclass
from pathlib import Path

try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:
    tomllib = None


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


@dataclass(frozen=True)
class CorpusBenchmarkContract:
    stage_id: str
    scenario_id: str
    tools: list[str]


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


def merge_pairs_benchmark_defaults() -> dict:
    return {
        "merge_overlap": None,
        "min_length": None,
        "unmerged_read_policy": "emit_unmerged_pairs",
    }


def default_results_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return corpus_root.parent / "results" / corpus_root.name / stage_id / "lunarc"


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
