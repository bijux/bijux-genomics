#!/usr/bin/env python3
from __future__ import annotations

import argparse

from benchmark_fastq_corpus.support import (
    CORPUS_01_PUBLICATION_CONTRACTS,
    corpus_01_make_report_target,
    corpus_01_make_run_target,
)


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


def resolve_targets(kind: str) -> list[str]:
    target_builder = (
        corpus_01_make_report_target if kind == "report" else corpus_01_make_run_target
    )
    return [target_builder(contract.stage_id) for contract in CORPUS_01_PUBLICATION_CONTRACTS]


def main() -> int:
    args = parse_args()
    print(" ".join(resolve_targets(args.kind)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
