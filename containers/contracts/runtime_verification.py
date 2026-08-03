#!/usr/bin/env python3
"""Verify catalog-owned version and help contracts for admitted SIF images."""

from __future__ import annotations

import argparse
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed
import datetime as dt
from pathlib import Path
import re
import shlex
import socket
import subprocess
import threading
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.9 on the governed LUNARC frontend
    import tomli as tomllib

from container_validation import (
    ContractError,
    git_output,
    load_governance,
    require_under,
    write_json,
)
from image_inventory import INVENTORY_SCHEMA, load_json, sha256_file

EXECUTION_SCHEMA = "bijux.container.runtime_verification.v1"


def catalog_contracts(path: Path) -> dict[str, dict[str, Any]]:
    try:
        catalog = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ContractError(f"cannot read container catalog {path}: {exc}") from exc
    contracts: dict[str, dict[str, Any]] = {}
    for tool in catalog.get("tools", []):
        if not tool.get("apptainer_supported"):
            continue
        tool_id = str(tool["id"])
        version_command = shlex.split(str(tool["version_command"]))
        help_command = shlex.split(str(tool["help_command"]))
        version_regex = str(tool["expected_version_regex"])
        if not version_command or not help_command:
            raise ContractError(f"runtime commands are empty for {tool_id}")
        try:
            re.compile(version_regex)
        except re.error as exc:
            raise ContractError(
                f"version regex is invalid for {tool_id}: {exc}"
            ) from exc
        contracts[tool_id] = {
            "version": str(tool["version"]),
            "version_command": version_command,
            "help_command": help_command,
            "version_regex": version_regex,
        }
    return contracts


def prepare_execution(
    inventory_path: Path,
    catalog_path: Path,
    governance_path: Path,
    output: Path,
) -> dict[str, Any]:
    if output.exists() or output.is_symlink():
        raise ContractError(f"refusing to replace runtime execution: {output}")
    governance = load_governance(governance_path)
    repository: Path = governance["active_repository"]
    if git_output(repository, "status", "--porcelain"):
        raise ContractError(
            "active repository must be clean before runtime verification"
        )
    commit = git_output(repository, "rev-parse", "HEAD")
    inventory = load_json(inventory_path)
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ContractError("image inventory schema is not supported")
    if inventory.get("repository_commit") != commit:
        raise ContractError(
            "image inventory must be regenerated from the active repository commit"
        )
    contracts = catalog_contracts(catalog_path)
    inventory_tools = {
        str(row["tool"]): row
        for row in inventory.get("tools", [])
        if row.get("apptainer_supported")
    }
    if set(inventory_tools) != set(contracts):
        raise ContractError("catalog and supported inventory tool sets disagree")
    tools: dict[str, dict[str, Any]] = {}
    for tool_id, contract in sorted(contracts.items()):
        row = inventory_tools[tool_id]
        if row.get("state") != "current":
            raise ContractError(
                f"runtime verification requires current image: {tool_id}"
            )
        image = Path(str(row.get("selected_image", ""))).resolve()
        require_under(
            image,
            governance["study_root"] / "containers" / "apptainer",
            f"{tool_id} image",
        )
        image_sha256 = str(row.get("selected_image_sha256", ""))
        if not image.is_file() or sha256_file(image) != image_sha256:
            raise ContractError(f"admitted image changed for {tool_id}: {image}")
        tools[tool_id] = {
            **contract,
            "image": str(image),
            "image_sha256": image_sha256,
            "status": "pending",
        }
    execution = {
        "schema_version": EXECUTION_SCHEMA,
        "status": "prepared",
        "prepared_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "repository_commit": commit,
        "repository_clean": True,
        "site": {"id": governance["site_id"], "host": socket.gethostname()},
        "catalog": {"path": str(catalog_path), "sha256": sha256_file(catalog_path)},
        "inventory": {
            "path": str(inventory_path),
            "sha256": sha256_file(inventory_path),
        },
        "tools": tools,
    }
    write_json(output, execution)
    return execution


def run_check(
    image: Path,
    argv: list[str],
    stdout_path: Path,
    stderr_path: Path,
    timeout_seconds: int,
) -> dict[str, Any]:
    for path in (stdout_path, stderr_path):
        if path.exists() or path.is_symlink():
            raise ContractError(f"refusing to replace runtime log: {path}")
        path.parent.mkdir(parents=True, exist_ok=True)
    started = dt.datetime.now(dt.timezone.utc)
    timed_out = False
    with stdout_path.open("wb") as stdout, stderr_path.open("wb") as stderr:
        try:
            completed = subprocess.run(
                [
                    "apptainer",
                    "exec",
                    "--cleanenv",
                    "--containall",
                    "--net",
                    "--network",
                    "none",
                    str(image),
                    *argv,
                ],
                check=False,
                stdout=stdout,
                stderr=stderr,
                timeout=timeout_seconds,
            )
            exit_code: int | None = completed.returncode
        except subprocess.TimeoutExpired:
            timed_out = True
            exit_code = None
    return {
        "argv": argv,
        "started_at_utc": started.isoformat(),
        "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "exit_code": exit_code,
        "timed_out": timed_out,
        "stdout": str(stdout_path),
        "stdout_sha256": sha256_file(stdout_path),
        "stderr": str(stderr_path),
        "stderr_sha256": sha256_file(stderr_path),
    }


def verify_tool(
    tool_id: str,
    tool: dict[str, Any],
    log_root: Path,
    commit: str,
    timeout_seconds: int,
) -> dict[str, Any]:
    image = Path(tool["image"])
    stem = f"{tool_id}-{tool['image_sha256'][:12]}-{commit[:12]}"
    checks = {}
    for name in ("version", "help"):
        checks[name] = run_check(
            image,
            tool[f"{name}_command"],
            log_root / f"{stem}.{name}.stdout.log",
            log_root / f"{stem}.{name}.stderr.log",
            timeout_seconds,
        )
    version_text = ""
    for stream in ("stdout", "stderr"):
        version_text += Path(checks["version"][stream]).read_text(
            encoding="utf-8", errors="replace"
        )
    failures = []
    for name, check in checks.items():
        if check["timed_out"]:
            failures.append(f"{name} command timed out")
        elif check["exit_code"] != 0:
            failures.append(f"{name} command exited {check['exit_code']}")
    if re.search(tool["version_regex"], version_text, re.IGNORECASE) is None:
        failures.append("version output did not match the catalog contract")
    return {
        **tool,
        "status": "passed" if not failures else "failed",
        "checks": checks,
        "failures": failures,
    }


def execute_verification(
    execution_path: Path,
    governance_path: Path,
    jobs: int,
    timeout_seconds: int,
) -> dict[str, Any]:
    if jobs < 1 or timeout_seconds < 1:
        raise ContractError("jobs and timeout must be positive")
    execution = load_json(execution_path)
    if execution.get("schema_version") != EXECUTION_SCHEMA:
        raise ContractError("runtime execution schema is not supported")
    if execution.get("status") != "prepared":
        raise ContractError("runtime execution must be in prepared state")
    governance = load_governance(governance_path)
    log_root = governance["study_root"] / "containers" / "evidence" / "runtime-logs"
    execution["status"] = "running"
    execution["started_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    execution["worker_count"] = jobs
    execution["timeout_seconds"] = timeout_seconds
    write_json(execution_path, execution)
    lock = threading.Lock()
    with ThreadPoolExecutor(max_workers=jobs) as workers:
        futures = {
            workers.submit(
                verify_tool,
                tool_id,
                dict(tool),
                log_root,
                execution["repository_commit"],
                timeout_seconds,
            ): tool_id
            for tool_id, tool in execution["tools"].items()
        }
        for future in as_completed(futures):
            tool_id = futures[future]
            try:
                result = future.result()
            except Exception as exc:
                result = {
                    **execution["tools"][tool_id],
                    "status": "failed",
                    "checks": {},
                    "failures": [f"runtime verification error: {exc}"],
                }
            with lock:
                execution["tools"][tool_id] = result
                write_json(execution_path, execution)
    summary = Counter(tool["status"] for tool in execution["tools"].values())
    execution["summary"] = dict(sorted(summary.items()))
    execution["status"] = "passed" if set(summary) == {"passed"} else "failed"
    execution["finished_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(execution_path, execution)
    return execution


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("--inventory", type=Path, required=True)
    prepare.add_argument("--catalog", type=Path, required=True)
    prepare.add_argument("--governance", type=Path, required=True)
    prepare.add_argument("--output", type=Path, required=True)
    execute = commands.add_parser("execute")
    execute.add_argument("--execution", type=Path, required=True)
    execute.add_argument("--governance", type=Path, required=True)
    execute.add_argument("--jobs", type=int, default=8)
    execute.add_argument("--timeout-seconds", type=int, default=30)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            execution = prepare_execution(
                args.inventory, args.catalog, args.governance, args.output
            )
            print(f"prepared runtime verification for {len(execution['tools'])} images")
            return 0
        execution = execute_verification(
            args.execution, args.governance, args.jobs, args.timeout_seconds
        )
        print(f"runtime verification: {execution['summary']}")
        return 0 if execution["status"] == "passed" else 1
    except ContractError as exc:
        print(f"runtime verification contract failed: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
