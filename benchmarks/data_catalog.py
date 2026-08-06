#!/usr/bin/env python3
"""Compatibility adapter for the crate-owned governed data index command."""

from __future__ import annotations

import argparse
import datetime as dt
import subprocess
from pathlib import Path


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--catalog-root", type=Path, required=True)
    root.add_argument("--data-root", type=Path, required=True)
    root.add_argument("--output", type=Path, required=True)
    root.add_argument("--report", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    repository = Path(__file__).resolve().parents[1]
    created_at_utc = dt.datetime.now(dt.timezone.utc).isoformat()
    completed = subprocess.run(
        [
            "cargo",
            "run",
            "-q",
            "-p",
            "bijux-dna",
            "--",
            "bench",
            "data-index",
            "--catalog-root",
            str(args.catalog_root),
            "--data-root",
            str(args.data_root),
            "--output",
            str(args.output),
            "--report",
            str(args.report),
            "--created-at-utc",
            created_at_utc,
        ],
        cwd=repository,
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
