#!/usr/bin/env python3
"""Derive container qualification from independent evidence dimensions."""

from __future__ import annotations

import argparse
import hashlib
import json
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:  # Python 3.9 on the governed LUNARC frontend
    import tomli as tomllib


REPORT_SCHEMA = "bijux.container.qualification_report.v1"
INVENTORY_PUBLICATION_SCHEMA = "bijux.container.image_inventory_publication.v1"
RUNTIME_PUBLICATION_SCHEMA = "bijux.container.runtime_verification_publication.v1"
DIMENSIONS = (
    "catalog",
    "source",
    "build",
    "runtime",
    "stage",
    "real_data",
    "science",
    "benchmark",
    "security",
    "release",
    "freshness",
)


class QualificationError(RuntimeError):
    """Raised when qualification inputs cannot support a deterministic report."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise QualificationError(f"cannot read JSON evidence {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise QualificationError(f"JSON evidence must be an object: {path}")
    return value


def load_catalog(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise QualificationError(f"cannot read container catalog {path}: {exc}") from exc
    if not isinstance(value.get("tools"), list):
        raise QualificationError("container catalog lacks a tools array")
    if tuple(value.get("evidence_dimensions", [])) != DIMENSIONS:
        raise QualificationError("catalog evidence dimensions differ from report policy")
    return value


def source_identity_is_resolved(tool: dict[str, Any]) -> bool:
    source_sha = str(tool.get("source_sha256", ""))
    commit = str(tool.get("pinned_commit", ""))
    valid_sha = len(source_sha) == 64 and set(source_sha) <= set("0123456789abcdef")
    valid_commit = len(commit) == 40 and set(commit) <= set("0123456789abcdef")
    return (valid_sha and set(source_sha) != {"0"}) or (
        valid_commit and set(commit) != {"0"}
    )


def catalog_identity_is_resolved(tool: dict[str, Any]) -> bool:
    """Return whether public catalog identity and licensing are release-auditable."""
    license_id = str(tool.get("license", ""))
    return (
        bool(str(tool.get("id", "")))
        and bool(str(tool.get("version", "")))
        and str(tool.get("upstream", "")).startswith("https://")
        and str(tool.get("documentation", "")).startswith("https://")
        and bool(str(tool.get("real_test_stage", "")))
        and license_id not in {"", "NOASSERTION"}
    )


def derive_report(
    catalog_path: Path,
    inventory_path: Path,
    runtime_path: Path,
    runs_path: Path,
) -> dict[str, Any]:
    catalog = load_catalog(catalog_path)
    inventory = load_json(inventory_path)
    runtime = load_json(runtime_path)
    runs = load_json(runs_path)
    if inventory.get("schema_version") != INVENTORY_PUBLICATION_SCHEMA:
        raise QualificationError(
            "qualification requires a published image inventory without operator paths"
        )
    if runtime.get("schema_version") != RUNTIME_PUBLICATION_SCHEMA:
        raise QualificationError(
            "qualification requires published runtime evidence without operator paths"
        )
    inventory_tools = {
        row["tool"]: row for row in inventory.get("tools", []) if isinstance(row, dict)
    }
    runtime_tools = runtime.get("tools", {})
    if not isinstance(runtime_tools, dict):
        raise QualificationError("runtime evidence lacks a tool mapping")
    run_records: dict[str, list[dict[str, Any]]] = defaultdict(list)
    for record in runs.get("runs", []):
        if isinstance(record, dict) and isinstance(record.get("tool"), str):
            run_records[record["tool"]].append(record)

    records = []
    dimension_passes: Counter[str] = Counter()
    qualification_counts: Counter[str] = Counter()
    for tool in catalog["tools"]:
        tool_id = str(tool["id"])
        disposition = str(tool["support_disposition"])
        declared_stage = str(tool.get("real_test_stage", ""))
        if disposition != "retained":
            dimensions = {name: "not_applicable" for name in DIMENSIONS}
            qualification = "unsupported"
            missing = []
        else:
            image = inventory_tools.get(tool_id, {})
            runtime_record = runtime_tools.get(tool_id, {})
            attempts = run_records.get(tool_id, [])
            stage_attempts = [
                record
                for record in attempts
                if record.get("execution_status") == "passed"
                and record.get("stage") == declared_stage
                and bool(record.get("corpus"))
                and bool(record.get("sample"))
                and bool(record.get("domain"))
            ]
            real_passed = bool(stage_attempts)
            mismatched_stages = sorted(
                {
                    str(record.get("stage"))
                    for record in attempts
                    if record.get("execution_status") == "passed"
                    and record.get("stage") != declared_stage
                }
            )
            science_passed = any(
                record.get("scientific_status") == "passed"
                for record in stage_attempts
            )
            evidence_commit_agrees = (
                inventory.get("repository_commit") == runtime.get("repository_commit")
            )
            dimensions = {
                "catalog": (
                    "passed" if catalog_identity_is_resolved(tool) else "missing"
                ),
                "source": (
                    "passed" if source_identity_is_resolved(tool) else "missing"
                ),
                "build": "passed" if image.get("state") == "current" else "missing",
                "runtime": (
                    "passed"
                    if runtime_record.get("status") == "passed"
                    else "missing"
                ),
                "stage": "passed" if real_passed else "missing",
                "real_data": "passed" if real_passed else "missing",
                "science": "passed" if science_passed else "missing",
                "benchmark": "missing",
                "security": "missing",
                "release": "missing",
                "freshness": (
                    "passed"
                    if image.get("state") == "current"
                    and runtime_record.get("status") == "passed"
                    and evidence_commit_agrees
                    else "missing"
                ),
            }
            missing = [name for name, state in dimensions.items() if state != "passed"]
            qualification = "qualified" if not missing else "unverified"
            for name, state in dimensions.items():
                if state == "passed":
                    dimension_passes[name] += 1
        qualification_counts[qualification] += 1
        records.append(
            {
                "id": tool_id,
                "version": str(tool["version"]),
                "support_disposition": disposition,
                "qualification": qualification,
                "dimensions": dimensions,
                "missing_dimensions": missing,
                "real_run_attempts": len(run_records.get(tool_id, [])),
                "real_run_passes": sum(
                    record.get("execution_status") == "passed"
                    for record in run_records.get(tool_id, [])
                ),
                "declared_real_test_stage": declared_stage,
                "passing_real_test_runs": [
                    str(record["run_id"]) for record in stage_attempts
                ],
                "mismatched_passing_stages": mismatched_stages,
            }
        )
    return {
        "schema_version": REPORT_SCHEMA,
        "sources": {
            "catalog": {"path": str(catalog_path), "sha256": sha256_file(catalog_path)},
            "inventory": {
                "path": str(inventory_path),
                "sha256": sha256_file(inventory_path),
            },
            "runtime": {"path": str(runtime_path), "sha256": sha256_file(runtime_path)},
            "runs": {"path": str(runs_path), "sha256": sha256_file(runs_path)},
        },
        "summary": {
            "tool_count": len(records),
            "qualification": dict(sorted(qualification_counts.items())),
            "dimension_passes": {
                name: dimension_passes[name] for name in DIMENSIONS
            },
        },
        "tools": records,
    }


def render_markdown(report: dict[str, Any]) -> str:
    summary = report["summary"]
    qualification = summary["qualification"]
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: containers/contracts/qualification_report.py -->",
        "# Container evidence qualification",
        "",
        "Qualification is derived from independent evidence dimensions. A current image",
        "or successful command does not become a scientific or release claim.",
        "",
        "## Summary",
        "",
        f"- Catalog tools: **{summary['tool_count']}**",
        f"- Qualified: **{qualification.get('qualified', 0)}**",
        f"- Retained but unverified: **{qualification.get('unverified', 0)}**",
        f"- Unsupported: **{qualification.get('unsupported', 0)}**",
        "- Scientific, benchmark, security, and release evidence remain mandatory",
        "  even after build, runtime, and real-data execution pass.",
        "",
        "## Dimension coverage",
        "",
        "| Dimension | Retained tools with passing evidence |",
        "|---|---:|",
    ]
    for dimension in DIMENSIONS:
        lines.append(
            f"| `{dimension}` | {summary['dimension_passes'][dimension]} |"
        )
    lines.extend(
        [
            "",
            "## Tool matrix",
            "",
            "`pass` means evidence exists for that dimension; `missing` means no claim is",
            "permitted. Stage and real-data evidence require a successful run whose stage",
            "exactly matches the catalog's declared real-test stage. Unsupported tools use",
            "`n/a`.",
            "",
            "| Tool | Version | Disposition | Build | Runtime | Real data | Science | Benchmark | Security | Release | Qualification |",
            "|---|---|---|---|---|---|---|---|---|---|---|",
        ]
    )
    labels = {"passed": "pass", "missing": "missing", "not_applicable": "n/a"}
    for tool in report["tools"]:
        dimensions = tool["dimensions"]
        cells = [
            f"`{tool['id']}`",
            f"`{tool['version']}`",
            tool["support_disposition"],
            labels[dimensions["build"]],
            labels[dimensions["runtime"]],
            labels[dimensions["real_data"]],
            labels[dimensions["science"]],
            labels[dimensions["benchmark"]],
            labels[dimensions["security"]],
            labels[dimensions["release"]],
            f"**{tool['qualification']}**",
        ]
        lines.append("| " + " | ".join(cells) + " |")
    lines.extend(
        [
            "",
            "The machine-readable companion is",
            "[qualification.json](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/qualification.json). Real-run details and failures are in",
            "the [run evidence report](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/run-evidence.md).",
        ]
    )
    return "\n".join(lines) + "\n"


def write_text(path: Path, content: str) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    temporary = path.with_suffix(path.suffix + ".partial")
    temporary.write_text(content, encoding="utf-8")
    temporary.replace(path)


def require_current(path: Path, expected: str) -> None:
    try:
        actual = path.read_text(encoding="utf-8")
    except OSError as exc:
        raise QualificationError(f"cannot read generated report {path}: {exc}") from exc
    if actual != expected:
        raise QualificationError(
            f"generated report is stale: {path}; rerun qualification_report.py"
        )


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--catalog", type=Path, required=True)
    parser.add_argument("--inventory", type=Path, required=True)
    parser.add_argument("--runtime", type=Path, required=True)
    parser.add_argument("--runs", type=Path, required=True)
    parser.add_argument("--json-output", type=Path, required=True)
    parser.add_argument("--markdown-output", type=Path, required=True)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when the existing outputs differ instead of rewriting them",
    )
    args = parser.parse_args()
    try:
        report = derive_report(args.catalog, args.inventory, args.runtime, args.runs)
        json_content = json.dumps(report, indent=2, sort_keys=True) + "\n"
        markdown_content = render_markdown(report)
        if args.check:
            require_current(args.json_output, json_content)
            require_current(args.markdown_output, markdown_content)
            action = "verified"
        else:
            write_text(args.json_output, json_content)
            write_text(args.markdown_output, markdown_content)
            action = "qualified"
        print(f"{action} {report['summary']['tool_count']} container tools")
        return 0
    except QualificationError as exc:
        print(f"container qualification failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
