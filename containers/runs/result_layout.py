#!/usr/bin/env python3
"""Migrate governed container results to flat, recoverable run identities."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import sys
from typing import Any


PLAN_SCHEMA = "bijux.container.result_layout_plan.v1"
RUN_ID = re.compile(r"run-[1-9][0-9]{3,}")
TOOL_ID = re.compile(r"[a-z0-9_]+")


class ResultLayoutError(RuntimeError):
    """Raised when a result move cannot preserve recorded evidence."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_suffix(path.suffix + ".partial")
    partial.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    partial.replace(path)


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ResultLayoutError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ResultLayoutError(f"JSON document must be an object: {path}")
    return value


def require_relative_child(value: str, label: str) -> Path:
    path = Path(value)
    if path.is_absolute() or ".." in path.parts or not path.parts:
        raise ResultLayoutError(f"{label} is not a safe relative path: {value}")
    return path


def inventory_digest(files: list[dict[str, Any]]) -> str:
    normalized = json.dumps(files, sort_keys=True, separators=(",", ":")).encode()
    return hashlib.sha256(normalized).hexdigest()


def validate_inventory(result: Path, files: list[dict[str, Any]]) -> None:
    expected_paths: set[Path] = set()
    for row in files:
        if not isinstance(row, dict):
            raise ResultLayoutError(f"invalid inventory row for {result}")
        relative = require_relative_child(str(row.get("path", "")), "inventory path")
        if relative in expected_paths:
            raise ResultLayoutError(
                f"duplicate inventory path for {result}: {relative}"
            )
        expected_paths.add(relative)
        candidate = result / relative
        if not candidate.is_file():
            raise ResultLayoutError(f"recorded result file is absent: {candidate}")
        if candidate.stat().st_size != row.get("size_bytes"):
            raise ResultLayoutError(f"recorded result size changed: {candidate}")
        if sha256_file(candidate) != row.get("sha256"):
            raise ResultLayoutError(f"recorded result digest changed: {candidate}")
    actual_paths = {
        path.relative_to(result) for path in result.rglob("*") if path.is_file()
    }
    if actual_paths != expected_paths:
        unrecorded = sorted(str(path) for path in actual_paths - expected_paths)
        missing = sorted(str(path) for path in expected_paths - actual_paths)
        raise ResultLayoutError(
            f"result inventory differs for {result}; "
            f"unrecorded={unrecorded}, missing={missing}"
        )


def tracked_locks(runs_root: Path) -> list[tuple[Path, dict[str, Any]]]:
    records = []
    for lock_path in sorted(runs_root.glob("run-*-*/evidence.lock.json")):
        record = load_json(lock_path)
        run_id = record.get("run_id")
        tool = record.get("tool")
        if (
            not isinstance(run_id, str)
            or RUN_ID.fullmatch(run_id) is None
            or not isinstance(tool, str)
            or TOOL_ID.fullmatch(tool) is None
            or lock_path.parent.name != f"{run_id}-{tool}"
        ):
            raise ResultLayoutError(f"tracked run identity is invalid: {lock_path}")
        records.append((lock_path, record))
    if not records:
        raise ResultLayoutError(f"no tracked run evidence found under {runs_root}")
    return records


def plan_layout(runs_root: Path, result_root: Path, output: Path) -> int:
    result_root = result_root.resolve()
    moves = []
    seen_targets: set[Path] = set()
    for lock_path, record in tracked_locks(runs_root):
        source_result = require_relative_child(
            str(record.get("source_result", "")), "source result"
        )
        prefix = Path("results/container-validation")
        try:
            source_relative = source_result.relative_to(prefix)
        except ValueError as exc:
            raise ResultLayoutError(
                f"source result is outside container validation: {lock_path}"
            ) from exc
        run_id = str(record["run_id"])
        tool = str(record["tool"])
        target_relative = Path(f"{run_id}-{tool}")
        if source_relative == target_relative:
            continue
        if source_relative != Path(tool) / run_id:
            raise ResultLayoutError(
                f"historical result layout is not recognized: {source_relative}"
            )
        source = result_root / source_relative
        target = result_root / target_relative
        if target in seen_targets:
            raise ResultLayoutError(f"duplicate flat result target: {target}")
        seen_targets.add(target)
        if not source.is_dir():
            raise ResultLayoutError(f"historical result directory is absent: {source}")
        if target.exists() or target.is_symlink():
            raise ResultLayoutError(f"flat result target already exists: {target}")
        files = record.get("source_files")
        if not isinstance(files, list) or not files:
            raise ResultLayoutError(f"source inventory is absent: {lock_path}")
        validate_inventory(source, files)
        moves.append(
            {
                "run_id": run_id,
                "tool": tool,
                "source": source_relative.as_posix(),
                "target": target_relative.as_posix(),
                "file_count": len(files),
                "size_bytes": sum(int(row["size_bytes"]) for row in files),
                "inventory_sha256": inventory_digest(files),
                "evidence_lock": lock_path.relative_to(runs_root).as_posix(),
                "evidence_lock_sha256": sha256_file(lock_path),
                "applied": False,
            }
        )
    write_json(
        output,
        {
            "schema_version": PLAN_SCHEMA,
            "state": "previewed",
            "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "result_root": str(result_root),
            "run_count": len(moves),
            "file_count": sum(int(move["file_count"]) for move in moves),
            "size_bytes": sum(int(move["size_bytes"]) for move in moves),
            "moves": moves,
            "removed_empty_directories": [],
        },
    )
    return len(moves)


def validate_plan(
    plan_path: Path, runs_root: Path, result_root: Path
) -> dict[str, Any]:
    plan = load_json(plan_path)
    if plan.get("schema_version") != PLAN_SCHEMA:
        raise ResultLayoutError(f"unsupported result-layout plan: {plan_path}")
    if Path(str(plan.get("result_root", ""))).resolve() != result_root.resolve():
        raise ResultLayoutError("plan result root differs from the requested root")
    moves = plan.get("moves")
    if not isinstance(moves, list):
        raise ResultLayoutError("plan moves must be an array")
    for move in moves:
        if not isinstance(move, dict):
            raise ResultLayoutError("plan move must be an object")
        lock_relative = require_relative_child(
            str(move.get("evidence_lock", "")), "evidence lock"
        )
        lock_path = runs_root / lock_relative
        if not lock_path.is_file() or sha256_file(lock_path) != move.get(
            "evidence_lock_sha256"
        ):
            raise ResultLayoutError(f"tracked evidence lock changed: {lock_path}")
        record = load_json(lock_path)
        files = record.get("source_files")
        if not isinstance(files, list) or inventory_digest(files) != move.get(
            "inventory_sha256"
        ):
            raise ResultLayoutError(f"tracked source inventory changed: {lock_path}")
    return plan


def apply_layout(plan_path: Path, runs_root: Path, result_root: Path) -> int:
    plan = validate_plan(plan_path, runs_root, result_root)
    if plan.get("state") not in {"previewed", "applying"}:
        raise ResultLayoutError(
            f"plan cannot be applied from state {plan.get('state')}"
        )
    plan["state"] = "applying"
    write_json(plan_path, plan)
    moved = 0
    for move in plan["moves"]:
        source = result_root / require_relative_child(move["source"], "source")
        target = result_root / require_relative_child(move["target"], "target")
        lock = load_json(runs_root / move["evidence_lock"])
        files = lock["source_files"]
        if move.get("applied"):
            validate_inventory(target, files)
            moved += 1
            continue
        if not source.is_dir() or target.exists() or target.is_symlink():
            raise ResultLayoutError(
                f"result move precondition failed: {source} -> {target}"
            )
        validate_inventory(source, files)
        source.rename(target)
        validate_inventory(target, files)
        move["applied"] = True
        move["applied_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
        write_json(plan_path, plan)
        moved += 1
    removed = []
    for tool_root in sorted(
        {(result_root / move["source"]).parent for move in plan["moves"]}
    ):
        if tool_root.is_dir() and not any(tool_root.iterdir()):
            tool_root.rmdir()
            removed.append(tool_root.relative_to(result_root).as_posix())
    plan["removed_empty_directories"] = removed
    plan["state"] = "applied"
    plan["completed_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(plan_path, plan)
    return moved


def verify_layout(plan_path: Path, runs_root: Path, result_root: Path) -> int:
    plan = validate_plan(plan_path, runs_root, result_root)
    if plan.get("state") != "applied":
        raise ResultLayoutError(f"plan is not applied: {plan.get('state')}")
    for move in plan["moves"]:
        source = result_root / require_relative_child(move["source"], "source")
        target = result_root / require_relative_child(move["target"], "target")
        if source.exists() or source.is_symlink():
            raise ResultLayoutError(f"nested result still exists: {source}")
        lock = load_json(runs_root / move["evidence_lock"])
        validate_inventory(target, lock["source_files"])
    return len(plan["moves"])


def rollback_layout(plan_path: Path, runs_root: Path, result_root: Path) -> int:
    plan = validate_plan(plan_path, runs_root, result_root)
    if plan.get("state") != "applied":
        raise ResultLayoutError(
            f"plan cannot be rolled back from state {plan.get('state')}"
        )
    restored = 0
    for move in reversed(plan["moves"]):
        source = result_root / require_relative_child(move["source"], "source")
        target = result_root / require_relative_child(move["target"], "target")
        lock = load_json(runs_root / move["evidence_lock"])
        validate_inventory(target, lock["source_files"])
        if source.exists() or source.is_symlink():
            raise ResultLayoutError(f"rollback source already exists: {source}")
        source.parent.mkdir(parents=True, exist_ok=True)
        target.rename(source)
        move["applied"] = False
        move.pop("applied_at_utc", None)
        write_json(plan_path, plan)
        restored += 1
    plan["state"] = "rolled_back"
    plan["rolled_back_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(plan_path, plan)
    return restored


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    for name in ("plan", "apply", "verify", "rollback"):
        command = commands.add_parser(name)
        command.add_argument("--runs-root", type=Path, required=True)
        command.add_argument("--result-root", type=Path, required=True)
        command.add_argument("--plan", type=Path, required=True)
    return root


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "plan":
            count = plan_layout(args.runs_root, args.result_root, args.plan)
            print(f"planned {count} governed result moves")
        elif args.command == "apply":
            count = apply_layout(args.plan, args.runs_root, args.result_root)
            print(f"applied {count} governed result moves")
        elif args.command == "verify":
            count = verify_layout(args.plan, args.runs_root, args.result_root)
            print(f"verified {count} flat governed results")
        else:
            count = rollback_layout(args.plan, args.runs_root, args.result_root)
            print(f"restored {count} nested governed results")
        return 0
    except ResultLayoutError as exc:
        print(f"container result layout failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
