#!/usr/bin/env python3
"""Quarantine superseded and interrupted SIF files with a reversible plan."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import sys
from pathlib import Path
from typing import Any

PLAN_SCHEMA = "bijux.container.image_storage_plan.v1"
INVENTORY_SCHEMA = "bijux.container.image_inventory.v1"


class ImageStorageError(RuntimeError):
    """Raised when an image move cannot preserve recorded evidence."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ImageStorageError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ImageStorageError(f"JSON document must be an object: {path}")
    return value


def write_json(path: Path, value: Any) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_suffix(path.suffix + ".partial")
    partial.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    partial.replace(path)


def require_under(path: Path, root: Path, label: str) -> Path:
    resolved = path.resolve()
    try:
        resolved.relative_to(root.resolve())
    except ValueError as exc:
        raise ImageStorageError(f"{label} is outside {root}: {path}") from exc
    return resolved


def validate_file(path: Path, size: int, digest: str) -> None:
    if not path.is_file() or path.stat().st_size != size:
        raise ImageStorageError(f"image size or type changed: {path}")
    if sha256_file(path) != digest:
        raise ImageStorageError(f"image digest changed: {path}")


def plan_storage(
    inventory_path: Path,
    image_root: Path,
    quarantine_root: Path,
    output: Path,
) -> int:
    if output.exists() or output.is_symlink():
        raise ImageStorageError(f"refusing to replace image storage plan: {output}")
    inventory = load_json(inventory_path)
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ImageStorageError("image inventory schema is not supported")
    image_root = image_root.resolve()
    quarantine_root = quarantine_root.resolve()
    if quarantine_root == image_root or image_root not in quarantine_root.parents:
        raise ImageStorageError("quarantine root must be beneath the image root")
    moves = []
    for tool in inventory.get("tools", []):
        for candidate in tool.get("candidates", []):
            if candidate.get("disposition") != "unreferenced_obsolete":
                continue
            if candidate.get("real_run_references") != 0:
                raise ImageStorageError(
                    "obsolete candidate still has real-run references"
                )
            moves.append(("obsolete", candidate))
    for image in inventory.get("unknown_images", []):
        if image.get("disposition") != "unreferenced_obsolete":
            continue
        if image.get("real_run_references") != 0:
            raise ImageStorageError(
                "obsolete unknown image still has real-run references"
            )
        moves.append(("obsolete", image))
    moves.extend(("interrupted", row) for row in inventory.get("staged_images", []))
    records = []
    targets: set[Path] = set()
    for category, row in moves:
        source = require_under(Path(str(row["path"])), image_root, "image")
        target = quarantine_root / category / source.name
        if target in targets or target.exists() or target.is_symlink():
            raise ImageStorageError(f"quarantine target already exists: {target}")
        targets.add(target)
        validate_file(source, int(row["size_bytes"]), str(row["sha256"]))
        records.append(
            {
                "category": category,
                "source": str(source),
                "target": str(target),
                "size_bytes": row["size_bytes"],
                "sha256": row["sha256"],
                "applied": False,
            }
        )
    write_json(
        output,
        {
            "schema_version": PLAN_SCHEMA,
            "state": "previewed",
            "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "inventory": {
                "path": str(inventory_path),
                "sha256": sha256_file(inventory_path),
            },
            "image_root": str(image_root),
            "quarantine_root": str(quarantine_root),
            "file_count": len(records),
            "size_bytes": sum(int(row["size_bytes"]) for row in records),
            "moves": records,
        },
    )
    return len(records)


def validate_plan(plan_path: Path, image_root: Path) -> dict[str, Any]:
    plan = load_json(plan_path)
    if plan.get("schema_version") != PLAN_SCHEMA:
        raise ImageStorageError("image storage plan schema is not supported")
    if Path(str(plan.get("image_root", ""))).resolve() != image_root.resolve():
        raise ImageStorageError("plan image root differs from requested root")
    inventory = plan.get("inventory", {})
    inventory_path = Path(str(inventory.get("path", "")))
    if not inventory_path.is_file() or sha256_file(inventory_path) != inventory.get(
        "sha256"
    ):
        raise ImageStorageError("source image inventory changed")
    return plan


def apply_storage(plan_path: Path, image_root: Path) -> int:
    plan = validate_plan(plan_path, image_root)
    if plan.get("state") not in {"previewed", "applying"}:
        raise ImageStorageError(f"plan cannot be applied from {plan.get('state')}")
    plan["state"] = "applying"
    write_json(plan_path, plan)
    for row in plan["moves"]:
        source = require_under(Path(row["source"]), image_root, "image")
        target = require_under(
            Path(row["target"]), Path(plan["quarantine_root"]), "quarantine target"
        )
        if row["applied"]:
            validate_file(target, row["size_bytes"], row["sha256"])
            continue
        validate_file(source, row["size_bytes"], row["sha256"])
        if target.exists() or target.is_symlink():
            raise ImageStorageError(f"quarantine target already exists: {target}")
        target.parent.mkdir(parents=True, exist_ok=True)
        source.rename(target)
        validate_file(target, row["size_bytes"], row["sha256"])
        row["applied"] = True
        row["applied_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
        write_json(plan_path, plan)
    plan["state"] = "applied"
    plan["completed_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(plan_path, plan)
    return len(plan["moves"])


def verify_storage(plan_path: Path, image_root: Path) -> int:
    plan = validate_plan(plan_path, image_root)
    if plan.get("state") != "applied":
        raise ImageStorageError(f"plan is not applied: {plan.get('state')}")
    for row in plan["moves"]:
        source = Path(row["source"])
        if source.exists() or source.is_symlink():
            raise ImageStorageError(f"quarantined source still exists: {source}")
        validate_file(Path(row["target"]), row["size_bytes"], row["sha256"])
    return len(plan["moves"])


def rollback_storage(plan_path: Path, image_root: Path) -> int:
    plan = validate_plan(plan_path, image_root)
    if plan.get("state") != "applied":
        raise ImageStorageError(f"plan cannot be rolled back from {plan.get('state')}")
    for row in reversed(plan["moves"]):
        source = Path(row["source"])
        target = Path(row["target"])
        validate_file(target, row["size_bytes"], row["sha256"])
        if source.exists() or source.is_symlink():
            raise ImageStorageError(f"rollback source already exists: {source}")
        source.parent.mkdir(parents=True, exist_ok=True)
        target.rename(source)
        row["applied"] = False
        row.pop("applied_at_utc", None)
        write_json(plan_path, plan)
    plan["state"] = "rolled_back"
    plan["rolled_back_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(plan_path, plan)
    return len(plan["moves"])


def study_relative_path(path: Path) -> str:
    """Return a stable locator below the governed study's containers root."""
    container_indexes = [
        index for index, part in enumerate(path.parts) if part == "containers"
    ]
    if not container_indexes:
        raise ImageStorageError(
            f"storage evidence is outside a containers root: {path}"
        )
    return Path(*path.parts[container_indexes[-1] :]).as_posix()


def storage_report(
    plan_path: Path,
    after_inventory_path: Path,
) -> tuple[dict[str, Any], str]:
    plan = load_json(plan_path)
    inventory = load_json(after_inventory_path)
    if plan.get("schema_version") != PLAN_SCHEMA:
        raise ImageStorageError("image storage plan schema is not supported")
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ImageStorageError("post-storage inventory schema is not supported")
    moves = plan.get("moves")
    if not isinstance(moves, list) or any(not isinstance(row, dict) for row in moves):
        raise ImageStorageError("image storage plan has invalid move records")
    sources = [str(row.get("source", "")) for row in moves]
    targets = [str(row.get("target", "")) for row in moves]
    if (
        any(not source for source in sources)
        or any(not target for target in targets)
        or len(sources) != len(set(sources))
        or len(targets) != len(set(targets))
    ):
        raise ImageStorageError("image storage plan has empty or duplicate paths")
    summary = inventory.get("summary")
    if not isinstance(summary, dict):
        raise ImageStorageError("post-storage inventory omits summary")
    states = summary.get("states")
    if not isinstance(states, dict):
        raise ImageStorageError("post-storage inventory omits state counts")
    tool_count = int(summary.get("tool_count", 0) or 0)
    current_or_unsupported = int(states.get("current", 0) or 0) + int(
        states.get("unsupported", 0) or 0
    )
    checks = {
        "plan_applied": plan.get("state") == "applied",
        "all_moves_applied": all(row.get("applied") is True for row in moves),
        "move_count_matches": plan.get("file_count") == len(moves),
        "move_size_matches": plan.get("size_bytes")
        == sum(int(row.get("size_bytes", 0) or 0) for row in moves),
        "only_current_or_unsupported_tools_remain": (
            tool_count > 0
            and current_or_unsupported == tool_count
            and set(states).issubset({"current", "unsupported"})
        ),
        "no_unknown_images": summary.get("unknown_image_count") == 0,
        "no_staged_images": summary.get("staged_image_count") == 0,
        "no_unreferenced_obsolete_images": (
            summary.get("unreferenced_obsolete_count") == 0
        ),
    }
    verdict = "passed" if all(checks.values()) else "failed"
    commit = str(inventory.get("repository_commit", ""))
    evidence_root = Path(str(inventory.get("evidence_root", "")))
    governed_inventory_root = evidence_root / "inventory" / commit
    rows = [
        {
            "category": str(row.get("category", "")),
            "name": Path(str(row.get("source", ""))).name,
            "sha256": str(row.get("sha256", "")),
            "size_bytes": int(row.get("size_bytes", 0) or 0),
            "quarantine_path": study_relative_path(Path(str(row.get("target", "")))),
        }
        for row in moves
    ]
    report = {
        "schema_version": "bijux.container.image_storage_report.v1",
        "repository_commit": commit,
        "verdict": verdict,
        "moved_file_count": len(rows),
        "moved_size_bytes": sum(row["size_bytes"] for row in rows),
        "post_storage_summary": summary,
        "checks": checks,
        "historical_dependencies_preserved": int(
            summary.get("historical_dependency_count", 0) or 0
        ),
        "recovery": {
            "mode": "same_filesystem_quarantine",
            "command": (
                "python3 -B containers/contracts/image_storage.py rollback "
                "--image-root <governed-containers-root>/apptainer "
                "--plan <governed-plan-path>"
            ),
        },
        "source_artifacts": {
            "storage_plan": {
                "study_relative_path": study_relative_path(
                    governed_inventory_root / plan_path.name
                ),
                "sha256": sha256_file(plan_path),
            },
            "post_storage_inventory": {
                "study_relative_path": study_relative_path(
                    governed_inventory_root / after_inventory_path.name
                ),
                "sha256": sha256_file(after_inventory_path),
            },
        },
        "moved_images": rows,
    }
    markdown = [
        "# LUNARC Apptainer canonical storage evidence",
        "",
        "Purpose: record the reversible removal of unreferenced obsolete SIFs from the active canonical image root.",
        "",
        "## Verdict",
        "",
        f"- Verdict: **{verdict}**",
        f"- Repository commit: `{commit}`",
        f"- Quarantined files: **{len(rows)}**",
        f"- Quarantined bytes: **{report['moved_size_bytes']}**",
        f"- Historical run dependencies preserved: **{report['historical_dependencies_preserved']}**",
        "",
        "No SIF was deleted. Every move remained on the governed filesystem and is recoverable through the checksummed source plan after target digests are revalidated.",
        "",
        "## Acceptance checks",
        "",
    ]
    markdown.extend(
        f"- {'PASS' if passed else 'FAIL'}: `{name}`" for name, passed in checks.items()
    )
    markdown.extend(
        [
            "",
            "## Quarantined images",
            "",
            "| Image | Category | Size (bytes) | SHA-256 | Quarantine path |",
            "|---|---|---:|---|---|",
        ]
    )
    for row in rows:
        markdown.append(
            f"| `{row['name']}` | `{row['category']}` | {row['size_bytes']} | `{row['sha256']}` | `{row['quarantine_path']}` |"
        )
    return report, "\n".join(markdown) + "\n"


def write_storage_report(
    plan_path: Path,
    after_inventory_path: Path,
    json_output: Path,
    markdown_output: Path,
    check: bool,
) -> None:
    report, markdown = storage_report(plan_path, after_inventory_path)
    expected_json = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if check:
        if (
            not json_output.is_file()
            or json_output.read_text(encoding="utf-8") != expected_json
        ):
            raise ImageStorageError(f"image storage JSON report drifted: {json_output}")
        if (
            not markdown_output.is_file()
            or markdown_output.read_text(encoding="utf-8") != markdown
        ):
            raise ImageStorageError(
                f"image storage Markdown report drifted: {markdown_output}"
            )
        return
    write_json(json_output, report)
    markdown_output.parent.mkdir(parents=True, exist_ok=True)
    markdown_output.write_text(markdown, encoding="utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "command", choices=("plan", "apply", "verify", "rollback", "report")
    )
    parser.add_argument("--inventory", type=Path)
    parser.add_argument("--after-inventory", type=Path)
    parser.add_argument("--image-root", type=Path)
    parser.add_argument("--quarantine-root", type=Path)
    parser.add_argument("--plan", type=Path, required=True)
    parser.add_argument("--json-output", type=Path)
    parser.add_argument("--markdown-output", type=Path)
    parser.add_argument("--check", action="store_true")
    args = parser.parse_args()
    try:
        if args.command == "report":
            if (
                args.after_inventory is None
                or args.json_output is None
                or args.markdown_output is None
            ):
                raise ImageStorageError(
                    "report requires after-inventory, JSON output, and Markdown output"
                )
            write_storage_report(
                args.plan,
                args.after_inventory,
                args.json_output,
                args.markdown_output,
                args.check,
            )
            print("report: OK" if args.check else "generated image storage report")
            return 0
        if args.image_root is None:
            raise ImageStorageError(f"{args.command} requires image root")
        if args.command == "plan":
            if args.inventory is None or args.quarantine_root is None:
                raise ImageStorageError("plan requires inventory and quarantine root")
            count = plan_storage(
                args.inventory, args.image_root, args.quarantine_root, args.plan
            )
        elif args.command == "apply":
            count = apply_storage(args.plan, args.image_root)
        elif args.command == "verify":
            count = verify_storage(args.plan, args.image_root)
        else:
            count = rollback_storage(args.plan, args.image_root)
        print(f"{args.command}: {count} image files")
        return 0
    except ImageStorageError as exc:
        print(f"container image storage failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
