#!/usr/bin/env python3
"""Resolve the complete repository-owned input identity of an Apptainer recipe."""

from __future__ import annotations

import subprocess
from pathlib import Path
from typing import Any


class RecipeIdentityError(RuntimeError):
    """Raised when a recipe or one of its repository inputs cannot be identified."""


def git_bytes(repository: Path, commit: str, relative_path: Path) -> bytes:
    completed = subprocess.run(
        ["git", "-C", str(repository), "show", f"{commit}:{relative_path.as_posix()}"],
        check=False,
        capture_output=True,
    )
    if completed.returncode != 0:
        raise RecipeIdentityError(
            f"cannot resolve {relative_path} at {commit}: "
            f"{completed.stderr.decode(errors='replace').strip()}"
        )
    return completed.stdout


def git_object(repository: Path, commit: str, relative_path: Path) -> str:
    completed = subprocess.run(
        [
            "git",
            "-C",
            str(repository),
            "rev-parse",
            f"{commit}:{relative_path.as_posix()}",
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    object_id = completed.stdout.strip()
    if completed.returncode != 0 or len(object_id) != 40:
        raise RecipeIdentityError(
            f"cannot identify {relative_path} at {commit}: {completed.stderr.strip()}"
        )
    return object_id


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
            raise RecipeIdentityError(
                f"%files source must remain repository-relative: {source}"
            )
        sources.append(source)
    return sources


def committed_build_context(
    repository: Path,
    commit: str,
    definition: bytes,
) -> list[dict[str, str]]:
    return [
        {
            "path": source.as_posix(),
            "git_object": git_object(repository, commit, source),
        }
        for source in definition_file_sources(definition)
    ]


def build_context_matches(
    repository: Path,
    commit: str,
    definition_path: Path,
    recorded_context: Any,
) -> bool:
    try:
        definition = git_bytes(repository, commit, definition_path)
        expected = committed_build_context(repository, commit, definition)
    except RecipeIdentityError:
        return False
    if recorded_context is None and not expected:
        recorded_context = []
    if not isinstance(recorded_context, list):
        return False
    return recorded_context == expected
