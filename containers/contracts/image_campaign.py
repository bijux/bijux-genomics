#!/usr/bin/env python3
"""Build and admit the governed LUNARC Apptainer image roster."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import socket
import subprocess
import sys
import threading
from collections import Counter
from concurrent.futures import ThreadPoolExecutor, as_completed
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.9 on the governed LUNARC frontend
    import tomli as tomllib

from container_build import finalize_image
from container_validation import (
    ContractError,
    freeze_repository,
    git_output,
    load_governance,
    require_under,
    write_json,
)
from image_inventory import INVENTORY_SCHEMA, load_json, sha256_file

CAMPAIGN_SCHEMA = "bijux.container.image_campaign.v1"
CAMPAIGN_EVIDENCE_SCHEMA = "bijux.container.image_campaign_evidence.v1"


def load_campaign(path: Path) -> dict[str, Any]:
    campaign = load_json(path)
    if campaign.get("schema_version") != CAMPAIGN_SCHEMA:
        raise ContractError(f"unsupported image campaign schema: {path}")
    if campaign.get("run_id") != "run-1000":
        raise ContractError("image preparation campaign must retain run-1000 identity")
    if campaign.get("runtime") != "apptainer" or campaign.get("site") != "lunarc":
        raise ContractError("image preparation campaign must target LUNARC Apptainer")
    if campaign.get("selection") not in {
        "catalog_apptainer_supported_all",
        "catalog_apptainer_supported_noncurrent",
    }:
        raise ContractError("image campaign selection policy is not recognized")
    return campaign


def load_tool_versions(path: Path) -> dict[str, str]:
    try:
        catalog = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise ContractError(f"cannot read container catalog {path}: {exc}") from exc
    versions = {}
    for tool in catalog.get("tools", []):
        if tool.get("apptainer_supported"):
            versions[str(tool["id"])] = str(tool["version"])
    return versions


def selected_tools(inventory: dict[str, Any], selection: str) -> list[str]:
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ContractError("image inventory schema is not supported")
    tools = [
        tool
        for tool in inventory.get("tools", [])
        if tool.get("apptainer_supported")
    ]
    if selection == "catalog_apptainer_supported_noncurrent":
        tools = [tool for tool in tools if tool.get("state") in {"missing", "stale_recipe"}]
    return sorted(str(tool["tool"]) for tool in tools)


def admission_roots(
    containers_root: Path,
    image_root: Path | None,
) -> tuple[Path, Path]:
    containers_root = containers_root.resolve()
    canonical_image_root = containers_root / "apptainer"
    admission_root = (image_root or canonical_image_root).resolve()
    require_under(admission_root, containers_root, "campaign image root")
    evidence_root = (
        containers_root / "evidence"
        if admission_root == canonical_image_root
        else admission_root.parent / "evidence"
    ).resolve()
    require_under(evidence_root, containers_root, "campaign evidence root")
    if admission_root == evidence_root:
        raise ContractError("campaign image and evidence roots must differ")
    return admission_root, evidence_root


def create_execution(
    campaign_path: Path,
    inventory_path: Path,
    catalog_path: Path,
    governance_path: Path,
    image_root: Path | None,
    output: Path,
) -> dict[str, Any]:
    if output.exists() or output.is_symlink():
        raise ContractError(f"refusing to replace campaign execution: {output}")
    campaign = load_campaign(campaign_path)
    inventory = load_json(inventory_path)
    governance = load_governance(governance_path)
    containers_root: Path = governance["containers_root"]
    admission_root, evidence_root = admission_roots(containers_root, image_root)
    recorded_image_root = inventory.get("image_root")
    if recorded_image_root is not None and Path(str(recorded_image_root)).resolve() != admission_root:
        raise ContractError("image inventory and campaign admission roots disagree")
    repository: Path = governance["active_repository"]
    if git_output(repository, "status", "--porcelain"):
        raise ContractError("active repository must be clean before image preparation")
    commit = git_output(repository, "rev-parse", "HEAD")
    if inventory.get("repository_commit") != commit:
        raise ContractError(
            "image inventory must be regenerated from the active commit"
        )
    frozen_repository = freeze_repository(
        commit,
        governance_path,
        checkout_scope="complete",
    )
    tools = selected_tools(inventory, str(campaign["selection"]))
    versions = load_tool_versions(catalog_path)
    unknown = sorted(set(tools) - set(versions))
    if unknown:
        raise ContractError(f"inventory selected tools absent from catalog: {unknown}")
    execution = {
        "schema_version": CAMPAIGN_EVIDENCE_SCHEMA,
        "run_id": campaign["run_id"],
        "status": "prepared",
        "prepared_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "site": {
            "id": governance["site_id"],
            "host": socket.gethostname(),
        },
        "repository_commit": commit,
        "repository_clean": True,
        "image_root": str(admission_root),
        "evidence_root": str(evidence_root),
        "frozen_repository": str(frozen_repository),
        "campaign": {
            "path": str(campaign_path),
            "sha256": sha256_file(campaign_path),
        },
        "catalog": {
            "path": str(catalog_path),
            "sha256": sha256_file(catalog_path),
        },
        "inventory": {
            "path": str(inventory_path),
            "sha256": sha256_file(inventory_path),
        },
        "selected_tools": tools,
        "tools": {
            tool: {
                "version": versions[tool],
                "status": "pending",
                "definition": f"containers/apptainer/{tool}.def",
            }
            for tool in tools
        },
    }
    write_json(output, execution)
    return execution


def build_one(
    tool: str,
    version: str,
    commit: str,
    governance_path: Path,
    repository: Path,
    image_root: Path,
    evidence_root: Path,
) -> dict[str, Any]:
    definition = repository / "containers" / "apptainer" / f"{tool}.def"
    definition_digest = sha256_file(definition)
    staged = image_root / ".building" / f"{tool}-{definition_digest[:12]}.sif"
    log = (
        evidence_root
        / "build-logs"
        / f"{tool}-{definition_digest[:12]}-{commit[:12]}.log"
    )
    staged.parent.mkdir(parents=True, exist_ok=True)
    log.parent.mkdir(parents=True, exist_ok=True)
    if staged.exists() or staged.is_symlink():
        raise ContractError(f"staged image path already exists: {staged}")
    if log.exists():
        raise ContractError(f"build log path already exists: {log}")
    started = dt.datetime.now(dt.timezone.utc)
    with log.open("wb") as stream:
        completed = subprocess.run(
            ["apptainer", "build", str(staged), str(definition)],
            check=False,
            stdout=stream,
            stderr=subprocess.STDOUT,
        )
    if completed.returncode != 0:
        return {
            "status": "failed",
            "started_at_utc": started.isoformat(),
            "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
            "exit_code": completed.returncode,
            "build_log": str(log),
            "build_log_sha256": sha256_file(log),
            "staged_image": str(staged) if staged.exists() else None,
        }
    admitted = finalize_image(
        governance_path,
        tool,
        version,
        commit,
        definition,
        staged,
        log,
        repository_path=repository,
        image_root_override=image_root,
        evidence_root_override=evidence_root,
    )
    return {
        "status": "passed",
        "started_at_utc": started.isoformat(),
        "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "exit_code": 0,
        "build_log": str(log),
        "build_log_sha256": sha256_file(log),
        "image": str(admitted),
        "image_sha256": sha256_file(admitted),
        "image_size_bytes": admitted.stat().st_size,
    }


def execute_campaign(
    execution_path: Path,
    governance_path: Path,
    jobs: int,
) -> dict[str, Any]:
    if jobs < 1 or jobs > 4:
        raise ContractError("frontend image build concurrency must be between 1 and 4")
    execution = load_json(execution_path)
    if execution.get("schema_version") != CAMPAIGN_EVIDENCE_SCHEMA:
        raise ContractError("campaign execution schema is not supported")
    if execution.get("status") not in {"prepared", "running"}:
        raise ContractError(
            f"campaign cannot run from status {execution.get('status')}"
        )
    governance = load_governance(governance_path)
    image_root = Path(str(execution.get("image_root", ""))).resolve()
    evidence_root = Path(str(execution.get("evidence_root", ""))).resolve()
    require_under(image_root, governance["containers_root"], "campaign image root")
    require_under(evidence_root, governance["containers_root"], "campaign evidence root")
    repository = Path(str(execution.get("frozen_repository", ""))).resolve()
    require_under(
        repository,
        governance["frozen_repositories"] / "image-builds",
        "frozen image-build repository",
    )
    commit = git_output(repository, "rev-parse", "HEAD")
    if git_output(repository, "status", "--porcelain"):
        raise ContractError(
            "frozen image-build repository must remain clean during image preparation"
        )
    if commit != execution.get("repository_commit"):
        raise ContractError("frozen image-build repository has the wrong commit")
    execution["status"] = "running"
    execution.setdefault("started_at_utc", dt.datetime.now(dt.timezone.utc).isoformat())
    execution["worker_count"] = jobs
    write_json(execution_path, execution)
    lock = threading.Lock()
    pending = [
        tool
        for tool in execution["selected_tools"]
        if execution["tools"][tool]["status"] == "pending"
    ]
    with ThreadPoolExecutor(max_workers=jobs) as workers:
        futures = {
            workers.submit(
                build_one,
                tool,
                execution["tools"][tool]["version"],
                commit,
                governance_path,
                repository,
                image_root,
                evidence_root,
            ): tool
            for tool in pending
        }
        for future in as_completed(futures):
            tool = futures[future]
            try:
                outcome = future.result()
            except Exception as exc:  # noqa: BLE001 - retain every tool outcome
                outcome = {
                    "status": "failed",
                    "finished_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
                    "error": f"{type(exc).__name__}: {exc}",
                }
            with lock:
                execution["tools"][tool].update(outcome)
                write_json(execution_path, execution)
    states = Counter(tool["status"] for tool in execution["tools"].values())
    execution["summary"] = dict(sorted(states.items()))
    execution["status"] = "passed" if states == {"passed": len(pending)} else "failed"
    if not pending:
        execution["status"] = "passed"
    execution["finished_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(execution_path, execution)
    return execution


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("--campaign", type=Path, required=True)
    prepare.add_argument("--inventory", type=Path, required=True)
    prepare.add_argument("--catalog", type=Path, required=True)
    prepare.add_argument("--governance", type=Path, required=True)
    prepare.add_argument("--image-root", type=Path)
    prepare.add_argument("--output", type=Path, required=True)
    execute = commands.add_parser("execute")
    execute.add_argument("--execution", type=Path, required=True)
    execute.add_argument("--governance", type=Path, required=True)
    execute.add_argument("--jobs", type=int, default=2)
    return root


def main(argv: list[str] | None = None) -> int:
    args = parser().parse_args(argv)
    try:
        if args.command == "prepare":
            execution = create_execution(
                args.campaign,
                args.inventory,
                args.catalog,
                args.governance,
                args.image_root,
                args.output,
            )
            print(
                f"prepared {execution['run_id']} for "
                f"{len(execution['selected_tools'])} images at "
                f"{execution['repository_commit']}"
            )
        else:
            execution = execute_campaign(args.execution, args.governance, args.jobs)
            print(f"image campaign {execution['status']}: {execution['summary']}")
        return 0
    except (ContractError, OSError, json.JSONDecodeError) as exc:
        print(f"container image campaign failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
