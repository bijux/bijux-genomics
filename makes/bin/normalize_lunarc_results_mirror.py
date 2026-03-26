#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Normalize raw Lunarc result pulls into the canonical local results mirror layout."
    )
    parser.add_argument(
        "--results-root",
        default="/Users/bijan/bijux/bijux-dna-results",
        help="Local results mirror root.",
    )
    parser.add_argument(
        "--corpus-id",
        default="corpus_01",
        help="Corpus identifier to normalize.",
    )
    parser.add_argument(
        "--dry-run",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Print planned moves without changing the filesystem.",
    )
    return parser.parse_args()


def normalize_results_root(results_root: Path, corpus_id: str, *, dry_run: bool) -> dict:
    raw_root = results_root / "home" / "bijan" / "bijux" / "results" / corpus_id
    actions: list[dict] = []
    if not raw_root.is_dir():
        return {
            "results_root": str(results_root),
            "corpus_id": corpus_id,
            "actions": actions,
        }

    for stage_root in sorted(path for path in raw_root.iterdir() if path.is_dir()):
        for run_root in sorted(path for path in stage_root.iterdir() if path.is_dir()):
            canonical_root = results_root / corpus_id / stage_root.name / run_root.name
            if canonical_root.exists():
                actions.append(
                    {
                        "stage_id": stage_root.name,
                        "run_id": run_root.name,
                        "status": "skipped_existing_target",
                        "source": str(run_root),
                        "target": str(canonical_root),
                    }
                )
                continue
            actions.append(
                {
                    "stage_id": stage_root.name,
                    "run_id": run_root.name,
                    "status": "planned_move" if dry_run else "moved",
                    "source": str(run_root),
                    "target": str(canonical_root),
                }
            )
            if dry_run:
                continue
            canonical_root.parent.mkdir(parents=True, exist_ok=True)
            run_root.rename(canonical_root)

    return {
        "results_root": str(results_root),
        "corpus_id": corpus_id,
        "actions": actions,
    }


def main() -> int:
    args = parse_args()
    results_root = Path(args.results_root).expanduser().resolve()
    report = normalize_results_root(
        results_root,
        args.corpus_id,
        dry_run=args.dry_run,
    )
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
