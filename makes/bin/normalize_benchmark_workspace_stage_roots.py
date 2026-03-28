#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

import converge_benchmark_workspace_roots as converge_workspace_roots
import corpus_01_fastq_benchmark_support as support


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Normalize duplicate local benchmark stage roots into the governed cache mirror layout."
    )
    parser.add_argument(
        "--corpus-id",
        default="corpus_01",
        help="Corpus identifier to normalize under the local benchmark workspace.",
    )
    parser.add_argument(
        "--confirm",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Apply moves and deletions. Defaults to dry-run.",
    )
    return parser.parse_args()


def _stage_names(root: Path) -> list[str]:
    if not root.is_dir():
        return []
    return sorted(path.name for path in root.iterdir() if path.is_dir())


def _remove_empty_parents(root: Path, stop_at: Path) -> None:
    current = root
    while current != stop_at and current.exists():
        try:
            current.rmdir()
        except OSError:
            break
        current = current.parent


def apply_stage_plan(plan: dict) -> dict:
    return converge_workspace_roots.apply_convergence(plan)


def normalize_stage_roots(*, corpus_id: str, confirm: bool) -> dict:
    local_results_root = support.benchmark_local_results_root()
    local_cache_mirror_root = support.benchmark_local_cache_mirror_root()
    legacy_corpus_root = local_results_root / corpus_id
    canonical_corpus_root = local_cache_mirror_root / "results" / corpus_id
    archive_stage_ids = _stage_names(legacy_corpus_root)
    cache_stage_ids = _stage_names(canonical_corpus_root)
    shared_stage_ids = sorted(set(archive_stage_ids) & set(cache_stage_ids))
    archive_only_stage_ids = sorted(set(archive_stage_ids) - set(cache_stage_ids))
    cache_only_stage_ids = sorted(set(cache_stage_ids) - set(archive_stage_ids))

    plan = converge_workspace_roots.plan_convergence(
        canonical_corpus_root,
        legacy_corpus_root,
    )
    convergence_report = apply_stage_plan(plan) if confirm else plan
    stage_reports: list[dict] = []
    for action in convergence_report["actions"]:
        stage_reports.append({**action, "stage_id": action["entry_name"]})

    if confirm and legacy_corpus_root.exists():
        _remove_empty_parents(legacy_corpus_root, local_results_root)

    manual_review_stage_ids = [
        report["stage_id"]
        for report in stage_reports
        if report["action"] == "manual-review-required"
    ]
    moved_stage_ids = [
        report["stage_id"]
        for report in stage_reports
        if report["action"] == "move-legacy-entry"
    ]
    removed_duplicate_stage_ids = [
        report["stage_id"]
        for report in stage_reports
        if report["action"] == "remove-legacy-duplicate"
    ]
    return {
        "corpus_id": corpus_id,
        "canonical_corpus_root": str(canonical_corpus_root),
        "legacy_corpus_root": str(legacy_corpus_root),
        "archive_stage_ids": archive_stage_ids,
        "cache_stage_ids": cache_stage_ids,
        "shared_stage_ids": shared_stage_ids,
        "archive_only_stage_ids": archive_only_stage_ids,
        "cache_only_stage_ids": cache_only_stage_ids,
        "mode": "confirm" if confirm else "dry-run",
        "status": "needs-review" if manual_review_stage_ids else "clear",
        "stage_reports": stage_reports,
        "moved_stage_ids": moved_stage_ids,
        "removed_duplicate_stage_ids": removed_duplicate_stage_ids,
        "manual_review_stage_ids": manual_review_stage_ids,
    }


def main() -> int:
    args = parse_args()
    report = normalize_stage_roots(corpus_id=args.corpus_id, confirm=args.confirm)
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
