#!/usr/bin/env python3
"""Validate and execute immutable container checks on a Slurm worker."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
import re
import socket
import subprocess
import sys
import time
from pathlib import Path
from typing import Any

import yaml

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

PIPELINE_SCHEMA_V1 = "bijux.container.pipeline.v1"
PIPELINE_SCHEMA_V2 = "bijux.container.pipeline.v2"
ISOLATED_RUNTIME_HOSTNAME = "bijux-runtime"
PIPELINE_SCHEMAS = {PIPELINE_SCHEMA_V1, PIPELINE_SCHEMA_V2}
CREDENTIAL_SCHEMA = "bijux.container.credential_reference.v1"
GOVERNANCE_SCHEMA = "bijux.container.governance.v1"
EVIDENCE_SCHEMA_V1 = "bijux.container.evidence.v1"
EVIDENCE_SCHEMA_V2 = "bijux.container.evidence.v2"
PARAMETER_EVIDENCE_SCHEMA = "bijux.benchmark.parameter_evidence.v1"
EFFECTIVE_PARAMETERS_SCHEMA = "bijux.benchmark.effective_parameters.v1"
STAGE_INTERFACE_EVIDENCE_SCHEMA = "bijux.benchmark.stage_interface_evidence.v1"
STAGE_INTERFACE_CATALOG_SCHEMA = "bijux.benchmark.stage_interfaces.v1"
REPEATED_MEASUREMENT_SCHEMA = "bijux.benchmark.repeated_measurement.v1"
BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA = "bijux.benchmark.attempt_archive.v1"
RUN_ID_PATTERN = re.compile(r"run-[1-9][0-9]{3,}")
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}")


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


def validate_parameter_evidence(
    value: Any, pipeline_path: Path, stage_id: str
) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise ContractError("parameters must be a mapping")
    require_keys(
        value,
        {
            "schema_version",
            "parameter_schema_version",
            "defaults_provider",
            "overrides_file",
            "overrides_sha256",
            "effective_file",
            "effective_sha256",
        },
        "parameters",
    )
    if value.get("schema_version") != PARAMETER_EVIDENCE_SCHEMA:
        raise ContractError("parameters schema_version is unsupported")
    parameter_schema = require_text(value, "parameter_schema_version")
    defaults_provider = require_text(value, "defaults_provider")
    file_contracts = (
        ("overrides_file", "overrides_sha256", "config-pipeline-parameters.yml"),
        ("effective_file", "effective_sha256", "effective-parameters.json"),
    )
    paths: list[Path] = []
    for file_key, digest_key, expected_name in file_contracts:
        name = require_text(value, file_key)
        if name != expected_name:
            raise ContractError(f"parameters.{file_key} must equal {expected_name}")
        path = pipeline_path.parent / name
        if not path.is_file():
            raise ContractError(f"parameter evidence file is absent: {path}")
        digest = require_text(value, digest_key)
        validate_digest(digest, f"parameters.{digest_key}")
        if sha256_file(path) != digest:
            raise ContractError(f"parameter evidence digest mismatch: {name}")
        paths.append(path)

    override_config = load_yaml(paths[0])
    stages = override_config.get("stages")
    if not isinstance(stages, dict) or stage_id not in stages:
        raise ContractError(f"parameter override file lacks stage {stage_id}")
    stage_config = stages[stage_id]
    if not isinstance(stage_config, dict):
        raise ContractError(f"parameter override entry is invalid: {stage_id}")
    if (
        stage_config.get("parameter_schema_version") != parameter_schema
        or stage_config.get("defaults_provider") != defaults_provider
        or not isinstance(stage_config.get("overrides"), dict)
    ):
        raise ContractError("parameter override entry disagrees with pipeline evidence")

    effective = load_json(paths[1])
    if effective.get("schema_version") != EFFECTIVE_PARAMETERS_SCHEMA:
        raise ContractError("effective parameter schema is unsupported")
    if (
        effective.get("stage_id") != stage_id
        or effective.get("parameter_schema_version") != parameter_schema
        or effective.get("defaults_provider") != defaults_provider
        or effective.get("overrides") != stage_config["overrides"]
    ):
        raise ContractError(
            "effective parameters disagree with declared defaults or overrides"
        )
    resolved_by_sample = effective.get("resolved_by_sample")
    if not isinstance(resolved_by_sample, dict) or not resolved_by_sample:
        raise ContractError("effective parameters contain no resolved samples")
    for sample_id, resolved_parameters in resolved_by_sample.items():
        if (
            not isinstance(sample_id, str)
            or not sample_id
            or not isinstance(resolved_parameters, dict)
        ):
            raise ContractError("effective parameters contain an invalid sample entry")
        if resolved_parameters.get("schema_version") != parameter_schema:
            raise ContractError(
                f"effective parameters for {sample_id} use the wrong schema"
            )
    return {
        "files": paths,
        "evidence": {
            "schema_version": PARAMETER_EVIDENCE_SCHEMA,
            "parameter_schema_version": parameter_schema,
            "defaults_provider": defaults_provider,
            "overrides_sha256": value["overrides_sha256"],
            "effective_sha256": value["effective_sha256"],
            "resolved_sample_count": len(resolved_by_sample),
        },
    }


def load_stage_interface_catalog(repository_path: Path) -> list[dict[str, Any]]:
    catalog_path = repository_path / "benchmarks/contracts/stage-interfaces.toml"
    if not catalog_path.is_file():
        return []
    try:
        catalog = tomllib.loads(catalog_path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ContractError(f"cannot read stage interface catalog: {exc}") from exc
    if catalog.get("schema_version") != STAGE_INTERFACE_CATALOG_SCHEMA:
        raise ContractError("stage interface catalog schema is unsupported")
    interfaces = catalog.get("interfaces")
    if not isinstance(interfaces, list) or any(
        not isinstance(interface, dict) for interface in interfaces
    ):
        raise ContractError("stage interface catalog must contain interface mappings")
    return interfaces


def validate_stage_interface_evidence(
    value: Any,
    repository_path: Path,
    stage_id: str,
    tool_id: str,
) -> dict[str, Any] | None:
    matches = [
        interface
        for interface in load_stage_interface_catalog(repository_path)
        if interface.get("stage_id") == stage_id
    ]
    if not matches:
        if value is not None:
            raise ContractError(
                f"pipeline declares an interface absent from the catalog: {stage_id}"
            )
        return None
    if len(matches) != 1:
        raise ContractError(f"stage interface is not unique: {stage_id}")
    if not isinstance(value, dict):
        raise ContractError(
            f"pipeline stage {stage_id} requires governed interface evidence"
        )
    require_keys(
        value,
        {
            "schema_version",
            "accepted_input_contracts",
            "output_contract",
            "normalized_result_contract",
            "adapter",
            "downstream_probe",
        },
        "interface",
    )
    if value.get("schema_version") != STAGE_INTERFACE_EVIDENCE_SCHEMA:
        raise ContractError("interface schema is unsupported")
    expected = matches[0]
    accepted = value.get("accepted_input_contracts")
    if not isinstance(accepted, list) or accepted != expected.get(
        "accepted_input_contracts"
    ):
        raise ContractError("pipeline input contracts differ from the stage catalog")
    for key in ("output_contract", "normalized_result_contract"):
        if value.get(key) != expected.get(key):
            raise ContractError(f"pipeline {key} differs from the stage catalog")
    normalized_schema = repository_path / str(value["normalized_result_contract"])
    if not normalized_schema.is_file():
        raise ContractError("normalized result contract is absent from frozen source")
    adapters = expected.get("adapters")
    if not isinstance(adapters, dict) or value.get("adapter") != adapters.get(tool_id):
        raise ContractError("pipeline adapter differs from the stage catalog")
    declared_probe = value.get("downstream_probe")
    catalog_probe = expected.get("downstream_probe")
    if not isinstance(declared_probe, dict) or not isinstance(catalog_probe, dict):
        raise ContractError("stage downstream probe must be a mapping")
    require_keys(
        declared_probe,
        {"stage_id", "input_contract", "source", "artifact"},
        "interface downstream probe",
    )
    expected_probe = {
        "stage_id": catalog_probe.get("consumer_stage_id"),
        "input_contract": catalog_probe.get("input_contract"),
        "source": catalog_probe.get("source"),
    }
    if any(
        declared_probe.get(key) != expected_value
        for key, expected_value in expected_probe.items()
    ):
        raise ContractError("pipeline downstream probe differs from the stage catalog")
    artifact = Path(require_text(declared_probe, "artifact"))
    if artifact.is_absolute() or ".." in artifact.parts:
        raise ContractError("interface downstream probe artifact is unsafe")
    return value


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
    required_paths = {"study_root", "active_repository", "frozen_repositories"}
    allowed_paths = required_paths | {"containers_root", "data_root", "results_root"}
    missing_paths = required_paths.difference(paths)
    unknown_paths = set(paths).difference(allowed_paths)
    if missing_paths:
        raise ContractError(
            f"governance paths is missing required fields: {sorted(missing_paths)}"
        )
    if unknown_paths:
        raise ContractError(
            f"governance paths contains unknown fields: {sorted(unknown_paths)}"
        )
    resolved_paths = {}
    for key in required_paths:
        declared = Path(require_text(paths, key))
        if not declared.is_absolute():
            raise ContractError(f"paths.{key} must be absolute")
        resolved_paths[key] = declared.resolve()
    derived_roots = {
        "containers_root": resolved_paths["study_root"] / "containers",
        "data_root": resolved_paths["study_root"] / "data",
        "results_root": resolved_paths["study_root"] / "results",
    }
    for key, default in derived_roots.items():
        declared = Path(str(paths.get(key, default)))
        if not declared.is_absolute():
            raise ContractError(f"paths.{key} must be absolute")
        resolved_paths[key] = declared.resolve()
        require_under(resolved_paths[key], resolved_paths["study_root"], f"paths.{key}")
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
    required_slurm_fields = {"account", "partitions"}
    allowed_slurm_fields = {
        *required_slurm_fields,
        "scratch_environment",
        "scratch_root",
    }
    missing_slurm_fields = required_slurm_fields.difference(slurm)
    unknown_slurm_fields = set(slurm).difference(allowed_slurm_fields)
    if missing_slurm_fields:
        raise ContractError(
            "governance Slurm policy is missing required fields: "
            f"{sorted(missing_slurm_fields)}"
        )
    if unknown_slurm_fields:
        raise ContractError(
            "governance Slurm policy contains unknown fields: "
            f"{sorted(unknown_slurm_fields)}"
        )
    account = require_text(slurm, "account")
    scratch_fields = {"scratch_environment", "scratch_root"}.intersection(slurm)
    if len(scratch_fields) != 1:
        raise ContractError(
            "governance Slurm policy must declare exactly one of "
            "scratch_environment or scratch_root"
        )
    scratch_environment: str | None = None
    scratch_root: Path | None = None
    if "scratch_environment" in scratch_fields:
        scratch_environment = require_text(slurm, "scratch_environment")
        if re.fullmatch(r"[A-Z][A-Z0-9_]*", scratch_environment) is None:
            raise ContractError(
                "slurm.scratch_environment must be an uppercase environment name"
            )
    else:
        scratch_root = Path(require_text(slurm, "scratch_root"))
        if not scratch_root.is_absolute() or not scratch_root.is_dir():
            raise ContractError(
                "slurm.scratch_root must be an existing absolute directory"
            )
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
        "containers_root": resolved_paths["containers_root"],
        "data_root": resolved_paths["data_root"],
        "results_root": resolved_paths["results_root"],
        "active_repository": resolved_paths["active_repository"],
        "frozen_repositories": resolved_paths["frozen_repositories"],
        "apptainer_architecture": architecture,
        "slurm_account": account,
        "slurm_partitions": partitions,
        "slurm_scratch_environment": scratch_environment,
        "slurm_scratch_root": scratch_root.resolve() if scratch_root else None,
    }


def load_governance(path: Path) -> dict[str, Any]:
    config = load_yaml(path)
    governance = validate_governance(config)
    governance["config"] = config
    governance["path"] = path.resolve()
    return governance


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
    results_root: Path,
) -> list[dict[str, Any]]:
    """Bind upstream results to passed immutable evidence before submission."""
    result_root = (results_root / "container-validation").resolve()
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
        if (
            not isinstance(runtime, dict)
            or runtime.get("slurm_job_id") != dependency["slurm_job_id"]
        ):
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
        if "parameters" in config:
            top_level_fields.add("parameters")
        if "interface" in config:
            top_level_fields.add("interface")
        if "measurement" in config:
            top_level_fields.add("measurement")
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
    missing_run_fields = run_fields.difference(run)
    optional_run_fields = {"kind"} if is_current else set()
    unknown_run_fields = set(run).difference(run_fields | optional_run_fields)
    if missing_run_fields:
        raise ContractError(
            f"run is missing required fields: {sorted(missing_run_fields)}"
        )
    if unknown_run_fields:
        raise ContractError(
            f"run contains unknown fields: {sorted(unknown_run_fields)}"
        )
    run_id = require_text(run, "id")
    if not RUN_ID_PATTERN.fullmatch(run_id):
        raise ContractError("run.id must use the run-1001 provenance sequence")
    for key in ("campaign", "corpus", "sample", "domain", "stage", "tool", "operator"):
        require_text(run, key)
    if is_current:
        require_utc_timestamp(run, "submitted_at_utc", "run")
    evidence_kind = run.get("kind", "qualification") if is_current else "qualification"
    if evidence_kind not in {"qualification", "benchmark"}:
        raise ContractError("run.kind must be qualification or benchmark")
    if evidence_kind == "benchmark" and "parameters" not in config:
        raise ContractError(
            "benchmark pipelines must archive override and effective parameter evidence"
        )
    parameter_evidence = None
    if "parameters" in config:
        parameter_evidence = validate_parameter_evidence(
            config["parameters"], pipeline_path, str(run["stage"])
        )

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
    interface_evidence = validate_stage_interface_evidence(
        config.get("interface"),
        repository_path,
        str(run["stage"]),
        str(run["tool"]),
    )

    image = require_mapping(config, "image")
    require_keys(image, {"path", "sha256"}, "image")
    image_path = resolved(require_text(image, "path"))
    require_under(image_path, governance["containers_root"] / "apptainer", "image.path")
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
            governance["results_root"]
            if input_kind == "upstream_result"
            else governance["data_root"]
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
                or re.fullmatch(r"[A-Za-z0-9][A-Za-z0-9._-]*", mount_path.name) is None
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
            governance["results_root"],
        )

    measurement = config.get("measurement")
    if measurement is not None:
        if evidence_kind != "benchmark" or not isinstance(measurement, dict):
            raise ContractError("measurement is valid only for benchmark pipelines")
        require_keys(
            measurement,
            {
                "schema_version",
                "warmup_repetitions",
                "measured_repetitions",
                "resource_file_template",
                "runtime_scope",
            },
            "measurement",
        )
        if measurement.get("schema_version") != REPEATED_MEASUREMENT_SCHEMA:
            raise ContractError("measurement schema is unsupported")
        if measurement.get("warmup_repetitions") != 1:
            raise ContractError("measurement requires exactly one warmup repetition")
        measured_repetitions = measurement.get("measured_repetitions")
        if (
            not isinstance(measured_repetitions, int)
            or isinstance(measured_repetitions, bool)
            or not 1 <= measured_repetitions <= 10
        ):
            raise ContractError("measurement repetitions must be between 1 and 10")
        if measurement.get("resource_file_template") != (
            "repetitions/repetition-{repetition}.resource.tsv"
        ):
            raise ContractError("measurement resource path template is unsupported")
        if measurement.get("runtime_scope") != "containerized_stage_end_to_end":
            raise ContractError("measurement runtime scope is unsupported")

    commands = require_mapping(config, "commands")
    command_fields = {"version", "help", "real", "negative"}
    if measurement is not None:
        command_fields.add("warmup")
    require_keys(commands, command_fields, "commands")
    for key in command_fields:
        require_argv(commands, key)
    if measurement is not None and not any(
        "{repetition}" in token for token in commands["real"]
    ):
        raise ContractError("measured command must identify its repetition")

    outputs = require_mapping(config, "outputs")
    output_fields = {"result_root", "required_files"}
    if is_current:
        output_fields.update({"collection_rule", "mount_path"})
    require_keys(outputs, output_fields, "outputs")
    declared_result_root = Path(require_text(outputs, "result_root"))
    if not declared_result_root.is_absolute():
        raise ContractError("outputs.result_root must be absolute")
    result_root = resolved(str(declared_result_root))
    if is_current and evidence_kind == "benchmark":
        stage_slug = re.sub(r"[^a-z0-9]+", "-", str(run["stage"]).lower()).strip("-")
        expected_result_root = (
            governance["results_root"]
            / "benchmarks"
            / f"{run_id}-{stage_slug}"
            / str(run["tool"])
        )
        if result_root != expected_result_root:
            raise ContractError(
                f"benchmark outputs.result_root must equal {expected_result_root}"
            )
    else:
        qualification_root = governance["results_root"] / "container-validation"
        require_under(result_root, qualification_root, "outputs.result_root")
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
        "evidence_kind": evidence_kind,
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
        "parameters": parameter_evidence,
        "interface": interface_evidence,
        "measurement": measurement,
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
    command = apptainer_command(image_path, argv, binds)
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        completed = subprocess.run(
            command,
            check=False,
            stdout=stdout,
            stderr=stderr,
            env=apptainer_environment(scratch_path),
        )
    return completed.returncode


def apptainer_command(
    image_path: Path,
    argv: list[str],
    binds: list[tuple[Path, Path, str]],
) -> list[str]:
    command = [
        "apptainer",
        "exec",
        "--cleanenv",
        "--containall",
        "--hostname",
        ISOLATED_RUNTIME_HOSTNAME,
        "--net",
        "--network",
        "none",
    ]
    for source, target, mode in binds:
        command.extend(["--bind", f"{source}:{target}:{mode}"])
    command.extend([str(image_path), *argv])
    return command


def apptainer_environment(scratch_path: Path) -> dict[str, str]:
    return {
        "APPTAINERENV_TMPDIR": "/scratch",
        "LANG": "C",
        "LC_ALL": "C",
        "PATH": os.environ["PATH"],
        "TMPDIR": str(scratch_path),
        "TZ": "UTC",
    }


def run_apptainer_measured(
    image_path: Path,
    argv: list[str],
    binds: list[tuple[Path, Path, str]],
    scratch_path: Path,
    stdout_path: Path,
    stderr_path: Path,
    resource_path: Path,
) -> int:
    if not sys.platform.startswith("linux") or not hasattr(os, "wait4"):
        raise ContractError("measured container execution requires Linux wait4")
    command = apptainer_command(image_path, argv, binds)
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        started = time.monotonic()
        process = subprocess.Popen(
            command,
            stdout=stdout,
            stderr=stderr,
            env=apptainer_environment(scratch_path),
        )
        _, wait_status, usage = os.wait4(process.pid, 0)
        wall_seconds = time.monotonic() - started
        process.returncode = os.waitstatus_to_exitcode(wait_status)
    resource_path.write_text(
        "metric\tvalue\tunit\n"
        f"wall_seconds\t{wall_seconds:.9f}\tseconds\n"
        f"user_seconds\t{usage.ru_utime:.9f}\tseconds\n"
        f"system_seconds\t{usage.ru_stime:.9f}\tseconds\n"
        f"peak_rss_kib\t{usage.ru_maxrss}\tkibibytes\n"
        f"filesystem_input_operations\t{usage.ru_inblock}\toperations\n"
        f"filesystem_output_operations\t{usage.ru_oublock}\toperations\n",
        encoding="utf-8",
    )
    return process.returncode


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


def tool_output_root(contract: dict[str, Any]) -> Path:
    """Keep current tool writes below immutable run-governance evidence."""
    result_root: Path = contract["result_root"]
    if contract["pipeline_schema"] == PIPELINE_SCHEMA_V2:
        return result_root / "outputs"
    return result_root


def governed_scratch_root(governance: dict[str, Any]) -> Path:
    environment = governance["slurm_scratch_environment"]
    if environment is not None:
        value = os.environ.get(environment)
        if not value:
            raise ContractError(
                f"{environment} is absent from the Slurm execution environment"
            )
        root = Path(value)
    else:
        root = governance["slurm_scratch_root"]
    if not isinstance(root, Path) or not root.is_absolute() or not root.is_dir():
        raise ContractError("governed Slurm scratch must be an existing absolute path")
    return root.resolve()


def execute(governance_path: Path, pipeline_path: Path, credential_path: Path) -> int:
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    started_at = dt.datetime.now(dt.timezone.utc)
    result_root: Path = contract["result_root"]
    if result_root.exists():
        raise ContractError(f"refusing to overwrite run result root: {result_root}")
    result_root.mkdir(parents=True, mode=0o750)
    output_root = tool_output_root(contract)
    output_root.mkdir(mode=0o750)
    (result_root / "config-governance.yml").write_bytes(governance_path.read_bytes())
    (result_root / "config-pipeline.yml").write_bytes(pipeline_path.read_bytes())
    (result_root / "config-credential.yml").write_bytes(credential_path.read_bytes())
    if contract["parameters"] is not None:
        for parameter_path in contract["parameters"]["files"]:
            (result_root / parameter_path.name).write_bytes(parameter_path.read_bytes())

    input_binds = [(source, target, "ro") for source, target in contract["input_binds"]]
    scratch_value = os.environ.get("TMPDIR")
    if not scratch_value:
        raise ContractError("TMPDIR is absent from the Slurm execution environment")
    scratch_path = Path(scratch_value).resolve()
    scratch_root = governed_scratch_root(contract["governance"])
    require_under(
        scratch_path,
        scratch_root,
        "Slurm scratch",
    )
    if not scratch_path.is_dir():
        raise ContractError(f"Slurm scratch directory does not exist: {scratch_path}")
    binds = [
        *input_binds,
        (contract["repository_path"], Path("/repository"), "ro"),
        (output_root, contract["result_mount_path"], "rw"),
        (scratch_path, Path("/scratch"), "rw"),
    ]
    checks: dict[str, dict[str, Any]] = {}
    for name in ("version", "help"):
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

    measurement = contract["measurement"]
    if measurement is None:
        stdout_path = result_root / "real.stdout.log"
        stderr_path = result_root / "real.stderr.log"
        exit_code = run_apptainer(
            contract["image_path"],
            contract["commands"]["real"],
            binds,
            scratch_path,
            stdout_path,
            stderr_path,
        )
        checks["real"] = {
            "argv": contract["commands"]["real"],
            "exit_code": exit_code,
            "stdout_sha256": sha256_file(stdout_path),
            "stderr_sha256": sha256_file(stderr_path),
        }
    else:
        warmup_stdout = result_root / "warmup.stdout.log"
        warmup_stderr = result_root / "warmup.stderr.log"
        warmup_exit = run_apptainer(
            contract["image_path"],
            contract["commands"]["warmup"],
            binds,
            scratch_path,
            warmup_stdout,
            warmup_stderr,
        )
        checks["warmup"] = {
            "argv": contract["commands"]["warmup"],
            "exit_code": warmup_exit,
            "stdout_sha256": sha256_file(warmup_stdout),
            "stderr_sha256": sha256_file(warmup_stderr),
        }
        repetitions = []
        for repetition in range(1, measurement["measured_repetitions"] + 1):
            argv = [
                token.replace("{repetition}", str(repetition))
                for token in contract["commands"]["real"]
            ]
            stdout_path = result_root / f"real.repetition-{repetition}.stdout.log"
            stderr_path = result_root / f"real.repetition-{repetition}.stderr.log"
            resource_relative = measurement["resource_file_template"].replace(
                "{repetition}", str(repetition)
            )
            resource_path = output_root / resource_relative
            resource_path.parent.mkdir(parents=True, exist_ok=True)
            exit_code = run_apptainer_measured(
                contract["image_path"],
                argv,
                binds,
                scratch_path,
                stdout_path,
                stderr_path,
                resource_path,
            )
            repetitions.append(
                {
                    "repetition": repetition,
                    "argv": argv,
                    "exit_code": exit_code,
                    "stdout_sha256": sha256_file(stdout_path),
                    "stderr_sha256": sha256_file(stderr_path),
                    "resource_path": resource_relative,
                    "resource_sha256": sha256_file(resource_path),
                }
            )
        checks["real"] = {
            "argv_template": contract["commands"]["real"],
            "exit_code": max(record["exit_code"] for record in repetitions),
            "repetitions": repetitions,
            "runtime_scope": measurement["runtime_scope"],
        }

    negative_stdout = result_root / "negative.stdout.log"
    negative_stderr = result_root / "negative.stderr.log"
    negative_exit = run_apptainer(
        contract["image_path"],
        contract["commands"]["negative"],
        binds,
        scratch_path,
        negative_stdout,
        negative_stderr,
    )
    checks["negative"] = {
        "argv": contract["commands"]["negative"],
        "exit_code": negative_exit,
        "stdout_sha256": sha256_file(negative_stdout),
        "stderr_sha256": sha256_file(negative_stderr),
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
    if "warmup" in checks and checks["warmup"]["exit_code"] != 0:
        failures.append("warmup command failed")
    if checks["real"]["exit_code"] != 0:
        failures.append("real command failed")
    if checks["negative"]["exit_code"] == 0:
        failures.append("negative command unexpectedly succeeded")

    output_records = []
    for relative in contract["required_files"]:
        output = output_root / relative
        if not output.is_file() or output.stat().st_size == 0:
            failures.append(f"required output is absent or empty: {relative}")
            continue
        evidence_path = output.relative_to(result_root).as_posix()
        output_records.append(
            {
                "path": evidence_path,
                "size_bytes": output.stat().st_size,
                "sha256": sha256_file(output),
            }
        )

    metric_records, metric_failures = evaluate_metrics(output_root, contract["metrics"])
    failures.extend(metric_failures)

    evidence = {
        "schema_version": (
            EVIDENCE_SCHEMA_V2
            if contract["pipeline_schema"] == PIPELINE_SCHEMA_V2
            else EVIDENCE_SCHEMA_V1
        ),
        "pipeline_schema_version": contract["pipeline_schema"],
        "run_id": contract["run_id"],
        "evidence_kind": contract["evidence_kind"],
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
        "parameters": (
            contract["parameters"]["evidence"]
            if contract["parameters"] is not None
            else None
        ),
        "interface": contract["interface"],
        "measurement": contract["measurement"],
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


def operational_submission_root(
    manifest_path: Path,
    contract: dict[str, Any],
) -> Path:
    if contract["pipeline_schema"] == PIPELINE_SCHEMA_V1:
        return manifest_path
    return (
        contract["governance"]["containers_root"]
        / "evidence"
        / "submissions"
        / f"{contract['run_id']}-{contract['pipeline']['run']['tool']}"
    )


def require_current_manifest_path(
    manifest_path: Path,
    active_repository: Path,
    containers_root: Path,
    results_root: Path | None = None,
) -> None:
    allowed_roots = [
        active_repository / "containers" / "runs",
        containers_root / "runs",
        containers_root / "evidence" / "specifications",
    ]
    if results_root is not None:
        allowed_roots.append(results_root / "benchmark-specifications")
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
    parameter_paths: list[Path] | None = None,
) -> tuple[tuple[Path, str], ...]:
    core = (
        (governance_path, "config-governance.yml"),
        (pipeline_path, "config-pipeline.yml"),
        (credential_path, "config-credential.yml"),
    )
    parameters = tuple((path, path.name) for path in (parameter_paths or []))
    return core + parameters


def archive_file_records(root: Path, scope: str) -> list[dict[str, Any]]:
    records = []
    for path in sorted(root.rglob("*")):
        if path.is_symlink():
            raise ContractError(f"archived evidence must not contain symlinks: {path}")
        if not path.is_file():
            continue
        records.append(
            {
                "path": f"{scope}/{path.relative_to(root).as_posix()}",
                "size_bytes": path.stat().st_size,
                "sha256": sha256_file(path),
            }
        )
    return records


def archive_terminal_run(
    manifest_path: Path,
    governance_override: Path | None = None,
    *,
    expected_status: str,
    disposition: str,
    reason: str | None = None,
) -> Path:
    manifest_path = manifest_path.resolve()
    governance_path, pipeline_path, credential_path = manifest_contract_paths(
        manifest_path, governance_override
    )
    contract = validate_contracts(governance_path, pipeline_path, credential_path)
    run_kind = str(contract["pipeline"]["run"].get("kind", "qualification"))
    archive_scope = (
        disposition
        if run_kind == "benchmark"
        else ("failures" if disposition == "failed" else disposition)
    )
    submission_root = operational_submission_root(manifest_path, contract)
    result_root: Path = contract["result_root"]
    if not submission_root.is_dir() or not result_root.is_dir():
        raise ContractError(
            f"{archive_scope} archival requires both submission and result evidence"
        )
    job_id_path = submission_root / "slurm-job-id.txt"
    try:
        job_id = job_id_path.read_text(encoding="utf-8").strip()
    except OSError as exc:
        raise ContractError(f"cannot read terminal Slurm job identity: {exc}") from exc
    if not job_id.isdigit():
        raise ContractError("terminal Slurm job identity must be numeric")
    evidence_path = result_root / "evidence.json"
    accounting_path = result_root / "slurm-accounting.json"
    evidence = load_json(evidence_path)
    accounting = load_json(accounting_path)
    if (
        evidence.get("schema_version") not in {EVIDENCE_SCHEMA_V1, EVIDENCE_SCHEMA_V2}
        or evidence.get("run_id") != contract["run_id"]
        or evidence.get("status") != expected_status
    ):
        raise ContractError(
            f"only terminal {expected_status} run evidence can be archived as "
            f"{archive_scope}"
        )
    if (
        accounting.get("schema_version") != "bijux.slurm.accounting.v1"
        or accounting.get("run_id") != contract["run_id"]
        or accounting.get("slurm_job_id") != job_id
    ):
        raise ContractError("Slurm accounting does not identify the terminal run")
    tool_id = str(contract["pipeline"]["run"]["tool"])
    bundle_name = f"{contract['run_id']}-{tool_id}-slurm-{job_id}"
    if run_kind == "benchmark":
        archive_root = (
            contract["governance"]["results_root"]
            / "benchmark-attempts"
            / archive_scope
        )
    else:
        archive_root = (
            contract["governance"]["containers_root"] / "evidence" / archive_scope
        )
    archive_root.mkdir(parents=True, mode=0o750, exist_ok=True)
    target = archive_root / bundle_name
    staging = archive_root / f".{bundle_name}.partial"
    if target.exists() or staging.exists():
        raise ContractError(f"refusing to replace {archive_scope} evidence: {target}")
    involved_devices = {
        submission_root.stat().st_dev,
        result_root.stat().st_dev,
        archive_root.stat().st_dev,
    }
    if len(involved_devices) != 1:
        raise ContractError(f"{archive_scope} evidence must remain on one filesystem")
    files = [
        *archive_file_records(submission_root, "submission"),
        *archive_file_records(result_root, "result"),
    ]
    study_root: Path = contract["governance"]["study_root"]
    try:
        submission_source = submission_root.relative_to(study_root)
        result_source = result_root.relative_to(study_root)
    except ValueError as exc:
        raise ContractError("attempt evidence paths must remain below STUDY") from exc
    schema_version = (
        BENCHMARK_ATTEMPT_ARCHIVE_SCHEMA
        if run_kind == "benchmark"
        else (
            "bijux.container.failed_run_archive.v1"
            if disposition == "failed"
            else "bijux.container.superseded_run_archive.v1"
        )
    )
    staging.mkdir(mode=0o750)
    try:
        submission_root.rename(staging / "submission")
        result_root.rename(staging / "result")
        write_json(
            staging / "archive.json",
            {
                "schema_version": schema_version,
                "run_id": contract["run_id"],
                **(
                    {
                        "attempt_id": f"slurm-{job_id}",
                        "tool_id": tool_id,
                        "disposition": disposition,
                    }
                    if run_kind == "benchmark"
                    else {"tool": tool_id}
                ),
                "slurm_job_id": job_id,
                **(
                    {
                        **(
                            {"disposition": "superseded"}
                            if run_kind != "benchmark"
                            else {}
                        ),
                        "reason": reason,
                    }
                    if reason is not None
                    else {}
                ),
                "archived_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
                "source_paths": {
                    "submission": f"study://{submission_source.as_posix()}",
                    "result": f"study://{result_source.as_posix()}",
                },
                "evidence_sha256": sha256_file(staging / "result/evidence.json"),
                "accounting_sha256": sha256_file(
                    staging / "result/slurm-accounting.json"
                ),
                "files": files,
            },
        )
        staging.rename(target)
    except OSError as exc:
        raise ContractError(
            f"{archive_scope} archival stopped; inspect the recoverable staging path "
            f"{staging}: {exc}"
        ) from exc
    return target


def archive_failed_run(
    manifest_path: Path,
    governance_override: Path | None = None,
) -> Path:
    return archive_terminal_run(
        manifest_path,
        governance_override,
        expected_status="failed",
        disposition="failed",
    )


def archive_superseded_run(
    manifest_path: Path,
    reason: str,
    governance_override: Path | None = None,
) -> Path:
    reason = reason.strip()
    if len(reason) < 20:
        raise ContractError("superseded run archival requires a substantive reason")
    return archive_terminal_run(
        manifest_path,
        governance_override,
        expected_status="passed",
        disposition="superseded",
        reason=reason,
    )


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    for name in ("validate", "execute"):
        command = commands.add_parser(name)
        command.add_argument("--governance", type=Path, required=True)
        command.add_argument("--pipeline", type=Path, required=True)
        command.add_argument("--credentials", type=Path, required=True)
    supersede = commands.add_parser("archive-superseded")
    supersede.add_argument("--manifest", type=Path, required=True)
    supersede.add_argument("--governance", type=Path)
    supersede.add_argument("--reason", required=True)
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
        if args.command == "archive-superseded":
            print(
                archive_superseded_run(
                    args.manifest,
                    args.reason,
                    args.governance,
                )
            )
            return 0
        raise ContractError(f"unsupported container worker command: {args.command}")
    except ContractError as exc:
        print(f"container validation contract failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
