#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
from dataclasses import dataclass
from pathlib import Path

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)
from corpus_01_fastq_benchmark_support import remove_duplicates_benchmark_defaults


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
    r2: str
    layout: str
    status: str
    exit_code: int
    command: list[str]
    report_json: str


def parse_args() -> argparse.Namespace:
    defaults = remove_duplicates_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.remove_duplicates benchmarks for the paired corpus-01 cohort."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--threads", type=int, default=None)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--dedup-mode", default=defaults["dedup_mode"])
    parser.add_argument(
        "--keep-order",
        type=parse_bool_literal,
        default=defaults["keep_order"],
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
    threads: int | None,
    jobs: int,
    dedup_mode: str,
    keep_order: bool,
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
        "remove-duplicates",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--r2",
        str(sample["r2"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--dedup-mode",
        dedup_mode,
        "--keep-order",
        "true" if keep_order else "false",
    ]
    if threads is not None:
        command.extend(["--threads", str(threads)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "remove_duplicates" / sample_id / "report.json"


def run_sample_command(
    *,
    repo_root: Path,
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False)
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]),
        layout=sample["layout"],
        status="completed" if completed.returncode == 0 else "failed",
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--dedup-mode", args.dedup_mode)
    append_stage_arg(stage_args, "--keep-order", args.keep_order)
    return run_corpus_stage_compat(
        stage_id="fastq.remove_duplicates",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
