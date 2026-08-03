"""Shared validation primitives for replayable benchmark specifications."""

from __future__ import annotations

import hashlib
import json
import shlex
import subprocess
from collections.abc import Iterable
from pathlib import Path
from typing import Any

import tomllib
import yaml


class SpecificationError(RuntimeError):
    """Raised when governed evidence cannot support a benchmark specification."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SpecificationError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"JSON root must be an object: {path}")
    return value


def load_yaml(path: Path) -> dict[str, Any]:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        raise SpecificationError(f"cannot read YAML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"YAML root must be a mapping: {path}")
    return value


def load_toml(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise SpecificationError(f"cannot read TOML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"TOML root must be a mapping: {path}")
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def study_path(uri: str, study_root: Path) -> Path:
    value = Path(uri)
    if value.is_absolute():
        path = value.resolve()
    elif uri.startswith("study://"):
        path = (study_root / uri.removeprefix("study://")).resolve()
    else:
        raise SpecificationError(f"governed path is not study-relative: {uri}")
    try:
        path.relative_to(study_root.resolve())
    except ValueError as exc:
        raise SpecificationError(
            f"governed path escapes the study root: {uri}"
        ) from exc
    return path


def current_commit(repository: Path) -> str:
    status = subprocess.run(
        ["git", "-C", str(repository), "status", "--porcelain"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if status:
        raise SpecificationError("active repository must be clean")
    commit = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise SpecificationError("repository HEAD is not a full commit identity")
    return commit


def governed_roots(governance: dict[str, Any]) -> dict[str, Path | str]:
    if governance.get("schema_version") != "bijux.container.governance.v1":
        raise SpecificationError("governance schema is unsupported")
    paths = governance.get("paths")
    slurm = governance.get("slurm")
    if not isinstance(paths, dict) or not isinstance(slurm, dict):
        raise SpecificationError("governance paths and Slurm policy are required")
    study_root = Path(str(paths["study_root"])).resolve()
    roots: dict[str, Path | str] = {
        "study_root": study_root,
        "containers_root": Path(
            str(paths.get("containers_root", study_root / "containers"))
        ).resolve(),
        "data_root": Path(str(paths.get("data_root", study_root / "data"))).resolve(),
        "results_root": Path(
            str(paths.get("results_root", study_root / "results"))
        ).resolve(),
        "active_repository": Path(str(paths["active_repository"])).resolve(),
        "frozen_repositories": Path(str(paths["frozen_repositories"])).resolve(),
        "account": str(slurm["account"]),
        "partition": str(slurm["partitions"][0]),
    }
    for key in ("containers_root", "data_root", "results_root"):
        try:
            Path(str(roots[key])).relative_to(study_root)
        except ValueError as exc:
            raise SpecificationError(f"{key} escapes the study root") from exc
    return roots


def runtime_contracts(
    catalog: dict[str, Any], tool_ids: Iterable[str]
) -> dict[str, dict[str, Any]]:
    wanted = set(tool_ids)
    if catalog.get("schema_version") != "bijux.container.catalog.v1":
        raise SpecificationError("container catalog schema is unsupported")
    contracts: dict[str, dict[str, Any]] = {}
    for record in catalog.get("tools", []):
        if not isinstance(record, dict) or record.get("id") not in wanted:
            continue
        tool = str(record["id"])
        if record.get("support_disposition") != "retained":
            raise SpecificationError(f"{tool} is not retained")
        fields = ("version_command", "help_command", "expected_version_regex")
        if any(
            not isinstance(record.get(field), str) or not record[field]
            for field in fields
        ):
            raise SpecificationError(f"{tool} runtime contract is incomplete")
        contracts[tool] = {
            "version": shlex.split(str(record["version_command"])),
            "help": shlex.split(str(record["help_command"])),
            "version_regex": str(record["expected_version_regex"]),
        }
    if set(contracts) != wanted:
        raise SpecificationError("container catalog lacks the complete tool closure")
    return contracts


def selected_images(
    inventory: dict[str, Any], study_root: Path, tool_ids: Iterable[str]
) -> dict[str, dict[str, str]]:
    wanted = set(tool_ids)
    if inventory.get("schema_version") != "bijux.container.image_inventory.v1":
        raise SpecificationError("image inventory schema is unsupported")
    selected: dict[str, dict[str, str]] = {}
    for record in inventory.get("tools", []):
        if not isinstance(record, dict) or record.get("tool") not in wanted:
            continue
        tool = str(record["tool"])
        if (
            record.get("support_disposition") != "retained"
            or record.get("state") != "current"
        ):
            raise SpecificationError(f"{tool} lacks a current retained image")
        selected[tool] = {
            "path": str(study_path(str(record["selected_image"]), study_root)),
            "sha256": str(record["selected_image_sha256"]),
        }
    if set(selected) != wanted:
        raise SpecificationError("image inventory lacks the complete tool closure")
    return selected
