#!/usr/bin/env python3
from __future__ import annotations

import argparse
import os

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)
from corpus_01_fastq_benchmark_support import trim_terminal_damage_benchmark_defaults


def parse_args() -> argparse.Namespace:
    defaults = trim_terminal_damage_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.trim_terminal_damage benchmarks for corpus-01."
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
        help="Comma-separated terminal-damage tool ids. Defaults to the governed benchmark cohort.",
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
    parser.add_argument("--damage-mode", default=defaults["damage_mode"])
    parser.add_argument("--execution-policy", default=defaults["execution_policy"])
    parser.add_argument("--trim-5p-bases", type=int, default=defaults["trim_5p_bases"])
    parser.add_argument("--trim-3p-bases", type=int, default=defaults["trim_3p_bases"])
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
    append_stage_arg(stage_args, "--damage-mode", args.damage_mode)
    append_stage_arg(stage_args, "--execution-policy", args.execution_policy)
    append_stage_arg(stage_args, "--trim-5p-bases", args.trim_5p_bases)
    append_stage_arg(stage_args, "--trim-3p-bases", args.trim_3p_bases)
    return run_corpus_stage_compat(
        stage_id="fastq.trim_terminal_damage",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
