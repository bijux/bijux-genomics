#!/usr/bin/env python3
"""Validate and execute immutable container checks on a Slurm worker."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import shlex
import socket
import subprocess
import sys
from pathlib import Path
from typing import Any

import yaml

PIPELINE_SCHEMA_V1 = "bijux.container.pipeline.v1"
PIPELINE_SCHEMA_V2 = "bijux.container.pipeline.v2"
PIPELINE_SCHEMAS = {PIPELINE_SCHEMA_V1, PIPELINE_SCHEMA_V2}
CREDENTIAL_SCHEMA = "bijux.container.credential_reference.v1"
GOVERNANCE_SCHEMA = "bijux.container.governance.v1"
EVIDENCE_SCHEMA_V1 = "bijux.container.evidence.v1"
EVIDENCE_SCHEMA_V2 = "bijux.container.evidence.v2"
RUN_ID_PATTERN = re.compile(r"run-[1-9][0-9]{3,}")
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}")
ACTIVE_SLURM_STATES = {
    "CONFIGURING",
    "COMPLETING",
    "PENDING",
    "REQUEUED",
    "RESIZING",
    "RUNNING",
    "SUSPENDED",
}
FROZEN_CHECKOUT_PATHS = ("containers/contracts",)


class ContractError(RuntimeError):
    """Raised when execution would violate a governed run contract."""


def require_keys(parent: dict[str, Any], required: set[str], label: str) -> None:
    missing = required.difference(parent)
    unknown = set(parent).difference(required)
    if missing:
        raise ContractError(f"{label} is missing required fields: {sorted(missing)}")
    if unknown:
        raise ContractError(f"{label} contains unknown fields: {sorted(unknown)}")


def load_yaml(path: Path) -> dict[str, Any]:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        raise ContractError(f"cannot read YAML contract {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ContractError(f"YAML contract must be a mapping: {path}")
    return value


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise ContractError(f"cannot read JSON contract {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise ContractError(f"JSON contract must be an object: {path}")
    return value


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def require_mapping(parent: dict[str, Any], key: str) -> dict[str, Any]:
    value = parent.get(key)
    if not isinstance(value, dict):
        raise ContractError(f"{key} must be a mapping")
    return value


def require_text(parent: dict[str, Any], key: str) -> str:
    value = parent.get(key)
    if not isinstance(value, str) or not value.strip():
        raise ContractError(f"{key} must be non-empty text")
    return value


def require_bool(parent: dict[str, Any], key: str) -> bool:
    value = parent.get(key)
    if not isinstance(value, bool):
        raise ContractError(f"{key} must be boolean")
    return value


def require_positive_integer(parent: dict[str, Any], key: str, label: str) -> int:
    value = parent.get(key)
    if not isinstance(value, int) or isinstance(value, bool) or value <= 0:
        raise ContractError(f"{label}.{key} must be a positive integer")
    return value


def require_utc_timestamp(parent: dict[str, Any], key: str, label: str) -> str:
    value = require_text(parent, key)
    try:
        parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise ContractError(f"{label}.{key} must be an ISO-8601 timestamp") from exc
    if parsed.tzinfo is None or parsed.utcoffset() != dt.timedelta(0):
        raise ContractError(f"{label}.{key} must use UTC")
    return value


def require_argv(commands: dict[str, Any], key: str) -> list[str]:
    value = commands.get(key)
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(token, str) or not token for token in value)
    ):
        raise ContractError(f"commands.{key} must be a non-empty argument vector")
    return value


def resolved(path: str) -> Path:
    return Path(path).expanduser().resolve()


def require_under(path: Path, root: Path, label: str) -> None:
    try:
        path.relative_to(root)
    except ValueError as exc:
        raise ContractError(f"{label} must remain under {root}: {path}") from exc


def validate_digest(value: str, label: str) -> None:
    if not SHA256_PATTERN.fullmatch(value) or len(set(value)) == 1:
        raise ContractError(f"{label} must be a resolved SHA-256 digest")


def git_output(repo: Path, *args: str) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repo), *args],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ContractError(f"git {' '.join(args)} failed: {completed.stderr.strip()}")
    return completed.stdout.strip()


def remove_incomplete_worktree(active_repository: Path, target: Path) -> bool:
    completed = subprocess.run(
        [
            "git",
            "-C",
            str(active_repository),
            "worktree",
            "remove",
            "--force",
            "--force",
            str(target),
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    return completed.returncode == 0 or not target.exists()


def validate_governance(config: dict[str, Any]) -> dict[str, Any]:
    require_keys(
        config,
        {"schema_version", "site", "paths", "runtime", "slurm"},
        "governance contract",
    )
    if config.get("schema_version") != GOVERNANCE_SCHEMA:
        raise ContractError(f"governance schema_version must be {GOVERNANCE_SCHEMA}")

    site = require_mapping(config, "site")
    require_keys(site, {"id", "hostname_regex"}, "governance site")
    require_text(site, "id")
    hostname_regex = require_text(site, "hostname_regex")
    try:
        hostname_pattern = re.compile(hostname_regex)
    except re.error as exc:
        raise ContractError(f"site.hostname_regex is invalid: {exc}") from exc
    hostname = socket.gethostname()
    if hostname_pattern.fullmatch(hostname) is None:
        raise ContractError(
            f"host {hostname!r} is outside governed site {site['id']!r}"
        )

    paths = require_mapping(config, "paths")
    require_keys(
        paths,
        {"study_root", "active_repository", "frozen_repositories"},
        "governance paths",
    )
    resolved_paths = {}
    for key in ("study_root", "active_repository", "frozen_repositories"):
        declared = Path(require_text(paths, key))
        if not declared.is_absolute():
            raise ContractError(f"paths.{key} must be absolute")
        resolved_paths[key] = declared.resolve()
    if resolved_paths["active_repository"] == resolved_paths["frozen_repositories"]:
        raise ContractError("active and frozen repository roots must differ")
    if not resolved_paths["study_root"].is_dir():
        raise ContractError(
            f"governed study root does not exist: {resolved_paths['study_root']}"
        )
    if not resolved_paths["active_repository"].is_dir():
        raise ContractError(
            f"active repository does not exist: {resolved_paths['active_repository']}"
        )

    runtime = require_mapping(config, "runtime")
    require_keys(runtime, {"apptainer_architecture"}, "governance runtime")
    architecture = require_text(runtime, "apptainer_architecture")
    if architecture not in {"linux/amd64", "linux/arm64"}:
        raise ContractError(f"unsupported Apptainer architecture: {architecture}")

    slurm = require_mapping(config, "slurm")
    require_keys(
        slurm,
        {"account", "partitions", "scratch_root"},
        "governance Slurm policy",
    )
    account = require_text(slurm, "account")
    scratch_root = Path(require_text(slurm, "scratch_root"))
    if not scratch_root.is_absolute() or not scratch_root.is_dir():
        raise ContractError("slurm.scratch_root must be an existing absolute directory")
    partitions = slurm.get("partitions")
    if (
        not isinstance(partitions, list)
        or not partitions
        or any(
            not isinstance(partition, str) or not partition for partition in partitions
        )
        or partitions != sorted(set(partitions))
    ):
        raise ContractError("slurm.partitions must be a sorted unique list")

    return {
        "site_id": site["id"],
        "hostname": hostname,
        "study_root": resolved_paths["study_root"],
        "active_repository": resolved_paths["active_repository"],
        "frozen_repositories": resolved_paths["frozen_repositories"],
        "apptainer_architecture": architecture,
        "slurm_account": account,
        "slurm_partitions": partitions,
        "slurm_scratch_root": scratch_root.resolve(),
    }


def load_governance(path: Path) -> dict[str, Any]:
    config = load_yaml(path)
    governance = validate_governance(config)
    governance["config"] = config
    governance["path"] = path.resolve()
    return governance


def freeze_repository(
    commit: str,
    governance_path: Path,
    checkout_scope: str = "contracts",
) -> Path:
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ContractError("frozen repository commit must be a full Git commit")
    if checkout_scope not in {"contracts", "complete"}:
        raise ContractError("frozen repository scope must be contracts or complete")
    governance = load_governance(governance_path)
    active_repository: Path = governance["active_repository"]
    frozen_repositories: Path = governance["frozen_repositories"]
    if git_output(active_repository, "status", "--porcelain"):
        raise ContractError(
            "active LUNARC repository must be clean before freezing a run"
        )
    git_output(active_repository, "cat-file", "-e", f"{commit}^{{commit}}")
    target = (
        frozen_repositories / commit
        if checkout_scope == "contracts"
        else frozen_repositories / "image-builds" / commit
    )
    if target.exists():
        if git_output(target, "rev-parse", "HEAD") != commit:
            raise ContractError(
                f"existing frozen worktree has the wrong commit: {target}"
            )
        if git_output(target, "status", "--porcelain"):
            raise ContractError(f"existing frozen worktree is dirty: {target}")
        return target
    target.parent.mkdir(parents=True, exist_ok=True)
    worktree_arguments = [
        "git",
        "-C",
        str(active_repository),
        "worktree",
        "add",
        "--detach",
    ]
    if checkout_scope == "contracts":
        worktree_arguments.append("--no-checkout")
    worktree_arguments.extend([str(target), commit])
    completed = subprocess.run(
        worktree_arguments,
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        cleanup_succeeded = remove_incomplete_worktree(active_repository, target)
        cleanup_note = ""
        if not cleanup_succeeded:
            cleanup_note = f"; incomplete worktree remains for inspection: {target}"
        raise ContractError(
            f"cannot create frozen worktree: {completed.stderr.strip()}{cleanup_note}"
        )
    checkout_commands = (
        (
            (
                ("sparse-checkout", "set", "--cone", *FROZEN_CHECKOUT_PATHS),
                "configure frozen sparse checkout",
            ),
            (("read-tree", "-mu", "HEAD"), "populate frozen sparse checkout"),
        )
        if checkout_scope == "contracts"
        else ()
    )
    for arguments, label in checkout_commands:
        configured = subprocess.run(
            ["git", "-C", str(target), *arguments],
            check=False,
            capture_output=True,
            text=True,
        )
        if configured.returncode != 0:
            cleanup_succeeded = remove_incomplete_worktree(active_repository, target)
            cleanup_note = ""
            if not cleanup_succeeded:
                cleanup_note = f"; incomplete worktree remains for inspection: {target}"
            raise ContractError(
                f"cannot {label}: {configured.stderr.strip()}{cleanup_note}"
            )
    try:
        if git_output(target, "rev-parse", "HEAD") != commit:
            raise ContractError("new frozen worktree has the wrong commit")
        if git_output(target, "status", "--porcelain"):
            raise ContractError("new frozen worktree is not clean")
    except ContractError as exc:
        remove_incomplete_worktree(active_repository, target)
        raise ContractError(f"cannot verify frozen worktree: {exc}") from exc
    return target


def validate_credentials(config: dict[str, Any]) -> None:
    require_keys(
        config,
        {"schema_version", "credentials_required", "references", "redaction"},
        "credential contract",
    )
    if config.get("schema_version") != CREDENTIAL_SCHEMA:
        raise ContractError(f"credential schema_version must be {CREDENTIAL_SCHEMA}")
    require_bool(config, "credentials_required")
    references = config.get("references")
    if not isinstance(references, list):
        raise ContractError("credential references must be a list")
    forbidden = {"secret", "token", "password", "value", "private_key"}
    for index, reference in enumerate(references):
        if not isinstance(reference, dict):
            raise ContractError(f"credential reference {index} must be a mapping")
        require_keys(
            reference,
            {"id", "purpose", "source_type", "variable_names", "failure_behavior"},
            f"credential reference {index}",
        )
        overlap = forbidden.intersection(reference)
        if overlap:
            raise ContractError(
                f"credential reference {index} embeds forbidden value fields: {sorted(overlap)}"
            )
        for key in ("id", "purpose", "source_type", "failure_behavior"):
            require_text(reference, key)
        variable_names = reference.get("variable_names")
        if not isinstance(variable_names, list) or any(
            not isinstance(name, str) or not name for name in variable_names
        ):
            raise ContractError(
                f"credential reference {index} variable_names must be a list"
            )
    if config["credentials_required"] != bool(references):
        raise ContractError(
            "credentials_required must agree with credential references"
        )
    redaction = config.get("redaction")
    if not isinstance(redaction, dict) or redaction.get("publish_values") is not False:
        raise ContractError(
            "credential redaction must explicitly forbid publishing values"
        )
    require_keys(redaction, {"publish_values"}, "credential redaction")


def validate_completed_dependencies(
    dependencies: list[dict[str, Any]],
    inputs: list[dict[str, Any]],
    study_root: Path,
) -> list[dict[str, Any]]:
    """Bind upstream results to passed immutable evidence before submission."""
    result_root = (study_root / "results" / "container-validation").resolve()
    records = []
    for dependency in dependencies:
        run_id = dependency["run_id"]
        required_digests = set(dependency["required_outputs"])
        matching_inputs = [
            item
            for item in inputs
            if item.get("kind") == "upstream_result"
            and item.get("sha256") in required_digests
        ]
        if {item["sha256"] for item in matching_inputs} != required_digests:
            raise ContractError(
                f"dependency {run_id} is not fully represented by upstream-result inputs"
            )
        run_directories = set()
        for item in matching_inputs:
            relative = resolved(item["path"]).relative_to(result_root)
            if not relative.parts or not relative.parts[0].startswith(f"{run_id}-"):
                raise ContractError(
                    f"dependency {run_id} input is outside its governed result directory"
                )
            run_directories.add(result_root / relative.parts[0])
        if len(run_directories) != 1:
            raise ContractError(
                f"dependency {run_id} inputs must share one governed result directory"
            )
        upstream_root = run_directories.pop()
        evidence_path = upstream_root / "evidence.json"
        evidence = load_json(evidence_path)
        if evidence.get("run_id") != run_id or evidence.get("status") != "passed":
            raise ContractError(f"dependency {run_id} lacks passing execution evidence")
        runtime = evidence.get("runtime")
        if not isinstance(runtime, dict) or runtime.get("slurm_job_id") != dependency[
            "slurm_job_id"
        ]:
            raise ContractError(
                f"dependency {run_id} Slurm identity disagrees with evidence"
            )
        output_digests = {
            output.get("sha256")
            for output in evidence.get("outputs", [])
            if isinstance(output, dict)
        }
        if not required_digests.issubset(output_digests):
            raise ContractError(f"dependency {run_id} output digest lacks evidence")
        records.append(
            {
                "run_id": run_id,
                "slurm_job_id": dependency["slurm_job_id"],
                "required_outputs": sorted(required_digests),
                "evidence_sha256": sha256_file(evidence_path),
            }
        )
    return records


def validate_pipeline(
    config: dict[str, Any],
    governance: dict[str, Any],
    governance_path: Path,
    pipeline_path: Path,
    credential_path: Path,
) -> dict[str, Any]:
    schema_version = config.get("schema_version")
    if schema_version not in PIPELINE_SCHEMAS:
        raise ContractError(
            f"pipeline schema_version must be one of {sorted(PIPELINE_SCHEMAS)}"
        )
    is_current = schema_version == PIPELINE_SCHEMA_V2
    top_level_fields = {
        "schema_version",
        "run",
        "repository",
        "image",
        "data",
        "commands",
        "outputs",
        "resources",
        "execution",
        "acceptance",
    }
    if is_current:
        top_level_fields.add("dependencies")
    require_keys(
        config,
        top_level_fields,
        "pipeline contract",
    )

    run = require_mapping(config, "run")
    run_fields = {
        "id",
        "campaign",
        "corpus",
        "sample",
        "domain",
        "stage",
        "tool",
        "operator",
    }
    if is_current:
        run_fields.add("submitted_at_utc")
    require_keys(
        run,
        run_fields,
        "run",
    )
    run_id = require_text(run, "id")
    if not RUN_ID_PATTERN.fullmatch(run_id):
        raise ContractError("run.id must use the run-1001 provenance sequence")
    for key in ("campaign", "corpus", "sample", "domain", "stage", "tool", "operator"):
        require_text(run, key)
    if is_current:
        require_utc_timestamp(run, "submitted_at_utc", "run")

    dependencies: list[dict[str, Any]] = []
    if is_current:
        declared_dependencies = config.get("dependencies")
        if not isinstance(declared_dependencies, list):
            raise ContractError("dependencies must be a list")
        dependency_run_ids: set[str] = set()
        for index, dependency in enumerate(declared_dependencies):
            if not isinstance(dependency, dict):
                raise ContractError(f"dependency {index} must be a mapping")
            require_keys(
                dependency,
                {"run_id", "slurm_job_id", "required_outputs"},
                f"dependency {index}",
            )
            dependency_run_id = require_text(dependency, "run_id")
            if not RUN_ID_PATTERN.fullmatch(dependency_run_id):
                raise ContractError(f"dependency {index} has an invalid run id")
            if dependency_run_id in dependency_run_ids:
                raise ContractError(
                    f"dependency run id is duplicated: {dependency_run_id}"
                )
            dependency_run_ids.add(dependency_run_id)
            job_id = require_text(dependency, "slurm_job_id")
            if not job_id.isdigit():
                raise ContractError(f"dependency {index} Slurm job id must be numeric")
            required_outputs = dependency.get("required_outputs")
            if not isinstance(required_outputs, list) or not required_outputs:
                raise ContractError(
                    f"dependency {index} required_outputs must be a non-empty list"
                )
            for output_digest in required_outputs:
                if not isinstance(output_digest, str):
                    raise ContractError(
                        f"dependency {index} output digests must be text"
                    )
                validate_digest(output_digest, f"dependency {index} output digest")
            dependencies.append(dependency)

    repository = require_mapping(config, "repository")
    require_keys(repository, {"path", "commit", "clean"}, "repository")
    repository_path = resolved(require_text(repository, "path"))
    expected_commit = require_text(repository, "commit")
    if not re.fullmatch(r"[0-9a-f]{40}", expected_commit):
        raise ContractError("repository.commit must be a full Git commit")
    active_repository: Path = governance["active_repository"]
    frozen_repositories: Path = governance["frozen_repositories"]
    study_root: Path = governance["study_root"]
    is_active_checkout = repository_path == active_repository
    is_frozen_checkout = (
        repository_path.parent == frozen_repositories
        and repository_path.name == expected_commit
    )
    if not (is_active_checkout or is_frozen_checkout):
        raise ContractError(
            "repository.path must identify the active checkout or its commit-addressed frozen worktree"
        )
    if require_bool(repository, "clean") is not True:
        raise ContractError("repository.clean must be true")
    if git_output(repository_path, "rev-parse", "HEAD") != expected_commit:
        raise ContractError("repository HEAD differs from frozen pipeline commit")
    if git_output(repository_path, "status", "--porcelain"):
        raise ContractError("repository worktree is not clean")

    image = require_mapping(config, "image")
    require_keys(image, {"path", "sha256"}, "image")
    image_path = resolved(require_text(image, "path"))
    require_under(image_path, study_root / "containers" / "apptainer", "image.path")
    if not image_path.is_file() or image_path.suffix != ".sif":
        raise ContractError(
            f"image.path must be an existing immutable SIF: {image_path}"
        )
    image_sha256 = require_text(image, "sha256")
    validate_digest(image_sha256, "image.sha256")
    if sha256_file(image_path) != image_sha256:
        raise ContractError("image digest differs from frozen pipeline digest")

    data = require_mapping(config, "data")
    require_keys(data, {"inputs"}, "data")
    inputs = data.get("inputs")
    if not isinstance(inputs, list) or not inputs:
        raise ContractError("data.inputs must contain at least one governed input")
    input_binds = []
    input_mount_paths: set[Path] = set()
    upstream_result_digests: set[str] = set()
    for index, item in enumerate(inputs):
        if not isinstance(item, dict):
            raise ContractError(f"data input {index} must be a mapping")
        allowed_input_keys = {"id", "kind", "path", "sha256"}
        if is_current:
            allowed_input_keys.add("mount_path")
        unknown_input_keys = set(item).difference(allowed_input_keys)
        required_input_keys = {"id", "path", "sha256"}
        if is_current:
            required_input_keys.add("mount_path")
        missing_input_keys = required_input_keys.difference(item)
        if missing_input_keys:
            raise ContractError(
                f"data input {index} is missing required fields: {sorted(missing_input_keys)}"
            )
        if unknown_input_keys:
            raise ContractError(
                f"data input {index} contains unknown fields: {sorted(unknown_input_keys)}"
            )
        input_id = require_text(item, "id")
        if is_current and re.fullmatch(r"[a-z0-9_]+", input_id) is None:
            raise ContractError(f"data input {index} id must use lowercase snake case")
        input_kind = item.get("kind", "data")
        if input_kind not in {"data", "reference", "database", "upstream_result"}:
            raise ContractError(
                f"data input {index} has unsupported kind: {input_kind}"
            )
        declared_input_path = Path(require_text(item, "path"))
        if not declared_input_path.is_absolute():
            raise ContractError(f"data input {index} path must be absolute")
        input_path = resolved(str(declared_input_path))
        input_root = (
            study_root / "results"
            if input_kind == "upstream_result"
            else study_root / "data"
        )
        require_under(input_path, input_root, f"data.inputs[{index}].path")
        if not input_path.is_file():
            raise ContractError(f"data input does not exist: {input_path}")
        digest = require_text(item, "sha256")
        validate_digest(digest, f"data.inputs[{index}].sha256")
        if sha256_file(input_path) != digest:
            raise ContractError(f"data input digest mismatch: {input_path}")
        if input_kind == "upstream_result":
            upstream_result_digests.add(digest)
        if is_current:
            mount_path = Path(require_text(item, "mount_path"))
            if (
                mount_path.parent != Path("/input")
                or re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", mount_path.name)
                is None
            ):
                raise ContractError(
                    f"data input {index} mount_path must be one safe filename under /input"
                )
            if mount_path in input_mount_paths:
                raise ContractError(
                    f"data input mount_path is duplicated: {mount_path}"
                )
            input_mount_paths.add(mount_path)
            input_binds.append((input_path, mount_path))
        else:
            input_binds.append((input_path, declared_input_path))
    if is_current:
        dependency_output_digests = {
            digest
            for dependency in config["dependencies"]
            for digest in dependency["required_outputs"]
        }
        if dependency_output_digests != upstream_result_digests:
            raise ContractError(
                "dependency output digests must exactly match upstream-result inputs"
            )
        dependencies = validate_completed_dependencies(
            dependencies,
            inputs,
            study_root,
        )

    commands = require_mapping(config, "commands")
    require_keys(commands, {"version", "help", "real", "negative"}, "commands")
    for key in ("version", "help", "real", "negative"):
        require_argv(commands, key)

    outputs = require_mapping(config, "outputs")
    output_fields = {"result_root", "required_files"}
    if is_current:
        output_fields.update({"collection_rule", "mount_path"})
    require_keys(outputs, output_fields, "outputs")
    declared_result_root = Path(require_text(outputs, "result_root"))
    if not declared_result_root.is_absolute():
        raise ContractError("outputs.result_root must be absolute")
    result_root = resolved(str(declared_result_root))
    require_under(
        result_root,
        study_root / "results" / "container-validation",
        "outputs.result_root",
    )
    expected_result_name = f"{run_id}-{run['tool']}" if is_current else run_id
    if result_root.name != expected_result_name:
        raise ContractError(
            f"outputs.result_root must end with {expected_result_name!r}"
        )
    result_mount_path = declared_result_root
    if is_current:
        result_mount_path = Path(require_text(outputs, "mount_path"))
        if result_mount_path != Path("/result"):
            raise ContractError("outputs.mount_path must equal /result")
    required_files = outputs.get("required_files")
    if not isinstance(required_files, list) or any(
        not isinstance(path, str) or not path for path in required_files
    ):
        raise ContractError("outputs.required_files must be a list of relative paths")
    for path in required_files:
        candidate = Path(path)
        if candidate.is_absolute() or ".." in candidate.parts:
            raise ContractError(f"required output must be a safe relative path: {path}")
    if is_current and outputs.get("collection_rule") != "required_files_and_metrics":
        raise ContractError(
            "outputs.collection_rule must be required_files_and_metrics"
        )

    resources = require_mapping(config, "resources")
    resource_fields = {
        "account",
        "partition",
        "cpus",
        "memory_mb",
        "walltime",
        "scratch_mb",
    }
    if is_current:
        resource_fields.add("retry")
    require_keys(
        resources,
        resource_fields,
        "resources",
    )
    for key in ("account", "partition", "walltime"):
        require_text(resources, key)
    if resources["account"] != governance["slurm_account"]:
        raise ContractError("resources.account is outside the governance contract")
    if resources["partition"] not in governance["slurm_partitions"]:
        raise ContractError("resources.partition is outside the governance contract")
    for key in ("cpus", "memory_mb", "scratch_mb"):
        require_positive_integer(resources, key, "resources")
    if is_current:
        retry = require_mapping(resources, "retry")
        require_keys(
            retry,
            {"max_attempts", "backoff_seconds", "retryable_exit_codes"},
            "resources.retry",
        )
        if retry.get("max_attempts") != 1:
            raise ContractError(
                "resources.retry.max_attempts must be 1 until governed resubmission exists"
            )
        backoff_seconds = retry.get("backoff_seconds")
        if (
            not isinstance(backoff_seconds, int)
            or isinstance(backoff_seconds, bool)
            or backoff_seconds < 0
        ):
            raise ContractError(
                "resources.retry.backoff_seconds must be a non-negative integer"
            )
        retryable_exit_codes = retry.get("retryable_exit_codes")
        if retryable_exit_codes != []:
            raise ContractError(
                "resources.retry.retryable_exit_codes must be empty when max_attempts is 1"
            )

    execution = require_mapping(config, "execution")
    execution_fields = {"locale", "timezone", "runtime_network"}
    if is_current:
        execution_fields.update(
            {"threads", "random_seed", "cache_policy", "determinism"}
        )
    require_keys(execution, execution_fields, "execution")
    if execution.get("locale") != "C" or execution.get("timezone") != "UTC":
        raise ContractError("execution must freeze locale C and timezone UTC")
    if execution.get("runtime_network") is not False:
        raise ContractError("container validation runs must disable runtime network")
    if is_current:
        threads = require_positive_integer(execution, "threads", "execution")
        if threads > resources["cpus"]:
            raise ContractError("execution.threads cannot exceed allocated CPUs")
        random_seed = execution.get("random_seed")
        if random_seed is not None and (
            not isinstance(random_seed, int) or isinstance(random_seed, bool)
        ):
            raise ContractError("execution.random_seed must be an integer or null")
        if execution.get("cache_policy") not in {"cold", "warm", "disabled"}:
            raise ContractError("execution.cache_policy has an unsupported value")
        determinism = execution.get("determinism")
        if determinism not in {"deterministic", "seeded", "nondeterministic"}:
            raise ContractError("execution.determinism has an unsupported value")
        if determinism == "seeded" and random_seed is None:
            raise ContractError("seeded execution requires execution.random_seed")

    acceptance = require_mapping(config, "acceptance")
    acceptance_fields = {"version_regex", "negative_exit_nonzero"}
    if is_current:
        acceptance_fields.add("metrics")
    require_keys(acceptance, acceptance_fields, "acceptance")
    try:
        re.compile(require_text(acceptance, "version_regex"))
    except re.error as exc:
        raise ContractError(f"acceptance.version_regex is invalid: {exc}") from exc
    if acceptance.get("negative_exit_nonzero") is not True:
        raise ContractError("acceptance must require a non-zero negative check")
    metrics: list[dict[str, Any]] = []
    if is_current:
        declared_metrics = acceptance.get("metrics")
        if not isinstance(declared_metrics, list) or not declared_metrics:
            raise ContractError("acceptance.metrics must be a non-empty list")
        metric_ids: set[str] = set()
        for index, metric in enumerate(declared_metrics):
            if not isinstance(metric, dict):
                raise ContractError(f"acceptance metric {index} must be a mapping")
            require_keys(
                metric,
                {"id", "path", "json_pointer", "operator", "expected", "unit"},
                f"acceptance metric {index}",
            )
            metric_id = require_text(metric, "id")
            if metric_id in metric_ids:
                raise ContractError(f"acceptance metric id is duplicated: {metric_id}")
            metric_ids.add(metric_id)
            metric_path = Path(require_text(metric, "path"))
            if metric_path.is_absolute() or ".." in metric_path.parts:
                raise ContractError(f"acceptance metric path is unsafe: {metric_path}")
            json_pointer = require_text(metric, "json_pointer")
            if not json_pointer.startswith("/"):
                raise ContractError("acceptance metric json_pointer must start with /")
            if metric.get("operator") not in {"eq", "ge", "gt", "le", "lt"}:
                raise ContractError("acceptance metric operator is unsupported")
            if not isinstance(metric.get("expected"), (int, float, str)) or isinstance(
                metric.get("expected"), bool
            ):
                raise ContractError("acceptance metric expected value must be scalar")
            require_text(metric, "unit")
            metrics.append(metric)

    contract_parent = pipeline_path.resolve().parent
    if credential_path.resolve().parent != contract_parent:
        raise ContractError(
            "pipeline and credential contracts must be archived together"
        )
    if is_current:
        if governance_path.resolve().parent != contract_parent:
            governance_root = active_repository / "containers" / "contracts" / "sites"
            require_under(
                governance_path.resolve(), governance_root, "governance contract"
            )
    elif governance_path.resolve().parent != contract_parent:
        raise ContractError(
            "historical governance, pipeline, and credential contracts must be archived together"
        )

    return {
        "run_id": run_id,
        "repository_path": repository_path,
        "repository_commit": expected_commit,
        "image_path": image_path,
        "image_sha256": image_sha256,
        "inputs": inputs,
        "input_binds": input_binds,
        "commands": commands,
        "result_root": result_root,
        "declared_result_root": declared_result_root,
        "result_mount_path": result_mount_path,
        "required_files": required_files,
        "resources": resources,
        "execution": execution,
        "acceptance": acceptance,
        "metrics": metrics,
        "dependencies": dependencies,
        "pipeline_schema": schema_version,
        "governance": governance,
    }


def validate_contracts(
    governance_path: Path, pipeline_path: Path, credential_path: Path
) -> dict[str, Any]:
    governance = load_governance(governance_path)
    credentials = load_yaml(credential_path)
    validate_credentials(credentials)
    pipeline = load_yaml(pipeline_path)
    resolved_contract = validate_pipeline(
        pipeline,
        governance,
        governance_path,
        pipeline_path,
        credential_path,
    )
    resolved_contract["pipeline"] = pipeline
    resolved_contract["credentials"] = credentials
    return resolved_contract


def run_apptainer(
    image_path: Path,
    argv: list[str],
    binds: list[tuple[Path, Path, str]],
    scratch_path: Path,
    stdout_path: Path,
    stderr_path: Path,
) -> int:
    command = [
        "apptainer",
        "exec",
        "--cleanenv",
        "--containall",
        "--net",
        "--network",
        "none",
    ]
    for source, target, mode in binds:
        command.extend(["--bind", f"{source}:{target}:{mode}"])
    command.extend([str(image_path), *argv])
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        completed = subprocess.run(
            command,
            check=False,
            stdout=stdout,
            stderr=stderr,
            env={
                "APPTAINERENV_TMPDIR": "/scratch",
                "LANG": "C",
                "LC_ALL": "C",
                "PATH": os.environ["PATH"],
                "TMPDIR": str(scratch_path),
                "TZ": "UTC",
            },
        )
    return completed.returncode


def write_json(path: Path, value: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".partial")
    temporary.write_text(
        json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    temporary.replace(path)


def json_pointer(value: Any, pointer: str) -> Any:
    current = value
    for encoded_token in pointer.removeprefix("/").split("/"):
        token = encoded_token.replace("~1", "/").replace("~0", "~")
        if isinstance(current, dict) and token in current:
            current = current[token]
        elif (
            isinstance(current, list) and token.isdigit() and int(token) < len(current)
        ):
            current = current[int(token)]
        else:
            raise ContractError(f"JSON pointer does not resolve: {pointer}")
    return current


def compare_metric(observed: Any, operator: str, expected: Any) -> bool:
    if operator == "eq":
        return observed == expected
    if (
        not isinstance(observed, (int, float))
        or isinstance(observed, bool)
        or not isinstance(expected, (int, float))
        or isinstance(expected, bool)
    ):
        raise ContractError(f"metric operator {operator} requires numeric values")
    return {
        "ge": observed >= expected,
        "gt": observed > expected,
        "le": observed <= expected,
        "lt": observed < expected,
    }[operator]


def evaluate_metrics(
    result_root: Path, metrics: list[dict[str, Any]]
) -> tuple[list[dict[str, Any]], list[str]]:
    records: list[dict[str, Any]] = []
    failures: list[str] = []
    for metric in metrics:
        source = result_root / metric["path"]
        try:
            payload = load_json_document(source)
            observed = json_pointer(payload, metric["json_pointer"])
            passed = compare_metric(observed, metric["operator"], metric["expected"])
        except ContractError as exc:
            observed = None
            passed = False
            failures.append(f"metric {metric['id']} cannot be evaluated: {exc}")
        if not passed and observed is not None:
            failures.append(
                f"metric {metric['id']} rejected {observed!r} "
                f"{metric['operator']} {metric['expected']!r}"
            )
        records.append(
            {
                "id": metric["id"],
                "path": metric["path"],
                "json_pointer": metric["json_pointer"],
                "operator": metric["operator"],
                "expected": metric["expected"],
                "observed": observed,
                "unit": metric["unit"],
                "status": "passed" if passed else "failed",
            }
        )
    return records, failures


def load_json_document(path: Path) -> Any:
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except OSError as exc:
        raise ContractError(f"metric source is unavailable: {path}: {exc}") from exc
    except json.JSONDecodeError as exc:
        raise ContractError(f"metric source is not valid JSON: {path}: {exc}") from exc


def execute(governance_path: Path, pipeline_path: Path, credential_path: Path) -> int:
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    started_at = dt.datetime.now(dt.timezone.utc)
    result_root: Path = contract["result_root"]
    if result_root.exists():
        raise ContractError(f"refusing to overwrite run result root: {result_root}")
    result_root.mkdir(parents=True, mode=0o750)
    (result_root / "config-governance.yml").write_bytes(governance_path.read_bytes())
    (result_root / "config-pipeline.yml").write_bytes(pipeline_path.read_bytes())
    (result_root / "config-credential.yml").write_bytes(credential_path.read_bytes())

    input_binds = [(source, target, "ro") for source, target in contract["input_binds"]]
    scratch_value = os.environ.get("TMPDIR")
    if not scratch_value:
        raise ContractError("TMPDIR is absent from the Slurm execution environment")
    scratch_path = Path(scratch_value).resolve()
    require_under(
        scratch_path,
        contract["governance"]["slurm_scratch_root"],
        "Slurm scratch",
    )
    if not scratch_path.is_dir():
        raise ContractError(f"Slurm scratch directory does not exist: {scratch_path}")
    binds = [
        *input_binds,
        (result_root, contract["result_mount_path"], "rw"),
        (scratch_path, Path("/scratch"), "rw"),
    ]
    checks: dict[str, dict[str, Any]] = {}
    for name in ("version", "help", "real", "negative"):
        stdout_path = result_root / f"{name}.stdout.log"
        stderr_path = result_root / f"{name}.stderr.log"
        exit_code = run_apptainer(
            contract["image_path"],
            contract["commands"][name],
            binds,
            scratch_path,
            stdout_path,
            stderr_path,
        )
        checks[name] = {
            "argv": contract["commands"][name],
            "exit_code": exit_code,
            "stdout_sha256": sha256_file(stdout_path),
            "stderr_sha256": sha256_file(stderr_path),
        }

    version_text = (result_root / "version.stdout.log").read_text(
        encoding="utf-8", errors="replace"
    ) + (result_root / "version.stderr.log").read_text(
        encoding="utf-8", errors="replace"
    )
    acceptance = contract["acceptance"]
    failures = []
    if checks["version"]["exit_code"] != 0:
        failures.append("version command failed")
    if re.search(acceptance["version_regex"], version_text, re.IGNORECASE) is None:
        failures.append("version output did not match the catalog contract")
    if checks["help"]["exit_code"] != 0:
        failures.append("help command failed")
    if checks["real"]["exit_code"] != 0:
        failures.append("real one-sample command failed")
    if checks["negative"]["exit_code"] == 0:
        failures.append("negative command unexpectedly succeeded")

    output_records = []
    for relative in contract["required_files"]:
        output = result_root / relative
        if not output.is_file() or output.stat().st_size == 0:
            failures.append(f"required output is absent or empty: {relative}")
            continue
        output_records.append(
            {
                "path": relative,
                "size_bytes": output.stat().st_size,
                "sha256": sha256_file(output),
            }
        )

    metric_records, metric_failures = evaluate_metrics(result_root, contract["metrics"])
    failures.extend(metric_failures)

    evidence = {
        "schema_version": (
            EVIDENCE_SCHEMA_V2
            if contract["pipeline_schema"] == PIPELINE_SCHEMA_V2
            else EVIDENCE_SCHEMA_V1
        ),
        "pipeline_schema_version": contract["pipeline_schema"],
        "run_id": contract["run_id"],
        "status": "passed" if not failures else "failed",
        "failure_class": None if not failures else "container_validation",
        "failures": failures,
        "repository_commit": contract["repository_commit"],
        "repository_path": str(contract["repository_path"]),
        "repository_clean": True,
        "governance_sha256": sha256_file(governance_path),
        "site": {
            "id": contract["governance"]["site_id"],
            "host": contract["governance"]["hostname"],
        },
        "pipeline_sha256": sha256_file(pipeline_path),
        "credential_reference_sha256": sha256_file(credential_path),
        "image": {
            "path": str(contract["image_path"]),
            "sha256": contract["image_sha256"],
            "size_bytes": contract["image_path"].stat().st_size,
        },
        "inputs": contract["inputs"],
        "dependencies": contract["dependencies"],
        "checks": checks,
        "outputs": output_records,
        "metrics": metric_records,
        "execution": contract["execution"],
        "runtime": {
            "host": socket.gethostname(),
            "slurm_job_id": os.environ.get("SLURM_JOB_ID", ""),
            "cpus": os.environ.get("SLURM_CPUS_PER_TASK", ""),
            "started_at_utc": started_at.isoformat(),
            "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "apptainer_version": subprocess.run(
                ["apptainer", "--version"], check=False, capture_output=True, text=True
            ).stdout.strip(),
            "scratch_path": str(scratch_path),
        },
    }
    write_json(result_root / "evidence.json", evidence)
    return 0 if not failures else 1


def render_slurm(
    governance_path: Path,
    pipeline_path: Path,
    credential_path: Path,
    output_path: Path,
) -> None:
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    resources = contract["resources"]
    run = contract["pipeline"]["run"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    job_name = (
        f"{run['domain']}-{run['stage'].split('.')[-1]}-{run['tool']}-{run['sample']}"
    )
    stdout_path = output_path.parent / "slurm-%j.stdout.log"
    stderr_path = output_path.parent / "slurm-%j.stderr.log"
    scratch_preflight = ""
    if contract["pipeline_schema"] == PIPELINE_SCHEMA_V2:
        required_scratch_kib = resources["scratch_mb"] * 1024
        scratch_root = contract["governance"]["slurm_scratch_root"]
        scratch_root_shell = shlex.quote(str(scratch_root))
        scratch_preflight = f"""if [[ ! "${{SLURM_JOB_ID:-}}" =~ ^[0-9]+$ ]]; then
  echo "Slurm job identity is unavailable" >&2
  exit 72
fi
readonly JOB_SCRATCH=$(mktemp -d {scratch_root_shell}/bijux-genomics-${{SLURM_JOB_ID}}-XXXXXX)
cleanup_scratch() {{
  if [[ "$JOB_SCRATCH" != {scratch_root_shell}/bijux-genomics-${{SLURM_JOB_ID}}-* ]]; then
    echo "refusing to remove unexpected scratch path: $JOB_SCRATCH" >&2
    return 73
  fi
  rm -rf -- "$JOB_SCRATCH"
}}
trap cleanup_scratch EXIT
export TMPDIR="$JOB_SCRATCH"
available_scratch_kib=$(df -Pk "$TMPDIR" | awk 'NR == 2 {{print $4}}')
if (( available_scratch_kib < {required_scratch_kib} )); then
  echo "insufficient Slurm scratch: ${{available_scratch_kib}} KiB" >&2
  exit 72
fi
"""
    script = f"""#!/usr/bin/env bash
#SBATCH --job-name={job_name}
#SBATCH --account={resources["account"]}
#SBATCH --partition={resources["partition"]}
#SBATCH --cpus-per-task={resources["cpus"]}
#SBATCH --mem={resources["memory_mb"]}M
#SBATCH --time={resources["walltime"]}
#SBATCH --output={stdout_path}
#SBATCH --error={stderr_path}
#SBATCH --no-requeue

set -euo pipefail
umask 027
export LANG=C
export LC_ALL=C
export TZ=UTC
{scratch_preflight}python3 {contract["repository_path"] / "containers/contracts/container_validation.py"} execute \\
  --governance {governance_path.resolve()} \\
  --pipeline {pipeline_path.resolve()} \\
  --credentials {credential_path.resolve()}
"""
    output_path.write_text(script, encoding="utf-8")
    output_path.chmod(0o750)


def parse_sbatch_job_id(value: str) -> str:
    job_id = value.strip().split(";", 1)[0]
    if not job_id.isdigit():
        raise ContractError(
            f"sbatch did not return a numeric job id: {value.strip()!r}"
        )
    return job_id


def operational_submission_root(
    manifest_path: Path,
    contract: dict[str, Any],
) -> Path:
    if contract["pipeline_schema"] == PIPELINE_SCHEMA_V1:
        return manifest_path
    return (
        contract["governance"]["study_root"]
        / "containers"
        / "evidence"
        / "submissions"
        / f"{contract['run_id']}-{contract['pipeline']['run']['tool']}"
    )


def require_current_manifest_path(
    manifest_path: Path,
    active_repository: Path,
    study_root: Path,
) -> None:
    allowed_roots = (
        active_repository / "containers" / "runs",
        study_root / "containers" / "evidence" / "specifications",
    )
    for root in allowed_roots:
        try:
            manifest_path.relative_to(root)
            return
        except ValueError:
            continue
    rendered_roots = ", ".join(str(root) for root in allowed_roots)
    raise ContractError(
        f"current manifest must remain under one of: {rendered_roots}: {manifest_path}"
    )


def manifest_contract_paths(
    manifest_path: Path,
    governance_override: Path | None,
) -> tuple[Path, Path, Path]:
    pipeline_path = manifest_path / "config-pipeline.yml"
    credential_path = manifest_path / "config-credential.yml"
    embedded_governance = manifest_path / "config-governance.yml"
    governance_path = governance_override or embedded_governance
    if governance_override is not None and embedded_governance.exists():
        raise ContractError(
            "governance must be embedded or supplied explicitly, not both"
        )
    return governance_path, pipeline_path, credential_path


def archived_contract_paths(
    governance_path: Path,
    pipeline_path: Path,
    credential_path: Path,
) -> tuple[tuple[Path, str], ...]:
    return (
        (governance_path, "config-governance.yml"),
        (pipeline_path, "config-pipeline.yml"),
        (credential_path, "config-credential.yml"),
    )


def submit_manifest(
    manifest_path: Path,
    governance_override: Path | None = None,
) -> str:
    manifest_path = manifest_path.resolve()
    governance_path, pipeline_path, credential_path = manifest_contract_paths(
        manifest_path, governance_override
    )
    governance = load_governance(governance_path)
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    if contract["pipeline_schema"] == PIPELINE_SCHEMA_V1:
        require_under(
            manifest_path,
            governance["study_root"] / "containers" / "manifests",
            "historical manifest",
        )
        expected_name = contract["run_id"]
    else:
        require_current_manifest_path(
            manifest_path,
            governance["active_repository"],
            governance["study_root"],
        )
        expected_name = f"{contract['run_id']}-{contract['pipeline']['run']['tool']}"
    if manifest_path.name != expected_name:
        raise ContractError(f"manifest directory must end with {expected_name!r}")
    submission_root = operational_submission_root(manifest_path, contract)
    if submission_root != manifest_path:
        if submission_root.exists():
            raise ContractError(
                f"refusing to overwrite an existing submission record: {submission_root}"
            )
        submission_root.mkdir(parents=True, mode=0o750)
        for source, archived_name in archived_contract_paths(
            governance_path,
            pipeline_path,
            credential_path,
        ):
            (submission_root / archived_name).write_bytes(source.read_bytes())
        governance_path = submission_root / "config-governance.yml"
        pipeline_path = submission_root / "config-pipeline.yml"
        credential_path = submission_root / "config-credential.yml"
    submit_path = submission_root / "submit.sh"
    job_id_path = submission_root / "slurm-job-id.txt"
    if submit_path.exists() or job_id_path.exists():
        raise ContractError("refusing to overwrite an existing submission record")
    render_slurm(governance_path, pipeline_path, credential_path, submit_path)
    syntax = subprocess.run(
        ["bash", "-n", str(submit_path)],
        check=False,
        capture_output=True,
        text=True,
    )
    if syntax.returncode != 0:
        submit_path.unlink()
        raise ContractError(
            f"rendered Slurm script is invalid: {syntax.stderr.strip()}"
        )
    submitted = subprocess.run(
        ["sbatch", "--parsable", str(submit_path)],
        check=False,
        capture_output=True,
        text=True,
    )
    if submitted.returncode != 0:
        write_json(
            submission_root / "submission-failure.json",
            {
                "schema_version": "bijux.slurm.submission_failure.v1",
                "run_id": contract["run_id"],
                "recorded_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
                "failure_class": "slurm_submission",
                "exit_code": submitted.returncode,
                "stdout": submitted.stdout,
                "stderr": submitted.stderr,
                "submit_script_sha256": sha256_file(submit_path),
            },
        )
        raise ContractError(f"Slurm submission failed: {submitted.stderr.strip()}")
    job_id = parse_sbatch_job_id(submitted.stdout)
    job_id_path.write_text(job_id + "\n", encoding="utf-8")
    return job_id


def collect_accounting(
    manifest_path: Path,
    governance_override: Path | None = None,
) -> Path:
    manifest_path = manifest_path.resolve()
    governance_path, pipeline_path, credential_path = manifest_contract_paths(
        manifest_path, governance_override
    )
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    submission_root = operational_submission_root(manifest_path, contract)
    job_id = (submission_root / "slurm-job-id.txt").read_text(encoding="utf-8").strip()
    if not job_id.isdigit():
        raise ContractError("manifest Slurm job id must be numeric")
    fields = [
        "JobID",
        "JobName",
        "State",
        "ExitCode",
        "Elapsed",
        "ElapsedRaw",
        "AllocCPUS",
        "ReqMem",
        "MaxRSS",
        "AveRSS",
        "TotalCPU",
        "CPUTimeRAW",
        "NodeList",
    ]
    completed = subprocess.run(
        ["sacct", "-X", "-n", "-P", "-j", job_id, "-o", ",".join(fields)],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ContractError(
            f"cannot collect accounting for {job_id}: {completed.stderr.strip()}"
        )
    rows = []
    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        values = line.split("|")
        if len(values) != len(fields):
            raise ContractError(f"unexpected sacct row for {job_id}: {line}")
        rows.append(dict(zip(fields, values)))
    if not rows:
        raise ContractError(f"Slurm accounting is unavailable for job {job_id}")
    active_states = {
        row["State"].split("+", 1)[0]
        for row in rows
        if row["State"].split("+", 1)[0] in ACTIVE_SLURM_STATES
    }
    if active_states:
        raise ContractError(
            f"Slurm job {job_id} is not terminal: {sorted(active_states)}"
        )
    result_root: Path = contract["result_root"]
    result_root_existed = result_root.is_dir()
    if result_root.exists() and not result_root_existed:
        raise ContractError(f"run result root is not a directory: {result_root}")
    if not result_root_existed:
        result_root.mkdir(parents=True, mode=0o750)
    accounting_path = result_root / "slurm-accounting.json"
    if accounting_path.exists():
        raise ContractError(
            f"refusing to overwrite accounting evidence: {accounting_path}"
        )
    submit_path = submission_root / "submit.sh"
    record = {
        "schema_version": "bijux.slurm.accounting.v1",
        "run_id": contract["run_id"],
        "slurm_job_id": job_id,
        "collected_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "rows": rows,
        "submit_script_sha256": sha256_file(submit_path),
        "governance_sha256": sha256_file(governance_path),
        "pipeline_sha256": sha256_file(pipeline_path),
        "credential_reference_sha256": sha256_file(credential_path),
    }
    (result_root / "submit.sh").write_bytes(submit_path.read_bytes())
    (result_root / "slurm-job-id.txt").write_text(job_id + "\n", encoding="utf-8")
    for source, archived_name in archived_contract_paths(
        governance_path,
        pipeline_path,
        credential_path,
    ):
        archived = result_root / archived_name
        if not archived.exists():
            archived.write_bytes(source.read_bytes())
    evidence_path = result_root / "evidence.json"
    if not evidence_path.exists():
        primary_row = next(
            (row for row in rows if row["JobID"] == job_id),
            rows[0],
        )
        scheduler_logs = []
        for stream in ("stdout", "stderr"):
            source = submission_root / f"slurm-{job_id}.{stream}.log"
            if not source.is_file():
                continue
            destination = result_root / source.name
            destination.write_bytes(source.read_bytes())
            scheduler_logs.append(
                {
                    "path": destination.name,
                    "size_bytes": destination.stat().st_size,
                    "sha256": sha256_file(destination),
                }
            )
        state = primary_row["State"].split("+", 1)[0]
        failure_class = (
            "execution_evidence_missing"
            if result_root_existed
            else "slurm_pre_execution"
        )
        failure = (
            f"Slurm job ended in {state} with exit code {primary_row['ExitCode']} "
            + (
                "without completing execution evidence"
                if result_root_existed
                else "before the executor created the run result"
            )
        )
        evidence = {
            "schema_version": (
                EVIDENCE_SCHEMA_V2
                if contract["pipeline_schema"] == PIPELINE_SCHEMA_V2
                else EVIDENCE_SCHEMA_V1
            ),
            "pipeline_schema_version": contract["pipeline_schema"],
            "run_id": contract["run_id"],
            "status": "failed",
            "failure_class": failure_class,
            "failures": [failure],
            "repository_commit": contract["repository_commit"],
            "repository_path": str(contract["repository_path"]),
            "repository_clean": True,
            "governance_sha256": sha256_file(governance_path),
            "site": {
                "id": contract["governance"]["site_id"],
                "host": contract["governance"]["hostname"],
            },
            "pipeline_sha256": sha256_file(pipeline_path),
            "credential_reference_sha256": sha256_file(credential_path),
            "image": {
                "path": str(contract["image_path"]),
                "sha256": contract["image_sha256"],
                "size_bytes": contract["image_path"].stat().st_size,
            },
            "inputs": contract["inputs"],
            "dependencies": contract["dependencies"],
            "checks": {},
            "outputs": [],
            "metrics": [],
            "execution": contract["execution"],
            "runtime": {
                "host": "",
                "slurm_job_id": job_id,
                "cpus": primary_row["AllocCPUS"],
                "started_at_utc": "",
                "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
                "apptainer_version": "",
                "scratch_path": "",
                "scheduler_state": state,
                "scheduler_exit_code": primary_row["ExitCode"],
            },
            "scheduler_logs": scheduler_logs,
        }
        write_json(evidence_path, evidence)
    write_json(accounting_path, record)
    return accounting_path


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    for name in ("validate", "execute"):
        command = commands.add_parser(name)
        command.add_argument("--governance", type=Path, required=True)
        command.add_argument("--pipeline", type=Path, required=True)
        command.add_argument("--credentials", type=Path, required=True)
    render = commands.add_parser("render-slurm")
    render.add_argument("--governance", type=Path, required=True)
    render.add_argument("--pipeline", type=Path, required=True)
    render.add_argument("--credentials", type=Path, required=True)
    render.add_argument("--output", type=Path, required=True)
    freeze = commands.add_parser("freeze-repository")
    freeze.add_argument("--governance", type=Path, required=True)
    freeze.add_argument("--commit", required=True)
    collect = commands.add_parser("collect-accounting")
    collect.add_argument("--manifest", type=Path, required=True)
    collect.add_argument("--governance", type=Path)
    submit = commands.add_parser("submit")
    submit.add_argument("--manifest", type=Path, required=True)
    submit.add_argument("--governance", type=Path)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "validate":
            validate_contracts(args.governance, args.pipeline, args.credentials)
            print("container validation contracts: OK")
            return 0
        if args.command == "execute":
            return execute(args.governance, args.pipeline, args.credentials)
        if args.command == "freeze-repository":
            print(freeze_repository(args.commit, args.governance))
            return 0
        if args.command == "collect-accounting":
            print(collect_accounting(args.manifest, args.governance))
            return 0
        if args.command == "submit":
            print(submit_manifest(args.manifest, args.governance))
            return 0
        render_slurm(args.governance, args.pipeline, args.credentials, args.output)
        print(args.output)
        return 0
    except ContractError as exc:
        print(f"container validation contract failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
