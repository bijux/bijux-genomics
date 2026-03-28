#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import shutil
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Converge legacy benchmark workspace roots into their governed canonical roots."
    )
    parser.add_argument("--canonical-root", required=True)
    parser.add_argument("--legacy-root", required=True)
    parser.add_argument(
        "--confirm",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Apply moves and deletions. Defaults to dry-run.",
    )
    return parser.parse_args()


def entry_summary(path: Path) -> dict:
    if not path.exists():
        return {
            "exists": False,
            "kind": "missing",
            "file_count": 0,
            "total_size_bytes": 0,
            "mtime": None,
        }
    if path.is_file():
        stat = path.stat()
        return {
            "exists": True,
            "kind": "file",
            "file_count": 1,
            "total_size_bytes": stat.st_size,
            "mtime": int(stat.st_mtime),
        }
    file_count = 0
    total_size = 0
    newest_mtime = int(path.stat().st_mtime)
    for child in path.rglob("*"):
        stat = child.stat()
        newest_mtime = max(newest_mtime, int(stat.st_mtime))
        if child.is_file():
            file_count += 1
            total_size += stat.st_size
    return {
        "exists": True,
        "kind": "directory",
        "file_count": file_count,
        "total_size_bytes": total_size,
        "mtime": newest_mtime,
    }


def remove_path(path: Path) -> None:
    if path.is_dir():
        shutil.rmtree(path)
        return
    path.unlink()


def move_path(source: Path, target: Path) -> None:
    target.parent.mkdir(parents=True, exist_ok=True)
    source.rename(target)


def plan_convergence(canonical_root: Path, legacy_root: Path) -> dict:
    actions: list[dict] = []
    if not legacy_root.exists():
        return {
            "canonical_root": str(canonical_root),
            "legacy_root": str(legacy_root),
            "actions": actions,
            "removable_legacy_root": False,
        }

    legacy_entries = sorted(legacy_root.iterdir(), key=lambda path: path.name)
    for legacy_entry in legacy_entries:
        canonical_entry = canonical_root / legacy_entry.name
        legacy_stats = entry_summary(legacy_entry)
        canonical_stats = entry_summary(canonical_entry)
        if not canonical_entry.exists():
            actions.append(
                {
                    "action": "move-legacy-entry",
                    "entry_name": legacy_entry.name,
                    "source": str(legacy_entry),
                    "target": str(canonical_entry),
                    "legacy_summary": legacy_stats,
                    "canonical_summary": canonical_stats,
                }
            )
            continue
        if (
            canonical_stats["file_count"] >= legacy_stats["file_count"]
            and canonical_stats["total_size_bytes"] >= legacy_stats["total_size_bytes"]
            and (canonical_stats["mtime"] or 0) >= (legacy_stats["mtime"] or 0)
        ):
            actions.append(
                {
                    "action": "remove-legacy-duplicate",
                    "entry_name": legacy_entry.name,
                    "source": str(legacy_entry),
                    "target": str(canonical_entry),
                    "legacy_summary": legacy_stats,
                    "canonical_summary": canonical_stats,
                }
            )
            continue
        actions.append(
            {
                "action": "manual-review-required",
                "entry_name": legacy_entry.name,
                "source": str(legacy_entry),
                "target": str(canonical_entry),
                "legacy_summary": legacy_stats,
                "canonical_summary": canonical_stats,
            }
        )

    removable_legacy_root = all(
        action["action"] != "manual-review-required" for action in actions
    )
    return {
        "canonical_root": str(canonical_root),
        "legacy_root": str(legacy_root),
        "actions": actions,
        "removable_legacy_root": removable_legacy_root,
    }


def apply_convergence(plan: dict) -> dict:
    report = dict(plan)
    applied_actions: list[dict] = []
    for action in plan["actions"]:
        source = Path(action["source"])
        target = Path(action["target"])
        if action["action"] == "move-legacy-entry":
            move_path(source, target)
        elif action["action"] == "remove-legacy-duplicate":
            remove_path(source)
        elif action["action"] == "manual-review-required":
            applied_actions.append({**action, "status": "pending-manual-review"})
            continue
        applied_actions.append({**action, "status": "applied"})

    legacy_root = Path(plan["legacy_root"])
    legacy_removed = False
    if plan["removable_legacy_root"] and legacy_root.exists() and not any(legacy_root.iterdir()):
        legacy_root.rmdir()
        legacy_removed = True

    report["actions"] = applied_actions
    report["legacy_root_removed"] = legacy_removed
    return report


def main() -> int:
    args = parse_args()
    canonical_root = Path(args.canonical_root).expanduser().resolve()
    legacy_root = Path(args.legacy_root).expanduser().resolve()
    plan = plan_convergence(canonical_root, legacy_root)
    report = apply_convergence(plan) if args.confirm else plan
    report["mode"] = "confirm" if args.confirm else "dry-run"
    print(json.dumps(report, indent=2) + "\n", end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
