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
from corpus_01_fastq_benchmark_support import trim_reads_benchmark_defaults


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
    defaults = trim_reads_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.trim_reads benchmarks for corpus-01."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root that contains Cargo.toml and configs/runtime/corpora/corpus-01.toml.",
    )
    parser.add_argument(
        "--corpus-root",
        default="",
        help="Materialized corpus root. Defaults to the preferred_root from the corpus spec.",
    )
    parser.add_argument(
        "--out-root",
        default="",
        help="Benchmark output root. Defaults to the configured stage benchmark root.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="",
        help="Comma-separated trim tool ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--threads", type=int, default=1)
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
    parser.add_argument("--min-length", type=int, default=defaults["min_length"])
    parser.add_argument("--quality-cutoff", type=int, default=defaults["quality_cutoff"])
    parser.add_argument("--n-policy", default=defaults["n_policy"])
    parser.add_argument("--adapter-policy", default=defaults["adapter_policy"])
    parser.add_argument("--polyx-policy", default=defaults["polyx_policy"])
    parser.add_argument("--contaminant-policy", default=defaults["contaminant_policy"])
    parser.add_argument("--adapter-bank-preset", default=defaults["adapter_bank_preset"] or "")
    parser.add_argument("--polyx-preset", default=defaults["polyx_preset"] or "")
    parser.add_argument(
        "--contaminant-preset",
        default=defaults["contaminant_preset"] or "",
    )
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


def optional_str(raw: str) -> str | None:
    value = raw.strip()
    return value or None


def build_command(
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    min_length: int | None,
    quality_cutoff: int | None,
    n_policy: str,
    adapter_policy: str,
    polyx_policy: str,
    contaminant_policy: str,
    adapter_bank_preset: str | None,
    polyx_preset: str | None,
    contaminant_preset: str | None,
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
        "trim-reads",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--threads",
        str(threads),
        "--n-policy",
        n_policy,
        "--adapter-policy",
        adapter_policy,
        "--polyx-policy",
        polyx_policy,
        "--contaminant-policy",
        contaminant_policy,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if min_length is not None:
        command.extend(["--min-length", str(min_length)])
    if quality_cutoff is not None:
        command.extend(["--quality-cutoff", str(quality_cutoff)])
    if adapter_bank_preset is not None:
        command.extend(["--adapter-bank-preset", adapter_bank_preset])
    if polyx_preset is not None:
        command.extend(["--polyx-preset", polyx_preset])
    if contaminant_preset is not None:
        command.extend(["--contaminant-preset", contaminant_preset])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "trim_reads" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "trim_reads" / sample_id


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
    env: dict[str, str],
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False, env=env)
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
    adapter_bank_preset = optional_str(args.adapter_bank_preset)
    polyx_preset = optional_str(args.polyx_preset)
    contaminant_preset = optional_str(args.contaminant_preset)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--min-length", args.min_length)
    append_stage_arg(stage_args, "--quality-cutoff", args.quality_cutoff)
    append_stage_arg(stage_args, "--n-policy", args.n_policy)
    append_stage_arg(stage_args, "--adapter-policy", args.adapter_policy)
    append_stage_arg(stage_args, "--polyx-policy", args.polyx_policy)
    append_stage_arg(stage_args, "--contaminant-policy", args.contaminant_policy)
    append_stage_arg(stage_args, "--adapter-bank-preset", adapter_bank_preset)
    append_stage_arg(stage_args, "--polyx-preset", polyx_preset)
    append_stage_arg(stage_args, "--contaminant-preset", contaminant_preset)
    return run_corpus_stage_compat(
        stage_id="fastq.trim_reads",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
