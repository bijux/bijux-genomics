#!/usr/bin/env python3
from __future__ import annotations

import argparse
import sys

from benchmark_fastq_corpus import configure_workspace_config_path
from benchmark_fastq_corpus.workspace_values import resolve_workspace_value


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print a configured benchmark workspace value."
    )
    parser.add_argument(
        "--config",
        default="",
        help="Benchmark workspace config path.",
    )
    parser.add_argument(
        "key_path",
        help="Dotted workspace key path, for example remote.corpus_root.",
    )
    return parser.parse_args()


def main() -> int:
    args = parse_args()
    configure_workspace_config_path(args.config)
    sys.stdout.write(resolve_workspace_value(args.key_path))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
