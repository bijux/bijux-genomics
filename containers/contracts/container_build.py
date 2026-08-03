#!/usr/bin/env python3
"""Admit a frontend-built Apptainer image into governed immutable storage."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import socket
import subprocess
import sys
from pathlib import Path

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


def definition_file_sources(definition: bytes) -> list[Path]:
    sources: list[Path] = []
    in_files = False
    for raw_line in definition.decode("utf-8").splitlines():
        line = raw_line.strip()
        if line.startswith("%"):
            in_files = line == "%files"
            continue
        if not in_files or not line or line.startswith("#"):
            continue
        source = Path(line.split()[0])
        if source.is_absolute() or ".." in source.parts:
            raise ContractError(
                f"%files source must remain repository-relative: {source}"
            )
        sources.append(source)
    return sources


def committed_build_context(
    repository: Path,
    commit: str,
    definition: bytes,
) -> list[dict[str, str]]:
    context: list[dict[str, str]] = []
    for source in definition_file_sources(definition):
        object_id = git_output(repository, "rev-parse", f"{commit}:{source.as_posix()}")
        if not (repository / source).exists():
            raise ContractError(
                f"%files source is absent from the build worktree: {source}"
            )
        context.append({"path": source.as_posix(), "git_object": object_id})
    return context


def immutable_image_name(
    tool: str, version: str, architecture: str, digest: str
) -> str:
    return f"{tool}-{version}-{architecture.replace('/', '-')}-{digest[:12]}.sif"


def finalize_image(
    governance_path: Path,
    tool: str,
    version: str,
    repository_commit: str,
    definition_path: Path,
    image_path: Path,
    build_log_path: Path,
    already_immutable: bool = False,
    repository_path: Path | None = None,
) -> Path:
    governance = load_governance(governance_path)
    if TOOL_PATTERN.fullmatch(tool) is None:
        raise ContractError("tool must use canonical lowercase snake case")
    if VERSION_PATTERN.fullmatch(version) is None:
        raise ContractError("version contains unsupported filename characters")
    if COMMIT_PATTERN.fullmatch(repository_commit) is None:
        raise ContractError("repository commit must be a full Git identity")

    active_repository: Path = governance["active_repository"]
    repository = (repository_path or active_repository).resolve()
    if repository != active_repository:
        require_under(
            repository,
            governance["frozen_repositories"] / "image-builds",
            "frozen image-build repository",
        )
    if git_output(repository, "status", "--porcelain"):
        raise ContractError("build repository must be clean before image admission")
    git_output(repository, "cat-file", "-e", f"{repository_commit}^{{commit}}")
    build_commit = git_output(repository, "rev-parse", "HEAD")
    if build_commit != repository_commit:
        raise ContractError(
            "build repository HEAD must equal the declared build commit: "
            f"{build_commit} != {repository_commit}"
        )

    definition_path = definition_path.resolve()
    definition_root = repository / "containers" / "apptainer"
    require_under(definition_path, definition_root, "definition")
    expected_definition = definition_root / f"{tool}.def"
    if definition_path != expected_definition or not definition_path.is_file():
        raise ContractError(
            f"definition must be the canonical recipe: {expected_definition}"
        )
    definition_relative = definition_path.relative_to(repository)
    committed_definition = git_blob(repository, repository_commit, definition_relative)
    if definition_path.read_bytes() != committed_definition:
        raise ContractError("definition differs from the declared repository commit")
    build_context = committed_build_context(
        repository, repository_commit, committed_definition
    )

    image_root = governance["containers_root"] / "apptainer"
    staged_root = image_root / ".building"
    image_path = image_path.resolve()
    require_under(
        image_path,
        image_root if already_immutable else staged_root,
        "immutable image" if already_immutable else "staged image",
    )
    if not image_path.is_file() or image_path.suffix != ".sif":
        raise ContractError(f"candidate image is not a SIF file: {image_path}")

    evidence_root = governance["containers_root"] / "evidence"
    build_log_path = build_log_path.resolve()
    require_under(build_log_path, evidence_root / "build-logs", "build log")
    if not build_log_path.is_file() or build_log_path.stat().st_size == 0:
        raise ContractError(f"build log is absent or empty: {build_log_path}")
    build_log = build_log_path.read_text(encoding="utf-8", errors="replace")
    if "Build complete:" not in build_log:
        raise ContractError("build log does not record Apptainer completion")

    image_sha256 = sha256_file(image_path)
    architecture = governance["apptainer_architecture"]
    final_image_path = image_root / immutable_image_name(
        tool,
        version,
        architecture,
        image_sha256,
    )
    if already_immutable and image_path != final_image_path:
        raise ContractError(
            f"immutable image name does not match its content: {image_path}"
        )
    if not already_immutable and final_image_path.exists():
        raise ContractError(
            f"refusing to overwrite immutable image: {final_image_path}"
        )

    inspection = subprocess.run(
        ["apptainer", "inspect", "--json", str(image_path)],
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
    if build_evidence_path.exists():
        raise ContractError(
            f"refusing to overwrite image evidence: {build_evidence_path}"
        )
    if inspection_path.exists():
        try:
            recorded_inspection = json.loads(
                inspection_path.read_text(encoding="utf-8")
            )
        except json.JSONDecodeError as exc:
            raise ContractError(
                f"recorded Apptainer inspection is invalid: {exc}"
            ) from exc
        if recorded_inspection != inspection_record:
            raise ContractError("recorded inspection differs from the candidate image")

    if not already_immutable:
        image_path.replace(final_image_path)
    if not inspection_path.exists():
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
        "build_context": build_context,
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
    image = root.add_mutually_exclusive_group(required=True)
    image.add_argument("--staged-image", type=Path)
    image.add_argument("--immutable-image", type=Path)
    root.add_argument("--build-log", type=Path, required=True)
    root.add_argument("--repository", type=Path)
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
            args.staged_image or args.immutable_image,
            args.build_log,
            already_immutable=args.immutable_image is not None,
            repository_path=args.repository,
        )
        print(admitted)
        return 0
    except ContractError as exc:
        print(f"container image admission failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
