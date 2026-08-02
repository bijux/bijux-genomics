#!/usr/bin/env python3
"""Admit a frontend-built Apptainer image into governed immutable storage."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path
import re
import socket
import subprocess
import sys

from container_validation import (
    ContractError,
    git_output,
    load_governance,
    require_under,
    sha256_file,
    write_json,
)


TOOL_PATTERN = re.compile(r"[a-z0-9_]+")
VERSION_PATTERN = re.compile(r"[A-Za-z0-9][A-Za-z0-9._+-]*")
COMMIT_PATTERN = re.compile(r"[0-9a-f]{40}")


def git_blob(repository: Path, commit: str, relative_path: Path) -> bytes:
    completed = subprocess.run(
        ["git", "-C", str(repository), "show", f"{commit}:{relative_path.as_posix()}"],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise ContractError(
            f"cannot resolve {relative_path} at {commit}: "
            f"{completed.stderr.decode(errors='replace').strip()}"
        )
    return completed.stdout


def immutable_image_name(tool: str, version: str, architecture: str, digest: str) -> str:
    return f"{tool}-{version}-{architecture.replace('/', '-')}-{digest[:12]}.sif"


def finalize_image(
    governance_path: Path,
    tool: str,
    version: str,
    repository_commit: str,
    definition_path: Path,
    staged_image_path: Path,
    build_log_path: Path,
) -> Path:
    governance = load_governance(governance_path)
    if TOOL_PATTERN.fullmatch(tool) is None:
        raise ContractError("tool must use canonical lowercase snake case")
    if VERSION_PATTERN.fullmatch(version) is None:
        raise ContractError("version contains unsupported filename characters")
    if COMMIT_PATTERN.fullmatch(repository_commit) is None:
        raise ContractError("repository commit must be a full Git identity")

    repository: Path = governance["active_repository"]
    if git_output(repository, "status", "--porcelain"):
        raise ContractError("active repository must be clean before image admission")
    git_output(repository, "cat-file", "-e", f"{repository_commit}^{{commit}}")

    definition_path = definition_path.resolve()
    definition_root = repository / "containers" / "apptainer"
    require_under(definition_path, definition_root, "definition")
    expected_definition = definition_root / f"{tool}.def"
    if definition_path != expected_definition or not definition_path.is_file():
        raise ContractError(f"definition must be the canonical recipe: {expected_definition}")
    definition_relative = definition_path.relative_to(repository)
    committed_definition = git_blob(repository, repository_commit, definition_relative)
    if definition_path.read_bytes() != committed_definition:
        raise ContractError("definition differs from the declared repository commit")

    study_root: Path = governance["study_root"]
    image_root = study_root / "containers" / "apptainer"
    staged_root = image_root / ".building"
    staged_image_path = staged_image_path.resolve()
    require_under(staged_image_path, staged_root, "staged image")
    if not staged_image_path.is_file() or staged_image_path.suffix != ".sif":
        raise ContractError(f"staged image is not a SIF file: {staged_image_path}")

    evidence_root = study_root / "containers" / "evidence"
    build_log_path = build_log_path.resolve()
    require_under(build_log_path, evidence_root / "build-logs", "build log")
    if not build_log_path.is_file() or build_log_path.stat().st_size == 0:
        raise ContractError(f"build log is absent or empty: {build_log_path}")
    build_log = build_log_path.read_text(encoding="utf-8", errors="replace")
    if "Build complete:" not in build_log:
        raise ContractError("build log does not record Apptainer completion")

    image_sha256 = sha256_file(staged_image_path)
    architecture = governance["apptainer_architecture"]
    final_image_path = image_root / immutable_image_name(
        tool,
        version,
        architecture,
        image_sha256,
    )
    if final_image_path.exists():
        raise ContractError(f"refusing to overwrite immutable image: {final_image_path}")

    inspection = subprocess.run(
        ["apptainer", "inspect", "--json", str(staged_image_path)],
        check=False,
        capture_output=True,
        text=True,
    )
    if inspection.returncode != 0:
        raise ContractError(f"cannot inspect staged image: {inspection.stderr.strip()}")
    try:
        inspection_record = json.loads(inspection.stdout)
    except json.JSONDecodeError as exc:
        raise ContractError(f"Apptainer inspection is not valid JSON: {exc}") from exc

    definition_sha256 = hashlib.sha256(committed_definition).hexdigest()
    identity = f"{tool}-{version}-{image_sha256[:12]}"
    inspection_path = evidence_root / f"{identity}.inspect.json"
    build_evidence_path = evidence_root / f"{identity}.build.json"
    for evidence_path in (inspection_path, build_evidence_path):
        if evidence_path.exists():
            raise ContractError(f"refusing to overwrite image evidence: {evidence_path}")

    staged_image_path.replace(final_image_path)
    write_json(inspection_path, inspection_record)
    record = {
        "schema_version": "bijux.container.build_evidence.v1",
        "status": "passed",
        "tool": tool,
        "version": version,
        "site": {"id": governance["site_id"], "host": socket.gethostname()},
        "admitted_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "repository_commit": repository_commit,
        "governance_sha256": sha256_file(governance_path),
        "definition": {
            "path": definition_relative.as_posix(),
            "sha256": definition_sha256,
        },
        "build_log": {
            "path": str(build_log_path),
            "sha256": sha256_file(build_log_path),
        },
        "runtime": {
            "architecture": architecture,
            "apptainer_version": subprocess.run(
                ["apptainer", "--version"],
                check=False,
                capture_output=True,
                text=True,
            ).stdout.strip(),
        },
        "image": {
            "path": str(final_image_path),
            "sha256": image_sha256,
            "size_bytes": final_image_path.stat().st_size,
        },
        "inspection": {
            "path": str(inspection_path),
            "sha256": sha256_file(inspection_path),
        },
    }
    write_json(build_evidence_path, record)
    return final_image_path


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--governance", type=Path, required=True)
    root.add_argument("--tool", required=True)
    root.add_argument("--version", required=True)
    root.add_argument("--repository-commit", required=True)
    root.add_argument("--definition", type=Path, required=True)
    root.add_argument("--staged-image", type=Path, required=True)
    root.add_argument("--build-log", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        admitted = finalize_image(
            args.governance,
            args.tool,
            args.version,
            args.repository_commit,
            args.definition,
            args.staged_image,
            args.build_log,
        )
        print(admitted)
        return 0
    except ContractError as exc:
        print(f"container image admission failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
