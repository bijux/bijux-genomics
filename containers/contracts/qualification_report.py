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
BENCHMARK_REPORT_SCHEMA = "bijux.container.stage_benchmark_report.v1"
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
        raise QualificationError(
            f"cannot read container catalog {path}: {exc}"
        ) from exc
    if not isinstance(value.get("tools"), list):
        raise QualificationError("container catalog lacks a tools array")
    if tuple(value.get("evidence_dimensions", [])) != DIMENSIONS:
        raise QualificationError(
            "catalog evidence dimensions differ from report policy"
        )
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


def admitted_benchmark_records(
    paths: tuple[Path, ...],
    catalog: dict[str, Any],
    inventory_tools: dict[str, dict[str, Any]],
) -> dict[str, list[dict[str, Any]]]:
    """Admit complete benchmark reports tied to current-recipe images."""
    catalog_tools = {str(tool["id"]): tool for tool in catalog["tools"]}
    stage_tools: dict[str, set[str]] = defaultdict(set)
    for tool_id, tool in catalog_tools.items():
        if tool.get("support_disposition") != "retained":
            continue
        for stage_id in tool.get("stage_ids", []):
            stage_tools[str(stage_id)].add(tool_id)

    records: dict[str, list[dict[str, Any]]] = defaultdict(list)
    seen_runs: set[str] = set()
    seen_stages: set[str] = set()
    for path in paths:
        report = load_json(path)
        run_id = str(report.get("run_id", ""))
        stage_id = str(report.get("stage_id", ""))
        if (
            report.get("schema_version") != BENCHMARK_REPORT_SCHEMA
            or report.get("verdict") != "passed"
            or not run_id
            or not stage_id
        ):
            raise QualificationError(f"benchmark report identity is invalid: {path}")
        if run_id in seen_runs or stage_id in seen_stages:
            raise QualificationError(
                f"benchmark report duplicates a run or stage: {path}"
            )
        seen_runs.add(run_id)
        seen_stages.add(stage_id)
        design = report.get("design")
        tool_rows = report.get("tools")
        if not isinstance(design, dict) or not isinstance(tool_rows, list):
            raise QualificationError(f"benchmark report structure is invalid: {path}")
        closure = design.get("tool_closure")
        observed = [
            str(row.get("tool", "")) for row in tool_rows if isinstance(row, dict)
        ]
        expected = stage_tools.get(stage_id, set())
        if (
            not expected
            or not isinstance(closure, list)
            or len(observed) != len(tool_rows)
            or len(observed) != len(set(observed))
            or set(observed) != {str(tool) for tool in closure}
            or set(observed) != expected
            or design.get("runtime") != "apptainer"
            or design.get("scheduler") != "slurm"
            or design.get("container_only") is not True
            or not isinstance(design.get("warmup_repetitions"), int)
            or design["warmup_repetitions"] < 1
            or not isinstance(design.get("measured_repetitions"), int)
            or design["measured_repetitions"] < 3
        ):
            raise QualificationError(
                f"benchmark report lacks complete retained-tool closure: {path}"
            )
        for row in tool_rows:
            tool_id = str(row["tool"])
            image = row.get("image")
            inventory_image = inventory_tools.get(tool_id, {})
            image_sha256 = image.get("sha256") if isinstance(image, dict) else None
            candidates = inventory_image.get("candidates", [])
            admitted_image = image_sha256 == inventory_image.get(
                "selected_image_sha256"
            ) or any(
                isinstance(candidate, dict)
                and candidate.get("sha256") == image_sha256
                and candidate.get("admitted") is True
                and candidate.get("current_recipe") is True
                for candidate in candidates
            )
            if (
                row.get("verdict") != "passed"
                or not isinstance(row.get("performance"), dict)
                or not isinstance(row.get("scientific_observations"), dict)
                or not row["scientific_observations"]
            ):
                raise QualificationError(
                    f"benchmark tool evidence is incomplete for {tool_id}: {path}"
                )
            records[tool_id].append(
                {
                    "run_id": run_id,
                    "stage_id": stage_id,
                    "path": str(path),
                    "sha256": sha256_file(path),
                    "admitted_image": admitted_image,
                }
            )
    return records


def derive_report(
    catalog_path: Path,
    inventory_path: Path,
    runtime_path: Path,
    runs_path: Path,
    benchmark_paths: tuple[Path, ...] = (),
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
    benchmark_records = admitted_benchmark_records(
        benchmark_paths, catalog, inventory_tools
    )
    catalog_sha256 = sha256_file(catalog_path)
    inventory_publication = inventory.get("publication", {})
    runtime_catalog = runtime.get("catalog", {})
    runtime_inventory = runtime.get("inventory", {})
    inventory_catalog_matches = inventory.get("catalog_sha256") == catalog_sha256
    runtime_catalog_matches = (
        isinstance(runtime_catalog, dict)
        and runtime_catalog.get("sha256") == catalog_sha256
    )
    inventory_raw_sha256 = (
        inventory_publication.get("raw_sha256")
        if isinstance(inventory_publication, dict)
        else None
    )
    runtime_inventory_matches = (
        isinstance(runtime_inventory, dict)
        and isinstance(inventory_raw_sha256, str)
        and runtime_inventory.get("sha256") == inventory_raw_sha256
    )
    evidence_commit_agrees = (
        isinstance(inventory.get("repository_commit"), str)
        and bool(inventory.get("repository_commit"))
        and inventory.get("repository_commit") == runtime.get("repository_commit")
    )
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
        stage_attempts: list[dict[str, Any]] = []
        mismatched_stages: list[str] = []
        if disposition != "retained":
            dimensions = {name: "not_applicable" for name in DIMENSIONS}
            qualification = "unsupported"
            missing = []
        else:
            image = inventory_tools.get(tool_id, {})
            runtime_record = runtime_tools.get(tool_id, {})
            image_identity_matches = (
                image.get("version") == str(tool["version"])
                and image.get("support_disposition") == disposition
                and image.get("apptainer_supported") is True
            )
            runtime_identity_matches = (
                runtime_record.get("version") == str(tool["version"])
                and isinstance(image.get("selected_image_sha256"), str)
                and bool(image.get("selected_image_sha256"))
                and runtime_record.get("image_sha256")
                == image.get("selected_image_sha256")
            )
            build_evidence_current = (
                inventory_catalog_matches
                and image_identity_matches
                and image.get("state") == "current"
            )
            runtime_evidence_current = (
                runtime_catalog_matches
                and runtime_inventory_matches
                and runtime_identity_matches
                and runtime_record.get("status") == "passed"
            )
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
                record.get("scientific_status") == "passed" for record in stage_attempts
            )
            benchmark_attempts = benchmark_records.get(tool_id, [])
            benchmark_passed = any(
                record["admitted_image"] for record in benchmark_attempts
            )
            dimensions = {
                "catalog": (
                    "passed" if catalog_identity_is_resolved(tool) else "missing"
                ),
                "source": (
                    "passed" if source_identity_is_resolved(tool) else "missing"
                ),
                "build": "passed" if build_evidence_current else "missing",
                "runtime": ("passed" if runtime_evidence_current else "missing"),
                "stage": "passed" if real_passed else "missing",
                "real_data": "passed" if real_passed else "missing",
                "science": "passed" if science_passed else "missing",
                "benchmark": "passed" if benchmark_passed else "missing",
                "security": "missing",
                "release": "missing",
                "freshness": (
                    "passed"
                    if build_evidence_current
                    and runtime_evidence_current
                    and evidence_commit_agrees
                    else "missing"
                ),
            }
            evidence_mismatches = []
            for matches, label in (
                (inventory_catalog_matches, "inventory_catalog"),
                (runtime_catalog_matches, "runtime_catalog"),
                (runtime_inventory_matches, "runtime_inventory"),
                (evidence_commit_agrees, "repository_commit"),
                (image_identity_matches, "inventory_tool_identity"),
                (runtime_identity_matches, "runtime_tool_identity"),
            ):
                if not matches:
                    evidence_mismatches.append(label)
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
                "passing_benchmark_runs": [
                    record["run_id"]
                    for record in benchmark_records.get(tool_id, [])
                    if record["admitted_image"]
                ],
                "benchmark_stages": sorted(
                    {
                        record["stage_id"]
                        for record in benchmark_records.get(tool_id, [])
                        if record["admitted_image"]
                    }
                ),
                "unadmitted_image_benchmark_runs": [
                    record["run_id"]
                    for record in benchmark_records.get(tool_id, [])
                    if not record["admitted_image"]
                ],
                "evidence_mismatches": (
                    evidence_mismatches if disposition == "retained" else []
                ),
            }
        )
    return {
        "schema_version": REPORT_SCHEMA,
        "evidence_compatibility": {
            "inventory_catalog": inventory_catalog_matches,
            "runtime_catalog": runtime_catalog_matches,
            "runtime_inventory": runtime_inventory_matches,
            "repository_commit": evidence_commit_agrees,
        },
        "sources": {
            "catalog": {"path": str(catalog_path), "sha256": sha256_file(catalog_path)},
            "inventory": {
                "path": str(inventory_path),
                "sha256": sha256_file(inventory_path),
            },
            "runtime": {"path": str(runtime_path), "sha256": sha256_file(runtime_path)},
            "runs": {"path": str(runs_path), "sha256": sha256_file(runs_path)},
            "benchmarks": [
                {"path": str(path), "sha256": sha256_file(path)}
                for path in benchmark_paths
            ],
        },
        "summary": {
            "tool_count": len(records),
            "qualification": dict(sorted(qualification_counts.items())),
            "dimension_passes": {name: dimension_passes[name] for name in DIMENSIONS},
        },
        "tools": records,
    }


def render_markdown(report: dict[str, Any]) -> str:
    summary = report["summary"]
    qualification = summary["qualification"]
    compatibility = report["evidence_compatibility"]
    evidence_chain_current = all(compatibility.values())
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
        f"- Build/runtime evidence chain: **{'current' if evidence_chain_current else 'stale'}**",
        "- Scientific, benchmark, security, and release evidence remain mandatory",
        "  even after build, runtime, and real-data execution pass.",
        "",
        "## Evidence compatibility",
        "",
        "Build, runtime, and freshness pass only when the published image inventory",
        "and runtime campaign match the current catalog, each other, and the same",
        "repository commit. A stale source remains auditable but contributes no pass.",
        "",
        "| Evidence link | Current |",
        "|---|---|",
        f"| Inventory → current catalog | {'yes' if compatibility['inventory_catalog'] else 'no'} |",
        f"| Runtime → current catalog | {'yes' if compatibility['runtime_catalog'] else 'no'} |",
        f"| Runtime → published inventory | {'yes' if compatibility['runtime_inventory'] else 'no'} |",
        f"| Inventory/runtime repository commit | {'yes' if compatibility['repository_commit'] else 'no'} |",
        "",
        "## Dimension coverage",
        "",
        "| Dimension | Retained tools with passing evidence |",
        "|---|---:|",
    ]
    for dimension in DIMENSIONS:
        lines.append(f"| `{dimension}` | {summary['dimension_passes'][dimension]} |")
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
    parser.add_argument(
        "--benchmark-report",
        type=Path,
        action="append",
        default=[],
        help="admit one reviewed stage benchmark report; may be repeated",
    )
    parser.add_argument("--json-output", type=Path, required=True)
    parser.add_argument("--markdown-output", type=Path, required=True)
    parser.add_argument(
        "--check",
        action="store_true",
        help="fail when the existing outputs differ instead of rewriting them",
    )
    args = parser.parse_args()
    try:
        report = derive_report(
            args.catalog,
            args.inventory,
            args.runtime,
            args.runs,
            tuple(args.benchmark_report),
        )
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
