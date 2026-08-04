#!/usr/bin/env python3
"""Audit immutable Apptainer images against recipes, evidence, and real runs."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import subprocess
import sys
from collections import Counter
from pathlib import Path
from typing import Any

from recipe_identity import build_context_matches

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.9 on the governed LUNARC frontend
    import tomli as tomllib


INVENTORY_SCHEMA = "bijux.container.image_inventory.v1"
PUBLICATION_SCHEMA = "bijux.container.image_inventory_publication.v1"
SHA256 = re.compile(r"[0-9a-f]{64}")


class ImageInventoryError(RuntimeError):
    """Raised when image evidence is inconsistent or unsafe to classify."""


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
        raise ImageInventoryError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ImageInventoryError(f"JSON document must be an object: {path}")
    return value


def stable_path(value: str, roots: dict[str, Path]) -> str:
    if not value.startswith("/"):
        return value
    candidate = Path(value).resolve()
    for scheme, root in roots.items():
        try:
            relative = candidate.relative_to(root.resolve())
        except ValueError:
            continue
        return f"{scheme}://{relative.as_posix()}"
    raise ImageInventoryError(
        f"public evidence contains an absolute path outside governed roots: {value}"
    )


def stable_paths(value: Any, roots: dict[str, Path]) -> Any:
    if isinstance(value, dict):
        return {key: stable_paths(item, roots) for key, item in value.items()}
    if isinstance(value, list):
        return [stable_paths(item, roots) for item in value]
    if isinstance(value, str):
        return stable_path(value, roots)
    return value


def publish_inventory(
    inventory_path: Path,
    study_root: Path,
    repository: Path,
    output: Path,
) -> dict[str, Any]:
    if output.exists() or output.is_symlink():
        raise ImageInventoryError(f"refusing to replace published inventory: {output}")
    inventory = load_json(inventory_path)
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ImageInventoryError("only raw image inventories can be published")
    published = stable_paths(
        inventory,
        {"study": study_root, "repository": repository},
    )
    published["schema_version"] = PUBLICATION_SCHEMA
    published["publication"] = {
        "raw_schema_version": INVENTORY_SCHEMA,
        "raw_sha256": sha256_file(inventory_path),
        "path_schemes": {
            "study": "governed execution-site study root",
            "repository": "frozen repository root",
        },
    }
    write_json(output, published)
    return published


def load_catalog(path: Path) -> list[dict[str, Any]]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ImageInventoryError(
            f"cannot read container catalog {path}: {exc}"
        ) from exc
    tools = value.get("tools")
    if not isinstance(tools, list):
        raise ImageInventoryError("container catalog lacks a tools array")
    return tools


def git_blob_sha256(repository: Path, commit: str, relative: str) -> str | None:
    completed = subprocess.run(
        ["git", "-C", str(repository), "show", f"{commit}:{relative}"],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        return None
    return hashlib.sha256(completed.stdout).hexdigest()


def evidence_values(record: dict[str, Any]) -> tuple[str, int | None, str, str]:
    image = record.get("image")
    if isinstance(image, dict):
        image_digest = str(image.get("sha256", ""))
        image_size = image.get("size_bytes")
    else:
        image_digest = str(record.get("image_sha256", ""))
        image_size = record.get("image_size_bytes")
    definition = record.get("definition")
    if isinstance(definition, dict):
        definition_digest = str(definition.get("sha256", ""))
    else:
        definition_digest = str(record.get("definition_sha256", ""))
    return (
        image_digest,
        image_size if isinstance(image_size, int) else None,
        definition_digest,
        str(record.get("repository_commit", "")),
    )


def real_run_references(runs_root: Path) -> Counter[str]:
    references: Counter[str] = Counter()
    for path in runs_root.glob("run-*-*/evidence.lock.json"):
        record = load_json(path)
        image = record.get("image")
        if isinstance(image, dict):
            digest = image.get("sha256")
            if isinstance(digest, str) and SHA256.fullmatch(digest):
                references[digest] += 1
    return references


def build_evidence(evidence_root: Path) -> dict[str, list[tuple[Path, dict[str, Any]]]]:
    by_digest: dict[str, list[tuple[Path, dict[str, Any]]]] = {}
    for path in sorted(evidence_root.glob("*.build.json")):
        record = load_json(path)
        image_digest, _, _, _ = evidence_values(record)
        if SHA256.fullmatch(image_digest):
            by_digest.setdefault(image_digest, []).append((path, record))
    return by_digest


def candidate_record(
    image: Path,
    tool: dict[str, Any],
    evidence_by_digest: dict[str, list[tuple[Path, dict[str, Any]]]],
    references: Counter[str],
    repository: Path,
) -> dict[str, Any]:
    digest = sha256_file(image)
    expected_name = f"{tool['id']}-{tool['version']}-linux-amd64-{digest[:12]}.sif"
    current_definition = repository / str(tool["apptainer_definition"])
    current_definition_digest = sha256_file(current_definition)
    matching_evidence = []
    for evidence_path, record in evidence_by_digest.get(digest, []):
        evidence_digest, evidence_size, definition_digest, commit = evidence_values(
            record
        )
        recipe_at_commit = git_blob_sha256(
            repository, commit, str(tool["apptainer_definition"])
        )
        context_matches_commit = build_context_matches(
            repository,
            commit,
            Path(str(tool["apptainer_definition"])),
            record.get("build_context"),
        )
        context_matches_current = build_context_matches(
            repository,
            "HEAD",
            Path(str(tool["apptainer_definition"])),
            record.get("build_context"),
        )
        matching_evidence.append(
            {
                "path": str(evidence_path),
                "status": record.get("status"),
                "repository_commit": commit,
                "definition_sha256": definition_digest,
                "build_context": record.get("build_context"),
                "definition_matches_commit": recipe_at_commit == definition_digest,
                "definition_matches_current": (
                    definition_digest == current_definition_digest
                ),
                "build_context_matches_commit": context_matches_commit,
                "build_context_matches_current": context_matches_current,
                "image_matches": (
                    evidence_digest == digest
                    and evidence_size == image.stat().st_size
                    and record.get("tool") == tool["id"]
                    and str(record.get("version")) == str(tool["version"])
                ),
            }
        )
    admitted = any(
        evidence["image_matches"]
        and evidence["definition_matches_commit"]
        and evidence["build_context_matches_commit"]
        and evidence["status"] in {"built", "passed"}
        for evidence in matching_evidence
    )
    current_recipe = any(
        evidence["definition_matches_current"]
        and evidence["build_context_matches_current"]
        for evidence in matching_evidence
    )
    return {
        "path": str(image),
        "name": image.name,
        "sha256": digest,
        "size_bytes": image.stat().st_size,
        "immutable_name": image.name == expected_name,
        "admitted": admitted and image.name == expected_name,
        "current_recipe": current_recipe,
        "real_run_references": references[digest],
        "build_evidence": matching_evidence,
    }


def audit_images(
    catalog_path: Path,
    image_root: Path,
    evidence_root: Path,
    runs_root: Path,
    repository: Path,
    output: Path,
) -> dict[str, Any]:
    if output.exists() or output.is_symlink():
        raise ImageInventoryError(f"refusing to replace image inventory: {output}")
    repository_status = subprocess.run(
        ["git", "-C", str(repository), "status", "--porcelain"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if repository_status:
        raise ImageInventoryError(
            "repository must be clean before recording an image inventory"
        )
    tools = load_catalog(catalog_path)
    evidence_by_digest = build_evidence(evidence_root)
    references = real_run_references(runs_root)
    records = []
    known_images: set[Path] = set()
    for tool in tools:
        tool_id = str(tool["id"])
        version = str(tool["version"])
        supported = bool(tool.get("apptainer_supported"))
        candidates = []
        if supported:
            for image in sorted(
                image_root.glob(f"{tool_id}-{version}-linux-amd64-*.sif")
            ):
                known_images.add(image)
                candidates.append(
                    candidate_record(
                        image,
                        tool,
                        evidence_by_digest,
                        references,
                        repository,
                    )
                )
        valid = [candidate for candidate in candidates if candidate["admitted"]]
        current = [candidate for candidate in valid if candidate["current_recipe"]]
        selected = current[-1] if current else (valid[-1] if valid else None)
        if not supported:
            state = "unsupported"
        elif selected is None:
            state = "missing"
        elif selected["current_recipe"]:
            state = "current"
        else:
            state = "stale_recipe"
        selected_digest = selected["sha256"] if selected else None
        for candidate in candidates:
            if candidate["sha256"] == selected_digest:
                candidate["disposition"] = "canonical"
            elif candidate["real_run_references"]:
                candidate["disposition"] = "historical_run_dependency"
            else:
                candidate["disposition"] = "unreferenced_obsolete"
        records.append(
            {
                "tool": tool_id,
                "version": version,
                "support_disposition": tool.get("support_disposition"),
                "apptainer_supported": supported,
                "state": state,
                "selected_image_sha256": selected_digest,
                "selected_image": selected["path"] if selected else None,
                "candidates": candidates,
            }
        )
    catalog_ids = sorted(
        (str(tool["id"]) for tool in tools),
        key=len,
        reverse=True,
    )
    historical_images = []
    unknown_images = []
    for image in sorted(image_root.glob("*.sif")):
        if image not in known_images:
            digest = sha256_file(image)
            catalog_tool = next(
                (
                    tool_id
                    for tool_id in catalog_ids
                    if image.name.startswith(f"{tool_id}-")
                    and re.search(
                        rf"-linux-amd64-{digest[:12]}[.]sif$",
                        image.name,
                    )
                ),
                None,
            )
            reference_count = references[digest]
            record = {
                "path": str(image),
                "sha256": digest,
                "size_bytes": image.stat().st_size,
                "real_run_references": reference_count,
                "catalog_tool": catalog_tool,
            }
            if catalog_tool is not None and reference_count:
                record["disposition"] = "historical_run_dependency"
                historical_images.append(record)
            else:
                record["disposition"] = (
                    "unreferenced_obsolete"
                    if catalog_tool is not None
                    else "unclassified"
                )
                unknown_images.append(record)
    staged_images = [
        {
            "path": str(path),
            "sha256": sha256_file(path),
            "size_bytes": path.stat().st_size,
        }
        for path in sorted((image_root / ".building").glob("*.sif"))
    ]
    states = Counter(record["state"] for record in records)
    inventory = {
        "schema_version": INVENTORY_SCHEMA,
        "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "image_root": str(image_root.resolve()),
        "evidence_root": str(evidence_root.resolve()),
        "runs_root": str(runs_root.resolve()),
        "repository_commit": subprocess.run(
            ["git", "-C", str(repository), "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip(),
        "catalog_sha256": sha256_file(catalog_path),
        "summary": {
            "tool_count": len(records),
            "states": dict(sorted(states.items())),
            "historical_dependency_count": len(historical_images),
            "unknown_image_count": len(unknown_images),
            "staged_image_count": len(staged_images),
            "unreferenced_obsolete_count": sum(
                candidate["disposition"] == "unreferenced_obsolete"
                for record in records
                for candidate in record["candidates"]
            )
            + sum(
                image["disposition"] == "unreferenced_obsolete"
                for image in unknown_images
            ),
        },
        "tools": records,
        "historical_images": historical_images,
        "unknown_images": unknown_images,
        "staged_images": staged_images,
    }
    write_json(output, inventory)
    return inventory


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    audit = commands.add_parser("audit")
    audit.add_argument("--catalog", type=Path, required=True)
    audit.add_argument("--image-root", type=Path, required=True)
    audit.add_argument("--evidence-root", type=Path, required=True)
    audit.add_argument("--runs-root", type=Path, required=True)
    audit.add_argument("--repository", type=Path, required=True)
    audit.add_argument("--output", type=Path, required=True)
    publish = commands.add_parser("publish")
    publish.add_argument("--inventory", type=Path, required=True)
    publish.add_argument("--study-root", type=Path, required=True)
    publish.add_argument("--repository", type=Path, required=True)
    publish.add_argument("--output", type=Path, required=True)
    return root


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "audit":
            inventory = audit_images(
                args.catalog,
                args.image_root,
                args.evidence_root,
                args.runs_root,
                args.repository,
                args.output,
            )
            print(
                f"audited {inventory['summary']['tool_count']} catalog tools; "
                f"states={inventory['summary']['states']}"
            )
        elif args.command == "publish":
            inventory = publish_inventory(
                args.inventory,
                args.study_root,
                args.repository,
                args.output,
            )
            print(f"published {len(inventory['tools'])} catalog tools")
        return 0
    except (ImageInventoryError, subprocess.CalledProcessError) as exc:
        print(f"container image inventory failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
