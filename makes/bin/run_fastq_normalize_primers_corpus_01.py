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
from corpus_01_fastq_benchmark_support import normalize_primers_benchmark_defaults


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
    defaults = normalize_primers_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.normalize_primers benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--primer-set-id", default=defaults["primer_set_id"])
    parser.add_argument(
        "--orientation-policy",
        default=defaults["orientation_policy"],
    )
    parser.add_argument(
        "--max-mismatch-rate",
        type=float,
        default=defaults["max_mismatch_rate"],
    )
    parser.add_argument(
        "--min-overlap-bp",
        type=int,
        default=defaults["min_overlap_bp"],
    )
    parser.add_argument(
        "--strict-5p-anchor",
        type=parse_bool_literal,
        default=defaults["strict_5p_anchor"],
    )
    parser.add_argument(
        "--allow-iupac-codes",
        type=parse_bool_literal,
        default=defaults["allow_iupac_codes"],
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
    jobs: int,
    primer_set_id: str,
    orientation_policy: str,
    max_mismatch_rate: float,
    min_overlap_bp: int,
    strict_5p_anchor: bool,
    allow_iupac_codes: bool,
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
        "normalize-primers",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--primer-set-id",
        primer_set_id,
        "--orientation-policy",
        orientation_policy,
        "--max-mismatch-rate",
        str(max_mismatch_rate),
        "--min-overlap-bp",
        str(min_overlap_bp),
        "--strict-5p-anchor",
        "true" if strict_5p_anchor else "false",
        "--allow-iupac-codes",
        "true" if allow_iupac_codes else "false",
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "normalize_primers" / sample_id / "report.json"


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
        r2=str(sample["r2"]) if sample["r2"] is not None else None,
        layout=sample["layout"],
        status="completed" if completed.returncode == 0 else "failed",
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--primer-set-id", args.primer_set_id)
    append_stage_arg(stage_args, "--orientation-policy", args.orientation_policy)
    append_stage_arg(stage_args, "--max-mismatch-rate", args.max_mismatch_rate)
    append_stage_arg(stage_args, "--min-overlap-bp", args.min_overlap_bp)
    append_stage_arg(stage_args, "--strict-5p-anchor", args.strict_5p_anchor)
    append_stage_arg(stage_args, "--allow-iupac-codes", args.allow_iupac_codes)
    return run_corpus_stage_compat(
        stage_id="fastq.normalize_primers",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
