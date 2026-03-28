#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)
from corpus_01_fastq_benchmark_support import filter_low_complexity_benchmark_defaults


def parse_args() -> argparse.Namespace:
    defaults = filter_low_complexity_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.filter_low_complexity benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
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
    parser.add_argument(
        "--entropy-threshold",
        type=float,
        default=defaults["entropy_threshold"],
    )
    parser.add_argument(
        "--polyx-threshold",
        type=int,
        default=defaults["polyx_threshold"],
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


def main() -> int:
    args = parse_args()
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--entropy-threshold", args.entropy_threshold)
    append_stage_arg(stage_args, "--polyx-threshold", args.polyx_threshold)
    return run_corpus_stage_compat(
        stage_id="fastq.filter_low_complexity",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
