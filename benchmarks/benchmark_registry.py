#!/usr/bin/env python3
"""Validate and report container-only real-data stage benchmark campaigns."""

from __future__ import annotations

import argparse
import json
import re
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

SCHEMA_VERSION = "bijux.benchmark.stage_campaigns.v1"
RUN_PATTERN = re.compile(r"^run-(2[0-9]{3}|[3-9][0-9]{3,})$")
REQUIRED_PERFORMANCE_METRICS = {
    "wall_seconds",
    "cpu_seconds",
    "peak_rss_kib",
    "input_bytes",
    "output_bytes",
}
COMPARATIVE_PERFORMANCE_METRICS = REQUIRED_PERFORMANCE_METRICS | {
    "cpu_utilization_percent",
    "filesystem_input_operations",
    "filesystem_output_operations",
    "throughput_input_bytes_per_second",
}


class RegistryError(RuntimeError):
    """Raised when a benchmark campaign makes an incomplete claim."""


def load_toml(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise RegistryError(f"cannot read TOML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise RegistryError(f"TOML root must be a mapping: {path}")
    return value


def retained_stage_tools(catalog: dict[str, Any]) -> dict[str, set[str]]:
    stages: dict[str, set[str]] = {}
    for tool in catalog.get("tools", []):
        if tool.get("support_disposition") != "retained":
            continue
        tool_id = str(tool.get("id", ""))
        for stage_id in tool.get("stage_ids", []):
            stages.setdefault(str(stage_id), set()).add(tool_id)
    return stages


def require_text(campaign: dict[str, Any], key: str) -> str:
    value = campaign.get(key)
    if not isinstance(value, str) or not value.strip():
        raise RegistryError(f"benchmark campaign {key} must be non-empty text")
    return value


def require_text_list(
    campaign: dict[str, Any], key: str, *, allow_empty: bool = False
) -> list[str]:
    value = campaign.get(key)
    if (
        not isinstance(value, list)
        or (not value and not allow_empty)
        or any(not isinstance(item, str) or not item for item in value)
    ):
        raise RegistryError(f"benchmark campaign {key} must be a non-empty text list")
    if len(value) != len(set(value)):
        raise RegistryError(f"benchmark campaign {key} contains duplicates")
    return value


def threads_by_tool(campaign: dict[str, Any]) -> dict[str, int]:
    """Return the declared execution width for every tool in a campaign."""
    tool_ids = require_text_list(campaign, "tool_ids")
    declared = campaign.get("threads_by_tool")
    if declared is not None:
        if campaign.get("threads") is not None:
            raise RegistryError(
                "benchmark campaign cannot mix threads and threads_by_tool"
            )
        if not isinstance(declared, dict) or set(declared) != set(tool_ids):
            raise RegistryError(
                "benchmark threads_by_tool must cover the exact tool closure"
            )
        if any(
            not isinstance(value, int) or isinstance(value, bool) or value < 1
            for value in declared.values()
        ):
            raise RegistryError("benchmark thread counts must be positive integers")
        return {tool: int(declared[tool]) for tool in tool_ids}
    threads = campaign.get("threads")
    if not isinstance(threads, int) or isinstance(threads, bool) or threads < 1:
        raise RegistryError("benchmark threads must be a positive integer")
    return {tool: threads for tool in tool_ids}


def validate_registry(
    registry: dict[str, Any], catalog: dict[str, Any]
) -> list[dict[str, Any]]:
    if registry.get("schema_version") != SCHEMA_VERSION:
        raise RegistryError("benchmark registry schema is unsupported")
    campaigns = registry.get("campaigns")
    if not isinstance(campaigns, list) or not campaigns:
        raise RegistryError("benchmark registry contains no campaigns")
    stage_tools = retained_stage_tools(catalog)
    seen_runs: set[str] = set()
    seen_stages: set[str] = set()
    validated: list[dict[str, Any]] = []
    for campaign in campaigns:
        if not isinstance(campaign, dict):
            raise RegistryError("benchmark campaign must be a mapping")
        run_id = require_text(campaign, "run_id")
        stage_id = require_text(campaign, "stage_id")
        slug = require_text(campaign, "slug")
        evidence_tier = require_text(campaign, "evidence_tier")
        if evidence_tier not in {"qualification", "comparative"}:
            raise RegistryError(
                "benchmark evidence_tier must be qualification or comparative"
            )
        require_text(campaign, "profile_id")
        require_text_list(campaign, "corpus_ids")
        if RUN_PATTERN.fullmatch(run_id) is None:
            raise RegistryError(f"benchmark run ID is outside run-2001+: {run_id}")
        expected_slug = re.sub(r"[^a-z0-9]+", "-", stage_id.lower()).strip("-")
        if slug != expected_slug:
            raise RegistryError(
                f"benchmark slug {slug!r} must derive from stage {stage_id!r}"
            )
        if run_id in seen_runs or stage_id in seen_stages:
            raise RegistryError("benchmark run IDs and stage IDs must be unique")
        seen_runs.add(run_id)
        seen_stages.add(stage_id)
        applicable_tools = stage_tools.get(stage_id)
        if not applicable_tools:
            raise RegistryError(f"benchmark stage has no retained tools: {stage_id}")
        declared_tools = set(require_text_list(campaign, "tool_ids"))
        if declared_tools != applicable_tools:
            missing = sorted(applicable_tools - declared_tools)
            extra = sorted(declared_tools - applicable_tools)
            raise RegistryError(
                f"benchmark tool closure failed for {stage_id}: "
                f"missing={missing}, extra={extra}"
            )
        for key in (
            "corpus_catalog_ids",
            "input_size_keys",
            "scientific_metrics",
            "limitations",
        ):
            require_text_list(campaign, key)
        require_text_list(campaign, "reference_catalog_ids", allow_empty=True)
        require_text_list(campaign, "database_catalog_ids", allow_empty=True)
        comparison_metrics = campaign.get("comparison_metrics")
        if not isinstance(comparison_metrics, list):
            raise RegistryError("benchmark comparison_metrics must be a list")
        if len(declared_tools) > 1 and not comparison_metrics:
            raise RegistryError(
                "multi-tool benchmarks require predeclared comparison metrics"
            )
        comparison_ids: set[str] = set()
        for comparison in comparison_metrics:
            if not isinstance(comparison, dict) or set(comparison) != {
                "id",
                "absolute_tolerance",
                "relative_tolerance",
            }:
                raise RegistryError("benchmark comparison metric is incomplete")
            comparison_id = str(comparison["id"])
            if (
                comparison_id not in campaign["scientific_metrics"]
                or comparison_id in comparison_ids
            ):
                raise RegistryError(
                    "benchmark comparison metric must name one unique scientific metric"
                )
            comparison_ids.add(comparison_id)
            for key in ("absolute_tolerance", "relative_tolerance"):
                value = comparison[key]
                if (
                    not isinstance(value, (int, float))
                    or isinstance(value, bool)
                    or value < 0
                ):
                    raise RegistryError(
                        f"benchmark comparison metric {key} must be non-negative"
                    )
        if any(
            re.fullmatch(r"[a-z][a-z0-9_]*_input_bytes", key) is None
            for key in campaign["input_size_keys"]
        ):
            raise RegistryError(
                "benchmark input_size_keys must name governed byte-count fields"
            )
        result_artifact = require_text(campaign, "result_artifact")
        if re.fullmatch(r"[a-z0-9][a-z0-9._-]*", result_artifact) is None:
            raise RegistryError("benchmark result_artifact must be a safe filename")
        performance_metrics = set(require_text_list(campaign, "performance_metrics"))
        if not REQUIRED_PERFORMANCE_METRICS.issubset(performance_metrics):
            raise RegistryError(
                f"benchmark performance metrics are incomplete for {stage_id}"
            )
        if (
            evidence_tier == "comparative"
            and not COMPARATIVE_PERFORMANCE_METRICS.issubset(performance_metrics)
        ):
            raise RegistryError(
                f"comparative performance metrics are incomplete for {stage_id}"
            )
        if campaign.get("runtime") != "apptainer":
            raise RegistryError("stage benchmarks must use Apptainer")
        if campaign.get("scheduler") != "slurm":
            raise RegistryError("stage benchmarks must use Slurm")
        if campaign.get("container_only") is not True:
            raise RegistryError("stage benchmarks must execute container-only tools")
        if campaign.get("cache_policy") not in {"cold", "warm"}:
            raise RegistryError("benchmark cache policy must be cold or warm")
        threads_by_tool(campaign)
        if campaign.get("warmup_repetitions", 0) < 1:
            raise RegistryError("benchmark campaigns require a warmup repetition")
        if campaign.get("measured_repetitions", 0) < 3:
            raise RegistryError(
                "benchmark campaigns require at least three measurements"
            )
        expected_root = "results/benchmarks/{run_id}-{slug}/{tool_id}"
        if campaign.get("result_root_template") != expected_root:
            raise RegistryError("benchmark result root template is not canonical")
        require_text(campaign, "scientific_question")
        require_text(campaign, "comparison_scope")
        require_text(campaign, "accuracy_claim")
        require_text(campaign, "interpretation")
        validated.append(campaign)
    return validated


def coverage_report(
    campaigns: list[dict[str, Any]],
    catalog: dict[str, Any],
    evidence_root: Path | None = None,
) -> dict[str, Any]:
    stage_tools = retained_stage_tools(catalog)
    defined = {str(campaign["stage_id"]): campaign for campaign in campaigns}
    rows = []
    for stage_id, tools in sorted(stage_tools.items()):
        campaign = defined.get(stage_id)
        execution_status = "not_defined"
        definition_status = "missing"
        if campaign:
            definition_status = str(campaign["evidence_tier"])
            execution_status = "not_admitted"
            if evidence_root is not None:
                evidence_path = (
                    evidence_root / f"{campaign['run_id']}-{campaign['slug']}.json"
                )
                if evidence_path.is_file():
                    try:
                        evidence = json.loads(evidence_path.read_text(encoding="utf-8"))
                    except (OSError, json.JSONDecodeError) as exc:
                        raise RegistryError(
                            f"cannot read benchmark evidence {evidence_path}: {exc}"
                        ) from exc
                    if (
                        not isinstance(evidence, dict)
                        or evidence.get("schema_version")
                        != "bijux.benchmark.stage_report.v1"
                        or evidence.get("verdict") != "passed"
                        or evidence.get("run_id") != campaign["run_id"]
                        or evidence.get("stage_id") != stage_id
                        or {
                            str(tool.get("tool"))
                            for tool in evidence.get("tools", [])
                            if isinstance(tool, dict)
                        }
                        != set(campaign["tool_ids"])
                    ):
                        raise RegistryError(
                            f"benchmark evidence identity is invalid: {evidence_path}"
                        )
                    execution_status = "admitted"
        rows.append(
            {
                "stage_id": stage_id,
                "retained_tools": sorted(tools),
                "campaign_run_id": campaign.get("run_id") if campaign else None,
                "definition_status": definition_status,
                "execution_status": execution_status,
            }
        )
    admitted = sum(row["execution_status"] == "admitted" for row in rows)
    qualification = sum(row["definition_status"] == "qualification" for row in rows)
    comparative = sum(row["definition_status"] == "comparative" for row in rows)
    return {
        "schema_version": "bijux.benchmark.stage_coverage.v1",
        "summary": {
            "retained_stage_count": len(rows),
            "defined_stage_count": len(defined),
            "qualification_stage_count": qualification,
            "comparative_stage_count": comparative,
            "admitted_stage_count": admitted,
            "missing_stage_count": len(rows) - len(defined),
        },
        "stages": rows,
    }


def write_report(
    report: dict[str, Any], json_path: Path, markdown_path: Path, check: bool = False
) -> None:
    expected_json = json.dumps(report, indent=2, sort_keys=True) + "\n"
    summary = report["summary"]
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: benchmarks/benchmark_registry.py -->",
        "# Stage benchmark coverage",
        "",
        "A qualification campaign proves container execution, retained-tool closure,",
        "and a reproducible measurement contract. It is not a comparative scientific",
        "benchmark. Comparative readiness is governed separately by the corpus portfolio,",
        "truth strategy, cohort size, output contract, and downstream substitution probes.",
        "",
        f"- Retained stages: **{summary['retained_stage_count']}**",
        f"- Defined campaigns: **{summary['defined_stage_count']}**",
        f"- Qualification-only campaigns: **{summary['qualification_stage_count']}**",
        f"- Comparative campaigns: **{summary['comparative_stage_count']}**",
        f"- Admitted campaigns: **{summary['admitted_stage_count']}**",
        f"- Missing campaigns: **{summary['missing_stage_count']}**",
        "",
        "| Stage | Retained tools | Campaign | Definition | Execution |",
        "|---|---|---|---|---|",
    ]
    for row in report["stages"]:
        tools = ", ".join(f"`{tool}`" for tool in row["retained_tools"])
        run_id = f"`{row['campaign_run_id']}`" if row["campaign_run_id"] else "—"
        lines.append(
            f"| `{row['stage_id']}` | {tools} | {run_id} | "
            f"{row['definition_status']} | {row['execution_status']} |"
        )
    expected_markdown = "\n".join(lines) + "\n"
    if check:
        if (
            not json_path.is_file()
            or json_path.read_text(encoding="utf-8") != expected_json
        ):
            raise RegistryError(f"stage benchmark coverage JSON drifted: {json_path}")
        if (
            not markdown_path.is_file()
            or markdown_path.read_text(encoding="utf-8") != expected_markdown
        ):
            raise RegistryError(
                f"stage benchmark coverage Markdown drifted: {markdown_path}"
            )
        return
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(expected_json, encoding="utf-8")
    markdown_path.write_text(expected_markdown, encoding="utf-8")


def resolve_evidence_root(
    evidence_root: Path | None,
    json_output: Path | None,
    markdown_output: Path | None,
) -> Path | None:
    if evidence_root is not None:
        return evidence_root
    if json_output is None or markdown_output is None:
        return None
    return (
        json_output.parent
        if json_output.parent == markdown_output.parent
        else None
    )


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--registry", type=Path, required=True)
    root.add_argument("--catalog", type=Path, required=True)
    root.add_argument("--json-output", type=Path)
    root.add_argument("--markdown-output", type=Path)
    root.add_argument("--evidence-root", type=Path)
    root.add_argument("--check", action="store_true")
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        registry = load_toml(args.registry)
        catalog = load_toml(args.catalog)
        campaigns = validate_registry(registry, catalog)
        if bool(args.json_output) != bool(args.markdown_output):
            raise RegistryError(
                "JSON and Markdown report paths must be supplied together"
            )
        if args.check and not (args.json_output and args.markdown_output):
            raise RegistryError("--check requires JSON and Markdown report paths")
        evidence_root = resolve_evidence_root(
            args.evidence_root,
            args.json_output,
            args.markdown_output,
        )
        report = coverage_report(campaigns, catalog, evidence_root)
        if args.json_output and args.markdown_output:
            write_report(
                report,
                args.json_output,
                args.markdown_output,
                check=args.check,
            )
        print(
            f"stage benchmark campaigns: {len(campaigns)} defined, "
            f"{report['summary']['missing_stage_count']} missing"
        )
        return 0
    except RegistryError as exc:
        print(f"stage benchmark registry failed: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
