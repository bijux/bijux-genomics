#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
import sys
from pathlib import Path

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
    parser.add_argument(
        "--config",
        default="",
        help="Benchmark publication config. Defaults to configs/bench/publication.toml.",
    )
    return parser.parse_args()

def main() -> int:
    args = parse_args()
    repo_root = Path(__file__).resolve().parents[2]
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "bench",
        "publication-targets",
    ]
    if args.config:
        command.extend(["--config", args.config])
    command.append(args.kind)
    completed = subprocess.run(command, cwd=repo_root, check=False)
    if completed.returncode == 0:
        return 0
    print(" ".join(resolve_targets(args.kind)))
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
