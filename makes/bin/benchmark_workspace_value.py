#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

import corpus_01_fastq_benchmark_support as support


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print a configured benchmark workspace value."
    )
    parser.add_argument(
        "key_path",
        help="Dotted workspace key path, for example remote.corpus_root.",
    )
    return parser.parse_args()


def resolve_workspace_value(key_path: str) -> str:
    if key_path == "remote.corpus_root":
        return str(support.benchmark_remote_corpus_root())
    if key_path == "remote.repo_root":
        return str(support.benchmark_remote_repo_root())
    if key_path == "local.results_root":
        return str(support.benchmark_local_results_root())
    if key_path == "local.cache_mirror_root":
        return str(support.benchmark_local_cache_mirror_root())
    raise SystemExit(f"unsupported benchmark workspace key path: {key_path}")


def main() -> int:
    args = parse_args()
    sys.stdout.write(resolve_workspace_value(args.key_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
