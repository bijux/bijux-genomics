#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os
from pathlib import Path

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run fastq.validate_reads benchmarks for corpus-01."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root that contains Cargo.toml and configs/runtime/corpora/corpus-01.toml.",
    )
    parser.add_argument(
        "--corpus-root",
        default="",
        help="Materialized corpus root. Defaults to the configured corpus benchmark root.",
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
        default="fastqvalidator,fastqc,fastq_scan,seqtk,fqtools",
        help="Comma-separated validate tool ids.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--validation-mode", default="")
    parser.add_argument("--pair-sync-policy", default="")
    add_workspace_config_argument(parser)
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Skip samples that already have report.json in the output tree.",
    )
    return parser.parse_args()


def build_command(
    *,
    repo_root: Path,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    validation_mode: str,
    pair_sync_policy: str,
    sample: dict,
) -> list[str]:
    del repo_root
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
        "validate-reads",
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
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if validation_mode:
        command.extend(["--validation-mode", validation_mode])
    if pair_sync_policy:
        command.extend(["--pair-sync-policy", pair_sync_policy])
    return command


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--validation-mode", args.validation_mode)
    append_stage_arg(stage_args, "--pair-sync-policy", args.pair_sync_policy)
    return run_corpus_stage_compat(
        stage_id="fastq.validate_reads",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
