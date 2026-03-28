#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run fastq.profile_overrepresented_sequences benchmarks for corpus-01."
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
        default="",
        help="Comma-separated overrepresented-sequence profiler ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--top-k", type=int, default=50)
    parser.add_argument(
        "--sample-limit",
        type=int,
        default=0,
        help="Optional positive limit for local or frontend preflight runs.",
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
    append_stage_arg(stage_args, "--top-k", args.top_k)
    return run_corpus_stage_compat(
        stage_id="fastq.profile_overrepresented_sequences",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
