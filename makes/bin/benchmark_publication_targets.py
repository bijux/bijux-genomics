#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def resolve_targets(
    kind: str,
    *,
    config: str = "",
) -> list[str]:
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
    if config:
        command.extend(["--config", config])
    command.append(kind)
    completed = subprocess.run(
        command,
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    return completed.stdout.split()


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
    print(" ".join(resolve_targets(args.kind, config=args.config)))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
