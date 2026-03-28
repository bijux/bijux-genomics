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
from corpus_01_fastq_benchmark_support import filter_reads_benchmark_defaults


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
    defaults = filter_reads_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.filter_reads benchmarks for corpus-01."
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
    parser.add_argument(
        "--sample-jobs",
        type=int,
        default=1,
        help="Number of corpus samples to benchmark concurrently.",
    )
    parser.add_argument(
        "--sample-limit",
        type=int,
        default=0,
        help="Optional positive limit for local or frontend preflight runs.",
    )
    parser.add_argument("--max-n", type=int, default=defaults["max_n"])
    parser.add_argument(
        "--max-n-fraction",
        type=float,
        default=defaults["max_n_fraction"],
    )
    parser.add_argument(
        "--max-n-count",
        type=int,
        default=defaults["max_n_count"],
    )
    parser.add_argument(
        "--low-complexity-threshold",
        type=float,
        default=defaults["low_complexity_threshold"],
    )
    parser.add_argument(
        "--entropy-threshold",
        type=float,
        default=defaults["entropy_threshold"],
    )
    parser.add_argument("--kmer-ref", default=defaults["kmer_ref"])
    parser.add_argument("--polyx-policy", default=defaults["polyx_policy"])
    add_workspace_config_argument(parser)
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Skip samples that already have report.json in the output tree.",
    )
    parser.add_argument(
        "--dry-run",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Write the run manifest without executing sample commands.",
    )
    return parser.parse_args()


def build_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    max_n: int | None,
    max_n_fraction: float | None,
    max_n_count: int | None,
    low_complexity_threshold: float | None,
    entropy_threshold: float | None,
    kmer_ref: str | None,
    polyx_policy: str | None,
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
        "filter",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if max_n is not None:
        command.extend(["--max-n", str(max_n)])
    if max_n_fraction is not None:
        command.extend(["--max-n-fraction", str(max_n_fraction)])
    if max_n_count is not None:
        command.extend(["--max-n-count", str(max_n_count)])
    if low_complexity_threshold is not None:
        command.extend(["--low-complexity-threshold", str(low_complexity_threshold)])
    if entropy_threshold is not None:
        command.extend(["--entropy-threshold", str(entropy_threshold)])
    if kmer_ref:
        command.extend(["--kmer-ref", kmer_ref])
    if polyx_policy:
        command.extend(["--polyx-policy", polyx_policy])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "filter" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "filter" / sample_id


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


def run_sample_command(
    *,
    repo_root: Path,
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False)
    status = "completed" if completed.returncode == 0 else "failed"
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]) if sample["r2"] is not None else None,
        layout=sample["layout"],
        status=status,
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--max-n", args.max_n)
    append_stage_arg(stage_args, "--max-n-fraction", args.max_n_fraction)
    append_stage_arg(stage_args, "--max-n-count", args.max_n_count)
    append_stage_arg(stage_args, "--low-complexity-threshold", args.low_complexity_threshold)
    append_stage_arg(stage_args, "--entropy-threshold", args.entropy_threshold)
    append_stage_arg(stage_args, "--kmer-ref", args.kmer_ref)
    append_stage_arg(stage_args, "--polyx-policy", args.polyx_policy)
    return run_corpus_stage_compat(
        stage_id="fastq.filter_reads",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
