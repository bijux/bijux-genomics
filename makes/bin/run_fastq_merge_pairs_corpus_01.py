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
from corpus_01_fastq_benchmark_support import merge_pairs_benchmark_defaults


def parse_args() -> argparse.Namespace:
    defaults = merge_pairs_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.merge_pairs benchmarks for the paired corpus-01 cohort."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
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
        help="Comma-separated merge tool ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--sample-jobs",
        type=int,
        default=1,
        help="Number of corpus samples to benchmark concurrently.",
    )
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--merge-overlap", type=int, default=defaults["merge_overlap"])
    parser.add_argument("--min-length", type=int, default=defaults["min_length"])
    parser.add_argument(
        "--unmerged-read-policy",
        default=defaults["unmerged_read_policy"],
        help="emit_unmerged_pairs or omit_unmerged_pairs.",
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


def build_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    merge_overlap: int | None,
    min_length: int | None,
    unmerged_read_policy: str,
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
        "merge",
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
        "--threads",
        str(threads),
        "--unmerged-read-policy",
        unmerged_read_policy,
    ]
    if merge_overlap is not None:
        command.extend(["--merge-overlap", str(merge_overlap)])
    if min_length is not None:
        command.extend(["--min-length", str(min_length)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    return command


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--merge-overlap", args.merge_overlap)
    append_stage_arg(stage_args, "--min-length", args.min_length)
    append_stage_arg(stage_args, "--unmerged-read-policy", args.unmerged_read_policy)
    return run_corpus_stage_compat(
        stage_id="fastq.merge_pairs",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
