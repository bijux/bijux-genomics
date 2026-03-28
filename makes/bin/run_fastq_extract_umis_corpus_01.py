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
from corpus_01_fastq_benchmark_support import extract_umis_benchmark_defaults


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
    defaults = extract_umis_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.extract_umis benchmarks for the paired corpus-01 cohort."
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
    parser.add_argument("--umi-pattern", default=defaults["umi_pattern"])
    parser.add_argument(
        "--allow-missing-umi-headers",
        action=argparse.BooleanOptionalAction,
        default=defaults["allow_missing_umi_headers"],
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
    umi_pattern: str,
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
        "umi",
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
        "--umi-pattern",
        umi_pattern,
    ]
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "extract_umis" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "extract_umis" / sample_id


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
    allow_missing_umi_headers: bool,
) -> SampleRun:
    env = os.environ.copy()
    if allow_missing_umi_headers:
        env["BIJUX_ALLOW_NO_UMI"] = "1"
    else:
        env.pop("BIJUX_ALLOW_NO_UMI", None)
    completed = subprocess.run(command, cwd=repo_root, check=False, env=env)
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
    append_stage_arg(stage_args, "--umi-pattern", args.umi_pattern)
    extra_env = os.environ.copy()
    if args.allow_missing_umi_headers:
        extra_env["BIJUX_ALLOW_NO_UMI"] = "1"
    else:
        extra_env.pop("BIJUX_ALLOW_NO_UMI", None)
    return run_corpus_stage_compat(
        stage_id="fastq.extract_umis",
        args=args,
        stage_args=stage_args,
        extra_env=extra_env,
    )


if __name__ == "__main__":
    raise SystemExit(main())
