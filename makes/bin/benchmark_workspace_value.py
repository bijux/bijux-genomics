#!/usr/bin/env python3
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path


def resolve_workspace_value(
    key_path: str,
    *,
    config: str = "",
) -> str:
    repo_root = Path(__file__).resolve().parents[2]
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "bench",
        "workspace-value",
    ]
    if config:
        command.extend(["--config", config])
    command.append(key_path)
    completed = subprocess.run(
        command,
        cwd=repo_root,
        check=True,
        capture_output=True,
        text=True,
    )
    value = completed.stdout.strip()
    if value == "true":
        return "1"
    if value == "false":
        return "0"
    return value


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
    print(resolve_workspace_value(args.key_path, config=args.config))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
