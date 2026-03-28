#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)
from corpus_01_fastq_benchmark_support import (
    artifact_bundle_exists,
    correct_errors_benchmark_defaults,
)


def parse_bool_literal(value: str) -> bool:
    lowered = value.strip().lower()
    if lowered in {"true", "1", "yes", "y"}:
        return True
    if lowered in {"false", "0", "no", "n"}:
        return False
    raise argparse.ArgumentTypeError(f"invalid boolean literal: {value}")


@dataclass
class SampleRun:
    sample_id: str
    r1: str
    r2: str | None
    layout: str
    status: str
    exit_code: int
    command: list[str]
    report_json: str


def parse_args() -> argparse.Namespace:
    defaults = correct_errors_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.correct_errors benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--threads", type=int, default=defaults["threads"])
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument(
        "--quality-encoding",
        default=defaults["quality_encoding"],
    )
    parser.add_argument("--kmer-size", type=int, default=defaults["kmer_size"])
    parser.add_argument(
        "--musket-kmer-budget",
        type=int,
        default=defaults["musket_kmer_budget"],
    )
    parser.add_argument("--genome-size", type=int, default=defaults["genome_size"])
    parser.add_argument(
        "--max-memory-gb",
        type=int,
        default=defaults["max_memory_gb"],
    )
    parser.add_argument(
        "--trusted-kmer-artifact",
        default=defaults["trusted_kmer_artifact"],
    )
    parser.add_argument(
        "--conservative-mode",
        type=parse_bool_literal,
        default=defaults["conservative_mode"],
    )
    add_workspace_config_argument(parser)
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    parser.add_argument(
        "--dry-run",
        action=argparse.BooleanOptionalAction,
        default=False,
    )
    return parser.parse_args()


def build_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    quality_encoding: str,
    kmer_size: int | None,
    musket_kmer_budget: int | None,
    genome_size: int | None,
    max_memory_gb: int | None,
    trusted_kmer_artifact: Path | None,
    conservative_mode: bool,
    sample: dict,
) -> list[str]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "--platform",
        platform,
        "bench",
        "fastq",
        "correct",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--quality-encoding",
        quality_encoding,
    ]
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if kmer_size is not None:
        command.extend(["--kmer-size", str(kmer_size)])
    if musket_kmer_budget is not None:
        command.extend(["--musket-kmer-budget", str(musket_kmer_budget)])
    if genome_size is not None:
        command.extend(["--genome-size", str(genome_size)])
    if max_memory_gb is not None:
        command.extend(["--max-memory-gb", str(max_memory_gb)])
    if trusted_kmer_artifact is not None:
        command.extend(["--trusted-kmer-artifact", str(trusted_kmer_artifact)])
    command.extend(["--conservative-mode", "true" if conservative_mode else "false"])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "correct_errors" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "correct_errors" / sample_id


def sample_report_is_resume_ready(sample_report: Path) -> bool:
    try:
        payload = json.loads(sample_report.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    if payload.get("failures"):
        return False
    gate = payload.get("gate")
    if isinstance(gate, dict) and gate.get("passes") is False:
        return False
    records = payload.get("records")
    return isinstance(records, list) and bool(records)


def reset_sample_payload(out_root: Path, sample_id: str) -> None:
    current_sample_root = sample_root(out_root, sample_id)
    if current_sample_root.is_dir():
        shutil.rmtree(current_sample_root)


def resolve_trusted_kmer_artifact(args: argparse.Namespace) -> Path | None:
    if not args.trusted_kmer_artifact:
        return None
    artifact = Path(args.trusted_kmer_artifact).expanduser().resolve()
    if not artifact_bundle_exists(artifact):
        raise SystemExit(
            f"missing trusted k-mer artifact bundle or prefix matches: {artifact}"
        )
    return artifact


def run_sample_command(
    *,
    repo_root: Path,
    runtime_env: dict[str, str],
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False, env=runtime_env)
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]) if sample["r2"] is not None else None,
        layout=sample["layout"],
        status="completed" if completed.returncode == 0 else "failed",
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


def main() -> int:
    args = parse_args()
    trusted_kmer_artifact = resolve_trusted_kmer_artifact(args)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--quality-encoding", args.quality_encoding)
    append_stage_arg(stage_args, "--kmer-size", args.kmer_size)
    append_stage_arg(stage_args, "--musket-kmer-budget", args.musket_kmer_budget)
    append_stage_arg(stage_args, "--genome-size", args.genome_size)
    append_stage_arg(stage_args, "--max-memory-gb", args.max_memory_gb)
    append_stage_arg(stage_args, "--trusted-kmer-artifact", trusted_kmer_artifact)
    append_stage_arg(stage_args, "--conservative-mode", args.conservative_mode)
    return run_corpus_stage_compat(
        stage_id="fastq.correct_errors",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
