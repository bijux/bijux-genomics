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
    validate_digest,
    write_json,
)
from image_inventory import INVENTORY_SCHEMA, load_json, sha256_file

CAMPAIGN_SCHEMA = "bijux.container.image_campaign.v1"
CAMPAIGN_EVIDENCE_SCHEMA = "bijux.container.image_campaign_evidence.v1"
ATTEMPT_OUTCOME_KEYS = {
    "attempt",
    "status",
    "started_at_utc",
    "finished_at_utc",
    "exit_code",
    "build_log",
    "build_log_sha256",
    "staged_image",
    "image",
    "image_sha256",
    "image_size_bytes",
    "error",
}


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
        tool for tool in inventory.get("tools", []) if tool.get("apptainer_supported")
    ]
    if selection == "catalog_apptainer_supported_noncurrent":
        tools = [
            tool for tool in tools if tool.get("state") in {"missing", "stale_recipe"}
        ]
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
    if (
        recorded_image_root is not None
        and Path(str(recorded_image_root)).resolve() != admission_root
    ):
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
                "attempts": [],
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
    attempt: int,
) -> dict[str, Any]:
    definition = repository / "containers" / "apptainer" / f"{tool}.def"
    definition_digest = sha256_file(definition)
    staged = (
        image_root
        / ".building"
        / f"{tool}-{definition_digest[:12]}-attempt-{attempt}.sif"
    )
    log = (
        evidence_root
        / "build-logs"
        / f"{tool}-{definition_digest[:12]}-{commit[:12]}-attempt-{attempt}.log"
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


def summarize_tool_states(
    selected_tools: list[str],
    tools: dict[str, dict[str, Any]],
) -> tuple[dict[str, int], str]:
    states = Counter(tools[tool]["status"] for tool in selected_tools)
    summary = dict(sorted(states.items()))
    status = "passed" if states == {"passed": len(selected_tools)} else "failed"
    return summary, status


def next_attempt_number(tool: dict[str, Any]) -> int:
    history = tool.setdefault("attempts", [])
    recorded = tool.get("attempt", 0)
    prior_attempt = recorded if isinstance(recorded, int) and recorded > 0 else 0
    return max(len(history), prior_attempt) + 1


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
    require_under(
        evidence_root, governance["containers_root"], "campaign evidence root"
    )
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
    attempts = {}
    for tool in pending:
        row = execution["tools"][tool]
        attempt = next_attempt_number(row)
        attempts[tool] = attempt
        row["attempt"] = attempt
        write_json(execution_path, execution)
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
                attempts[tool],
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
            outcome["attempt"] = attempts[tool]
            with lock:
                row = execution["tools"][tool]
                row.setdefault("attempts", []).append(outcome.copy())
                row.update(outcome)
                write_json(execution_path, execution)
    execution["summary"], execution["status"] = summarize_tool_states(
        execution["selected_tools"],
        execution["tools"],
    )
    execution["finished_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    write_json(execution_path, execution)
    return execution


def queue_failed_tools(
    execution_path: Path,
    requested_tools: list[str],
) -> dict[str, Any]:
    execution = load_json(execution_path)
    if execution.get("schema_version") != CAMPAIGN_EVIDENCE_SCHEMA:
        raise ContractError("campaign execution schema is not supported")
    tools = execution.get("tools")
    if not isinstance(tools, dict):
        raise ContractError("campaign execution tools must be an object")
    failed = sorted(
        tool for tool, row in tools.items() if row.get("status") == "failed"
    )
    selected = sorted(set(requested_tools)) if requested_tools else failed
    unknown = sorted(set(selected) - set(tools))
    if unknown:
        raise ContractError(f"retry requested unknown tools: {unknown}")
    not_failed = sorted(tool for tool in selected if tool not in failed)
    if not_failed:
        raise ContractError(f"retry requested tools that did not fail: {not_failed}")
    if not selected:
        raise ContractError("campaign has no failed tools to retry")
    for tool in selected:
        row = tools[tool]
        history = row.setdefault("attempts", [])
        current = {key: row[key] for key in ATTEMPT_OUTCOME_KEYS if key in row}
        if current and not history:
            current.setdefault("attempt", 1)
            history.append(current)
        for key in ATTEMPT_OUTCOME_KEYS:
            row.pop(key, None)
        row["status"] = "pending"
    execution["status"] = "prepared"
    execution.pop("summary", None)
    execution.pop("finished_at_utc", None)
    execution["retry_queued_at_utc"] = dt.datetime.now(dt.timezone.utc).isoformat()
    execution["retry_tools"] = selected
    write_json(execution_path, execution)
    return execution


def study_relative_path(path: Path) -> str:
    """Return a stable locator below the governed study's containers root."""
    container_indexes = [
        index for index, part in enumerate(path.parts) if part == "containers"
    ]
    if not container_indexes:
        raise ContractError(f"campaign evidence is outside a containers root: {path}")
    return Path(*path.parts[container_indexes[-1] :]).as_posix()


def campaign_report(
    execution_path: Path,
    inventory_path: Path,
) -> tuple[dict[str, Any], str]:
    execution = load_json(execution_path)
    inventory = load_json(inventory_path)
    if execution.get("schema_version") != CAMPAIGN_EVIDENCE_SCHEMA:
        raise ContractError("campaign execution schema is not supported")
    if inventory.get("schema_version") != INVENTORY_SCHEMA:
        raise ContractError("campaign inventory schema is not supported")
    if execution.get("repository_commit") != inventory.get("repository_commit"):
        raise ContractError("campaign execution and inventory commits disagree")
    selected = execution.get("selected_tools")
    tools = execution.get("tools")
    if (
        not isinstance(selected, list)
        or not selected
        or any(not isinstance(tool, str) or not tool for tool in selected)
        or len(selected) != len(set(selected))
        or not isinstance(tools, dict)
    ):
        raise ContractError("campaign execution has invalid selected-tools data")
    inventory_tools = inventory.get("tools")
    if not isinstance(inventory_tools, list):
        raise ContractError("campaign inventory omits per-tool evidence")
    inventory_by_tool = {
        str(row.get("tool")): row for row in inventory_tools if isinstance(row, dict)
    }
    if len(inventory_by_tool) != len(inventory_tools):
        raise ContractError(
            "campaign inventory contains invalid or duplicate tool rows"
        )
    rows = []
    for tool in sorted(str(tool) for tool in selected):
        record = tools.get(tool)
        if not isinstance(record, dict):
            raise ContractError(f"campaign execution omits selected tool: {tool}")
        inventory_record = inventory_by_tool.get(tool)
        if not isinstance(inventory_record, dict):
            raise ContractError(f"campaign inventory omits selected tool: {tool}")
        attempts = record.get("attempts")
        attempt_rows = attempts if isinstance(attempts, list) else []
        attempt_count = len(attempt_rows) or (
            1 if record.get("status") != "pending" else 0
        )
        failed_attempts = sum(
            1
            for attempt in attempt_rows
            if isinstance(attempt, dict) and attempt.get("status") == "failed"
        )
        version = str(record.get("version", ""))
        status = str(record.get("status", ""))
        image_sha256 = str(record.get("image_sha256", ""))
        image_size_bytes = int(record.get("image_size_bytes", 0) or 0)
        build_log_sha256 = str(record.get("build_log_sha256", ""))
        if status == "passed":
            if not version:
                raise ContractError(f"passed tool omits version: {tool}")
            validate_digest(image_sha256, f"{tool}.image_sha256")
            validate_digest(build_log_sha256, f"{tool}.build_log_sha256")
            if image_size_bytes <= 0:
                raise ContractError(f"passed tool has invalid image size: {tool}")
        inventory_state = str(inventory_record.get("state", ""))
        inventory_digest = str(inventory_record.get("selected_image_sha256", ""))
        inventory_matches = (
            inventory_state == "current"
            and str(inventory_record.get("version", "")) == version
            and inventory_digest == image_sha256
        )
        rows.append(
            {
                "tool": tool,
                "version": version,
                "status": status,
                "attempt_count": attempt_count,
                "failed_attempt_count": failed_attempts,
                "image_name": Path(str(record.get("image", ""))).name,
                "image_sha256": image_sha256,
                "image_size_bytes": image_size_bytes,
                "build_log_name": Path(str(record.get("build_log", ""))).name,
                "build_log_sha256": build_log_sha256,
                "inventory_state": inventory_state,
                "inventory_matches_execution": inventory_matches,
            }
        )
    inventory_summary = inventory.get("summary")
    if not isinstance(inventory_summary, dict):
        raise ContractError("campaign inventory omits summary")
    states = inventory_summary.get("states")
    if not isinstance(states, dict):
        raise ContractError("campaign inventory omits state counts")
    selected_count = len(selected)
    checks = {
        "campaign_passed": execution.get("status") == "passed",
        "all_selected_tools_passed": all(row["status"] == "passed" for row in rows),
        "all_selected_images_current": states.get("current") == selected_count,
        "all_selected_records_match_inventory": all(
            row["inventory_matches_execution"] for row in rows
        ),
        "no_staged_images": inventory_summary.get("staged_image_count") == 0,
        "no_unknown_images": inventory_summary.get("unknown_image_count") == 0,
        "no_unreferenced_obsolete_images": (
            inventory_summary.get("unreferenced_obsolete_count") == 0
        ),
    }
    verdict = "passed" if all(checks.values()) else "failed"
    execution_summary = execution.get("summary")
    passed_count = (
        int(execution_summary.get("passed", 0))
        if isinstance(execution_summary, dict)
        else 0
    )
    evidence_root = Path(str(execution.get("evidence_root", "")))
    campaign_root = evidence_root.parent
    report = {
        "schema_version": "bijux.container.image_rebuild_report.v1",
        "run_id": execution.get("run_id"),
        "site": execution.get("site"),
        "repository_commit": execution.get("repository_commit"),
        "finished_at_utc": execution.get("finished_at_utc"),
        "verdict": verdict,
        "selected_tool_count": selected_count,
        "execution_summary": execution_summary,
        "inventory_summary": inventory_summary,
        "checks": checks,
        "evidence_dimensions": ["source", "build", "freshness"],
        "not_evidence_for": [
            "runtime",
            "stage",
            "real_data",
            "science",
            "benchmark",
            "security",
            "release",
        ],
        "source_artifacts": {
            "execution": {
                "name": execution_path.name,
                "study_relative_path": study_relative_path(
                    campaign_root / "campaign" / execution_path.name
                ),
                "sha256": sha256_file(execution_path),
            },
            "after_build_inventory": {
                "name": inventory_path.name,
                "study_relative_path": study_relative_path(
                    campaign_root / "inventory" / inventory_path.name
                ),
                "sha256": sha256_file(inventory_path),
            },
        },
        "tools": rows,
    }
    markdown = [
        "# LUNARC Apptainer image rebuild evidence",
        "",
        "Purpose: record whether every catalog-supported Apptainer image can be rebuilt from a frozen clean repository commit and admitted without leftover or unknown images.",
        "",
        "## Verdict",
        "",
        f"- Verdict: **{verdict}**",
        f"- Run: `{report['run_id']}`",
        f"- Repository commit: `{report['repository_commit']}`",
        f"- Selected tools: **{selected_count}**",
        f"- Final execution: **{passed_count} passed**, **{selected_count - passed_count} not passed**",
        f"- Final inventory: **{states.get('current', 0)} current**, **{states.get('unsupported', 0)} unsupported**, **{inventory_summary.get('staged_image_count', 0)} staged**, **{inventory_summary.get('unknown_image_count', 0)} unknown**, **{inventory_summary.get('unreferenced_obsolete_count', 0)} unreferenced obsolete**",
        "",
        "## Claim boundary",
        "",
        "This report proves source, build, and freshness evidence for the complete retained Apptainer roster. It does not prove runtime behavior, stage integration, real-data correctness, scientific validity, benchmark quality, security acceptance, or release qualification.",
        "",
        "The machine-readable report records SHA-256 digests and study-relative locations for the immutable campaign execution and after-build inventory retained on LUNARC.",
        "",
        "## Acceptance checks",
        "",
    ]
    markdown.extend(
        f"- {'PASS' if passed else 'FAIL'}: `{name}`" for name, passed in checks.items()
    )
    markdown.extend(
        [
            "",
            "## Per-tool evidence",
            "",
            "| Tool | Version | Final status | Attempts | Failed attempts | Image SHA-256 | Size (bytes) | Build-log SHA-256 |",
            "|---|---|---|---:|---:|---|---:|---|",
        ]
    )
    for row in rows:
        markdown.append(
            f"| `{row['tool']}` | `{row['version']}` | `{row['status']}` | {row['attempt_count']} | {row['failed_attempt_count']} | `{row['image_sha256']}` | {row['image_size_bytes']} | `{row['build_log_sha256']}` |"
        )
    return report, "\n".join(markdown) + "\n"


def write_campaign_report(
    execution_path: Path,
    inventory_path: Path,
    json_output: Path,
    markdown_output: Path,
    check: bool,
) -> None:
    report, markdown = campaign_report(execution_path, inventory_path)
    expected_json = json.dumps(report, indent=2, sort_keys=True) + "\n"
    if check:
        if (
            not json_output.is_file()
            or json_output.read_text(encoding="utf-8") != expected_json
        ):
            raise ContractError(f"image rebuild JSON report drifted: {json_output}")
        if (
            not markdown_output.is_file()
            or markdown_output.read_text(encoding="utf-8") != markdown
        ):
            raise ContractError(
                f"image rebuild Markdown report drifted: {markdown_output}"
            )
        return
    write_json(json_output, report)
    markdown_output.write_text(markdown, encoding="utf-8")


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
    retry = commands.add_parser("retry-failed")
    retry.add_argument("--execution", type=Path, required=True)
    retry.add_argument("--tool", action="append", default=[])
    report = commands.add_parser("report")
    report.add_argument("--execution", type=Path, required=True)
    report.add_argument("--inventory", type=Path, required=True)
    report.add_argument("--json-output", type=Path, required=True)
    report.add_argument("--markdown-output", type=Path, required=True)
    report.add_argument("--check", action="store_true")
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
        elif args.command == "execute":
            execution = execute_campaign(args.execution, args.governance, args.jobs)
            print(f"image campaign {execution['status']}: {execution['summary']}")
        elif args.command == "retry-failed":
            execution = queue_failed_tools(args.execution, args.tool)
            print(f"queued failed image builds: {', '.join(execution['retry_tools'])}")
        else:
            write_campaign_report(
                args.execution,
                args.inventory,
                args.json_output,
                args.markdown_output,
                args.check,
            )
            print(
                "image rebuild report: OK"
                if args.check
                else "generated image rebuild report"
            )
        return 0
    except (ContractError, OSError, json.JSONDecodeError) as exc:
        print(f"container image campaign failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
