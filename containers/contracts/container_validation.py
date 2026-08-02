#!/usr/bin/env python3
"""Validate and execute immutable container checks on a Slurm worker."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import os
from pathlib import Path
import re
import socket
import subprocess
import sys
from typing import Any

import yaml


PIPELINE_SCHEMA = "bijux.container.pipeline.v1"
CREDENTIAL_SCHEMA = "bijux.container.credential_reference.v1"
EVIDENCE_SCHEMA = "bijux.container.evidence.v1"
RUN_ID_PATTERN = re.compile(r"run-[1-9][0-9]{3,}")
SHA256_PATTERN = re.compile(r"[0-9a-f]{64}")
GOVERNED_ROOT = Path("/home/bijan/lu2024-12-24/Dardel/STUDY").resolve()
ACTIVE_SOURCE_ROOT = Path("/home/bijan/bijux/bijux-genomics").resolve()
FROZEN_SOURCE_ROOT = Path("/home/bijan/bijux/bijux-genomics-runs").resolve()


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


def freeze_repository(commit: str) -> Path:
    if not re.fullmatch(r"[0-9a-f]{40}", commit):
        raise ContractError("frozen repository commit must be a full Git commit")
    if git_output(ACTIVE_SOURCE_ROOT, "status", "--porcelain"):
        raise ContractError("active LUNARC repository must be clean before freezing a run")
    git_output(ACTIVE_SOURCE_ROOT, "cat-file", "-e", f"{commit}^{{commit}}")
    target = FROZEN_SOURCE_ROOT / commit
    if target.exists():
        if git_output(target, "rev-parse", "HEAD") != commit:
            raise ContractError(f"existing frozen worktree has the wrong commit: {target}")
        if git_output(target, "status", "--porcelain"):
            raise ContractError(f"existing frozen worktree is dirty: {target}")
        return target
    FROZEN_SOURCE_ROOT.mkdir(parents=True, exist_ok=True)
    completed = subprocess.run(
        [
            "git",
            "-C",
            str(ACTIVE_SOURCE_ROOT),
            "worktree",
            "add",
            "--detach",
            str(target),
            commit,
        ],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise ContractError(f"cannot create frozen worktree: {completed.stderr.strip()}")
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
            raise ContractError(f"credential reference {index} variable_names must be a list")
    if config["credentials_required"] != bool(references):
        raise ContractError("credentials_required must agree with credential references")
    redaction = config.get("redaction")
    if not isinstance(redaction, dict) or redaction.get("publish_values") is not False:
        raise ContractError("credential redaction must explicitly forbid publishing values")
    require_keys(redaction, {"publish_values"}, "credential redaction")


def validate_pipeline(
    config: dict[str, Any], pipeline_path: Path, credential_path: Path
) -> dict[str, Any]:
    require_keys(
        config,
        {
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
        },
        "pipeline contract",
    )
    if config.get("schema_version") != PIPELINE_SCHEMA:
        raise ContractError(f"pipeline schema_version must be {PIPELINE_SCHEMA}")

    run = require_mapping(config, "run")
    require_keys(
        run,
        {"id", "campaign", "corpus", "sample", "domain", "stage", "tool", "operator"},
        "run",
    )
    run_id = require_text(run, "id")
    if not RUN_ID_PATTERN.fullmatch(run_id):
        raise ContractError("run.id must use the run-1001 provenance sequence")
    for key in ("campaign", "corpus", "sample", "domain", "stage", "tool", "operator"):
        require_text(run, key)

    repository = require_mapping(config, "repository")
    require_keys(repository, {"path", "commit", "clean"}, "repository")
    repository_path = resolved(require_text(repository, "path"))
    expected_commit = require_text(repository, "commit")
    if not re.fullmatch(r"[0-9a-f]{40}", expected_commit):
        raise ContractError("repository.commit must be a full Git commit")
    is_active_checkout = repository_path == ACTIVE_SOURCE_ROOT
    is_frozen_checkout = (
        repository_path.parent == FROZEN_SOURCE_ROOT
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
    require_under(image_path, GOVERNED_ROOT / "containers" / "apptainer", "image.path")
    if not image_path.is_file() or image_path.suffix != ".sif":
        raise ContractError(f"image.path must be an existing immutable SIF: {image_path}")
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
    for index, item in enumerate(inputs):
        if not isinstance(item, dict):
            raise ContractError(f"data input {index} must be a mapping")
        allowed_input_keys = {"id", "kind", "path", "sha256"}
        unknown_input_keys = set(item).difference(allowed_input_keys)
        missing_input_keys = {"id", "path", "sha256"}.difference(item)
        if missing_input_keys:
            raise ContractError(
                f"data input {index} is missing required fields: {sorted(missing_input_keys)}"
            )
        if unknown_input_keys:
            raise ContractError(
                f"data input {index} contains unknown fields: {sorted(unknown_input_keys)}"
            )
        require_text(item, "id")
        input_kind = item.get("kind", "data")
        if input_kind not in {"data", "reference", "database", "upstream_result"}:
            raise ContractError(f"data input {index} has unsupported kind: {input_kind}")
        declared_input_path = Path(require_text(item, "path"))
        if not declared_input_path.is_absolute():
            raise ContractError(f"data input {index} path must be absolute")
        input_path = resolved(str(declared_input_path))
        input_root = (
            GOVERNED_ROOT / "results"
            if input_kind == "upstream_result"
            else GOVERNED_ROOT / "data"
        )
        require_under(input_path, input_root, f"data.inputs[{index}].path")
        if not input_path.is_file():
            raise ContractError(f"data input does not exist: {input_path}")
        digest = require_text(item, "sha256")
        validate_digest(digest, f"data.inputs[{index}].sha256")
        if sha256_file(input_path) != digest:
            raise ContractError(f"data input digest mismatch: {input_path}")
        input_binds.append((input_path, declared_input_path))

    commands = require_mapping(config, "commands")
    require_keys(commands, {"version", "help", "real", "negative"}, "commands")
    for key in ("version", "help", "real", "negative"):
        require_argv(commands, key)

    outputs = require_mapping(config, "outputs")
    require_keys(outputs, {"result_root", "required_files"}, "outputs")
    declared_result_root = Path(require_text(outputs, "result_root"))
    if not declared_result_root.is_absolute():
        raise ContractError("outputs.result_root must be absolute")
    result_root = resolved(str(declared_result_root))
    require_under(
        result_root,
        GOVERNED_ROOT / "results" / "container-validation",
        "outputs.result_root",
    )
    if result_root.name != run_id:
        raise ContractError("outputs.result_root must end with the run id")
    required_files = outputs.get("required_files")
    if not isinstance(required_files, list) or any(
        not isinstance(path, str) or not path for path in required_files
    ):
        raise ContractError("outputs.required_files must be a list of relative paths")
    for path in required_files:
        candidate = Path(path)
        if candidate.is_absolute() or ".." in candidate.parts:
            raise ContractError(f"required output must be a safe relative path: {path}")

    resources = require_mapping(config, "resources")
    require_keys(
        resources,
        {"account", "partition", "cpus", "memory_mb", "walltime", "scratch_mb"},
        "resources",
    )
    for key in ("account", "partition", "walltime"):
        require_text(resources, key)
    for key in ("cpus", "memory_mb", "scratch_mb"):
        value = resources.get(key)
        if not isinstance(value, int) or value <= 0:
            raise ContractError(f"resources.{key} must be a positive integer")

    execution = require_mapping(config, "execution")
    require_keys(execution, {"locale", "timezone", "runtime_network"}, "execution")
    if execution.get("locale") != "C" or execution.get("timezone") != "UTC":
        raise ContractError("execution must freeze locale C and timezone UTC")
    if execution.get("runtime_network") is not False:
        raise ContractError("container validation runs must disable runtime network")

    acceptance = require_mapping(config, "acceptance")
    require_keys(acceptance, {"version_regex", "negative_exit_nonzero"}, "acceptance")
    try:
        re.compile(require_text(acceptance, "version_regex"))
    except re.error as exc:
        raise ContractError(f"acceptance.version_regex is invalid: {exc}") from exc
    if acceptance.get("negative_exit_nonzero") is not True:
        raise ContractError("acceptance must require a non-zero negative check")

    if credential_path.resolve().parent != pipeline_path.resolve().parent:
        raise ContractError("pipeline and credential contracts must be archived together")

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
        "required_files": required_files,
        "resources": resources,
        "acceptance": acceptance,
    }


def validate_contracts(pipeline_path: Path, credential_path: Path) -> dict[str, Any]:
    credentials = load_yaml(credential_path)
    validate_credentials(credentials)
    pipeline = load_yaml(pipeline_path)
    resolved_contract = validate_pipeline(pipeline, pipeline_path, credential_path)
    resolved_contract["pipeline"] = pipeline
    resolved_contract["credentials"] = credentials
    return resolved_contract


def run_apptainer(
    image_path: Path,
    argv: list[str],
    binds: list[tuple[Path, Path, str]],
    stdout_path: Path,
    stderr_path: Path,
) -> int:
    command = ["apptainer", "exec", "--cleanenv", "--containall", "--net", "--network", "none"]
    for source, target, mode in binds:
        command.extend(["--bind", f"{source}:{target}:{mode}"])
    command.extend([str(image_path), *argv])
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        completed = subprocess.run(
            command,
            check=False,
            stdout=stdout,
            stderr=stderr,
            env={"LANG": "C", "LC_ALL": "C", "TZ": "UTC", "PATH": os.environ["PATH"]},
        )
    return completed.returncode


def write_json(path: Path, value: dict[str, Any]) -> None:
    temporary = path.with_suffix(path.suffix + ".partial")
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    temporary.replace(path)


def execute(pipeline_path: Path, credential_path: Path) -> int:
    contract = validate_contracts(pipeline_path, credential_path)
    result_root: Path = contract["result_root"]
    if result_root.exists():
        raise ContractError(f"refusing to overwrite run result root: {result_root}")
    result_root.mkdir(parents=True, mode=0o750)
    (result_root / "config-pipeline.yml").write_bytes(pipeline_path.read_bytes())
    (result_root / "config-credential.yml").write_bytes(credential_path.read_bytes())

    input_binds = [(source, target, "ro") for source, target in contract["input_binds"]]
    binds = [
        *input_binds,
        (result_root, contract["declared_result_root"], "rw"),
    ]
    checks: dict[str, dict[str, Any]] = {}
    for name in ("version", "help", "real", "negative"):
        stdout_path = result_root / f"{name}.stdout.log"
        stderr_path = result_root / f"{name}.stderr.log"
        exit_code = run_apptainer(
            contract["image_path"],
            contract["commands"][name],
            binds,
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
    ) + (result_root / "version.stderr.log").read_text(encoding="utf-8", errors="replace")
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

    evidence = {
        "schema_version": EVIDENCE_SCHEMA,
        "run_id": contract["run_id"],
        "status": "passed" if not failures else "failed",
        "failure_class": None if not failures else "container_validation",
        "failures": failures,
        "repository_commit": contract["repository_commit"],
        "repository_path": str(contract["repository_path"]),
        "repository_clean": True,
        "pipeline_sha256": sha256_file(pipeline_path),
        "credential_reference_sha256": sha256_file(credential_path),
        "image": {
            "path": str(contract["image_path"]),
            "sha256": contract["image_sha256"],
            "size_bytes": contract["image_path"].stat().st_size,
        },
        "inputs": contract["inputs"],
        "checks": checks,
        "outputs": output_records,
        "runtime": {
            "host": socket.gethostname(),
            "slurm_job_id": os.environ.get("SLURM_JOB_ID", ""),
            "cpus": os.environ.get("SLURM_CPUS_PER_TASK", ""),
            "started_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "apptainer_version": subprocess.run(
                ["apptainer", "--version"], check=False, capture_output=True, text=True
            ).stdout.strip(),
        },
    }
    write_json(result_root / "evidence.json", evidence)
    return 0 if not failures else 1


def render_slurm(pipeline_path: Path, credential_path: Path, output_path: Path) -> None:
    contract = validate_contracts(pipeline_path, credential_path)
    resources = contract["resources"]
    run = contract["pipeline"]["run"]
    output_path.parent.mkdir(parents=True, exist_ok=True)
    job_name = f"{run['domain']}-{run['stage'].split('.')[-1]}-{run['tool']}-{run['sample']}"
    stdout_path = output_path.parent / "slurm-%j.stdout.log"
    stderr_path = output_path.parent / "slurm-%j.stderr.log"
    script = f"""#!/usr/bin/env bash
#SBATCH --job-name={job_name}
#SBATCH --account={resources['account']}
#SBATCH --partition={resources['partition']}
#SBATCH --cpus-per-task={resources['cpus']}
#SBATCH --mem={resources['memory_mb']}M
#SBATCH --time={resources['walltime']}
#SBATCH --output={stdout_path}
#SBATCH --error={stderr_path}

set -euo pipefail
export LANG=C
export LC_ALL=C
export TZ=UTC
exec python3 {contract['repository_path'] / 'containers/contracts/container_validation.py'} execute \\
  --pipeline {pipeline_path.resolve()} \\
  --credentials {credential_path.resolve()}
"""
    output_path.write_text(script, encoding="utf-8")
    output_path.chmod(0o750)


def collect_accounting(manifest_path: Path) -> Path:
    manifest_path = manifest_path.resolve()
    require_under(
        manifest_path,
        GOVERNED_ROOT / "containers" / "manifests",
        "manifest",
    )
    pipeline_path = manifest_path / "config-pipeline.yml"
    credential_path = manifest_path / "config-credential.yml"
    contract = validate_contracts(pipeline_path, credential_path)
    job_id = (manifest_path / "slurm-job-id.txt").read_text(encoding="utf-8").strip()
    if not job_id.isdigit():
        raise ContractError("manifest Slurm job id must be numeric")
    queued = subprocess.run(
        ["squeue", "-h", "-j", job_id, "-o", "%T"],
        check=False,
        capture_output=True,
        text=True,
    )
    if queued.returncode != 0:
        raise ContractError(f"cannot query queued job {job_id}: {queued.stderr.strip()}")
    if queued.stdout.strip():
        raise ContractError(f"Slurm job {job_id} is not terminal: {queued.stdout.strip()}")
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
        raise ContractError(f"cannot collect accounting for {job_id}: {completed.stderr.strip()}")
    rows = []
    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        values = line.split("|")
        if len(values) != len(fields):
            raise ContractError(f"unexpected sacct row for {job_id}: {line}")
        rows.append(dict(zip(fields, values, strict=True)))
    if not rows:
        raise ContractError(f"Slurm accounting is unavailable for terminal job {job_id}")
    result_root: Path = contract["result_root"]
    if not result_root.is_dir():
        raise ContractError(f"run result root does not exist: {result_root}")
    accounting_path = result_root / "slurm-accounting.json"
    if accounting_path.exists():
        raise ContractError(f"refusing to overwrite accounting evidence: {accounting_path}")
    submit_path = manifest_path / "submit.sh"
    record = {
        "schema_version": "bijux.slurm.accounting.v1",
        "run_id": contract["run_id"],
        "slurm_job_id": job_id,
        "collected_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "rows": rows,
        "submit_script_sha256": sha256_file(submit_path),
        "pipeline_sha256": sha256_file(pipeline_path),
        "credential_reference_sha256": sha256_file(credential_path),
    }
    write_json(accounting_path, record)
    (result_root / "submit.sh").write_bytes(submit_path.read_bytes())
    (result_root / "slurm-job-id.txt").write_text(job_id + "\n", encoding="utf-8")
    return accounting_path


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    for name in ("validate", "execute"):
        command = commands.add_parser(name)
        command.add_argument("--pipeline", type=Path, required=True)
        command.add_argument("--credentials", type=Path, required=True)
    render = commands.add_parser("render-slurm")
    render.add_argument("--pipeline", type=Path, required=True)
    render.add_argument("--credentials", type=Path, required=True)
    render.add_argument("--output", type=Path, required=True)
    freeze = commands.add_parser("freeze-repository")
    freeze.add_argument("--commit", required=True)
    collect = commands.add_parser("collect-accounting")
    collect.add_argument("--manifest", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "validate":
            validate_contracts(args.pipeline, args.credentials)
            print("container validation contracts: OK")
            return 0
        if args.command == "execute":
            return execute(args.pipeline, args.credentials)
        if args.command == "freeze-repository":
            print(freeze_repository(args.commit))
            return 0
        if args.command == "collect-accounting":
            print(collect_accounting(args.manifest))
            return 0
        render_slurm(args.pipeline, args.credentials, args.output)
        print(args.output)
        return 0
    except ContractError as exc:
        print(f"container validation contract failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
