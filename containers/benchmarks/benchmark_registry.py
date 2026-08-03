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

SCHEMA_VERSION = "bijux.container.stage_benchmarks.v1"
RUN_PATTERN = re.compile(r"^run-(2[0-9]{3}|[3-9][0-9]{3,})$")
REQUIRED_PERFORMANCE_METRICS = {
    "wall_seconds",
    "cpu_seconds",
    "peak_rss_kib",
    "input_bytes",
    "output_bytes",
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


def require_text_list(campaign: dict[str, Any], key: str) -> list[str]:
    value = campaign.get(key)
    if (
        not isinstance(value, list)
        or not value
        or any(not isinstance(item, str) or not item for item in value)
    ):
        raise RegistryError(
            f"benchmark campaign {key} must be a non-empty text list"
        )
    if len(value) != len(set(value)):
        raise RegistryError(f"benchmark campaign {key} contains duplicates")
    return value


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
            "reference_catalog_ids",
            "database_catalog_ids",
            "scientific_metrics",
            "limitations",
        ):
            require_text_list(campaign, key)
        performance_metrics = set(
            require_text_list(campaign, "performance_metrics")
        )
        if not REQUIRED_PERFORMANCE_METRICS.issubset(performance_metrics):
            raise RegistryError(
                f"benchmark performance metrics are incomplete for {stage_id}"
            )
        if campaign.get("runtime") != "apptainer":
            raise RegistryError("stage benchmarks must use Apptainer")
        if campaign.get("scheduler") != "slurm":
            raise RegistryError("stage benchmarks must use Slurm")
        if campaign.get("container_only") is not True:
            raise RegistryError("stage benchmarks must execute container-only tools")
        if campaign.get("cache_policy") not in {"cold", "warm"}:
            raise RegistryError("benchmark cache policy must be cold or warm")
        if not isinstance(campaign.get("threads"), int) or campaign["threads"] < 1:
            raise RegistryError("benchmark threads must be positive")
        if campaign.get("warmup_repetitions", 0) < 1:
            raise RegistryError("benchmark campaigns require a warmup repetition")
        if campaign.get("measured_repetitions", 0) < 3:
            raise RegistryError("benchmark campaigns require at least three measurements")
        expected_root = "results/benchmarks/{run_id}-{slug}/{tool_id}"
        if campaign.get("result_root_template") != expected_root:
            raise RegistryError("benchmark result root template is not canonical")
        require_text(campaign, "scientific_question")
        require_text(campaign, "comparison_scope")
        require_text(campaign, "accuracy_claim")
        validated.append(campaign)
    return validated


def coverage_report(
    campaigns: list[dict[str, Any]], catalog: dict[str, Any]
) -> dict[str, Any]:
    stage_tools = retained_stage_tools(catalog)
    defined = {str(campaign["stage_id"]): campaign for campaign in campaigns}
    rows = []
    for stage_id, tools in sorted(stage_tools.items()):
        campaign = defined.get(stage_id)
        rows.append(
            {
                "stage_id": stage_id,
                "retained_tools": sorted(tools),
                "campaign_run_id": campaign.get("run_id") if campaign else None,
                "definition_status": "defined" if campaign else "missing",
            }
        )
    return {
        "schema_version": "bijux.container.stage_benchmark_coverage.v1",
        "summary": {
            "retained_stage_count": len(rows),
            "defined_stage_count": len(defined),
            "missing_stage_count": len(rows) - len(defined),
        },
        "stages": rows,
    }


def write_report(report: dict[str, Any], json_path: Path, markdown_path: Path) -> None:
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    summary = report["summary"]
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: containers/benchmarks/benchmark_registry.py -->",
        "# Stage benchmark coverage",
        "",
        "A defined campaign has complete retained-tool closure and a reproducible",
        "measurement contract. It is not marked executed until Slurm evidence is",
        "admitted separately.",
        "",
        f"- Retained stages: **{summary['retained_stage_count']}**",
        f"- Defined campaigns: **{summary['defined_stage_count']}**",
        f"- Missing campaigns: **{summary['missing_stage_count']}**",
        "",
        "| Stage | Retained tools | Campaign | Definition |",
        "|---|---|---|---|",
    ]
    for row in report["stages"]:
        tools = ", ".join(f"`{tool}`" for tool in row["retained_tools"])
        run_id = f"`{row['campaign_run_id']}`" if row["campaign_run_id"] else "—"
        lines.append(
            f"| `{row['stage_id']}` | {tools} | {run_id} | "
            f"{row['definition_status']} |"
        )
    markdown_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--registry", type=Path, required=True)
    root.add_argument("--catalog", type=Path, required=True)
    root.add_argument("--json-output", type=Path)
    root.add_argument("--markdown-output", type=Path)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        registry = load_toml(args.registry)
        catalog = load_toml(args.catalog)
        campaigns = validate_registry(registry, catalog)
        report = coverage_report(campaigns, catalog)
        if bool(args.json_output) != bool(args.markdown_output):
            raise RegistryError("JSON and Markdown report paths must be supplied together")
        if args.json_output and args.markdown_output:
            write_report(report, args.json_output, args.markdown_output)
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
