#!/usr/bin/env python3
from __future__ import annotations

import argparse

from benchmark_fastq_corpus.publication_targets import resolve_targets


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Print governed corpus-01 FASTQ benchmark make targets."
    )
    parser.add_argument(
        "kind",
        choices=("run", "report"),
        help="Whether to print corpus run targets or report targets.",
    )
    return parser.parse_args()

def main() -> int:
    args = parse_args()
    print(" ".join(resolve_targets(args.kind)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
