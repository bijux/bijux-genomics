#!/usr/bin/env python3
"""Validate benchmark corpora, scientific profiles, and all-stage readiness."""

from __future__ import annotations

import argparse
import json
import re
from collections import defaultdict
from pathlib import Path
from typing import Any

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

PORTFOLIO_SCHEMA = "bijux.benchmark.portfolio.v1"
REGISTRY_SCHEMA = "bijux.benchmark.stage_campaigns.v1"
INTERFACE_SCHEMA = "bijux.benchmark.stage_interfaces.v1"
PARAMETER_CONFIG_SCHEMA = "bijux.benchmark.pipeline_parameters.v1"
REPORT_SCHEMA = "bijux.container.benchmark_readiness.v1"
CORPUS_STATUSES = {"missing", "source_defined", "admitted"}
EVIDENCE_SCOPES = {"qualification", "comparative"}
STAGE_RUN = re.compile(r"run-2[0-9]{3}")
REQUIRED_RESOURCE_METRICS = {
    "wall_seconds",
    "cpu_seconds",
    "cpu_utilization_percent",
    "peak_rss_kib",
    "filesystem_read_bytes",
    "filesystem_write_bytes",
    "input_bytes",
    "output_bytes",
    "throughput_input_bytes_per_second",
}
REQUIRED_QUALITY_DIMENSIONS = {
    "scientific_correctness",
    "repeatability",
    "robustness",
    "output_contract",
}


class PortfolioError(RuntimeError):
    """Raised when benchmark readiness is overstated or ambiguous."""


def load_toml(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise PortfolioError(f"cannot read TOML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise PortfolioError(f"TOML root must be a mapping: {path}")
    return value


def load_yaml(path: Path) -> dict[str, Any]:
    """Load the dependency-free JSON subset of YAML used by benchmark policy."""
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise PortfolioError(f"cannot read YAML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise PortfolioError(f"YAML root must be a mapping: {path}")
    return value


def text(value: Any, label: str) -> str:
    if not isinstance(value, str) or not value.strip():
        raise PortfolioError(f"{label} must be non-empty text")
    return value


def text_list(value: Any, label: str, *, allow_empty: bool = False) -> list[str]:
    if (
        not isinstance(value, list)
        or (not value and not allow_empty)
        or any(not isinstance(item, str) or not item for item in value)
    ):
        qualifier = "a text list" if allow_empty else "a non-empty text list"
        raise PortfolioError(f"{label} must be {qualifier}")
    if len(value) != len(set(value)):
        raise PortfolioError(f"{label} contains duplicates")
    return value


def retained_stage_tools(catalog: dict[str, Any]) -> dict[str, set[str]]:
    stages: dict[str, set[str]] = defaultdict(set)
    for tool in catalog.get("tools", []):
        if not isinstance(tool, dict) or tool.get("support_disposition") != "retained":
            continue
        tool_id = text(tool.get("id"), "catalog tool id")
        for stage_id in tool.get("stage_ids", []):
            stage = text(stage_id, f"{tool_id} stage id")
            if stage.split(".", 1)[0] in {"fastq", "bam", "vcf"}:
                stages[stage].add(tool_id)
    if not stages:
        raise PortfolioError("catalog contains no retained FASTQ, BAM, or VCF stages")
    return dict(stages)


def validate_measurement(value: Any) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise PortfolioError("portfolio measurement contract must be a mapping")
    if value.get("warmup_repetitions", 0) < 1:
        raise PortfolioError("benchmark measurement requires at least one warmup")
    if value.get("measured_repetitions", 0) < 3:
        raise PortfolioError("benchmark measurement requires at least three repetitions")
    resources = set(text_list(value.get("required_resource_metrics"), "resource metrics"))
    if not REQUIRED_RESOURCE_METRICS.issubset(resources):
        raise PortfolioError(
            "benchmark measurement omits required resource metrics: "
            f"{sorted(REQUIRED_RESOURCE_METRICS - resources)}"
        )
    quality = set(text_list(value.get("required_quality_dimensions"), "quality dimensions"))
    if not REQUIRED_QUALITY_DIMENSIONS.issubset(quality):
        raise PortfolioError(
            "benchmark measurement omits required quality dimensions: "
            f"{sorted(REQUIRED_QUALITY_DIMENSIONS - quality)}"
        )
    return value


def validate_corpora(
    values: Any, repository_root: Path
) -> tuple[dict[str, dict[str, Any]], dict[str, list[str]]]:
    if not isinstance(values, list) or not values:
        raise PortfolioError("benchmark portfolio contains no corpora")
    corpora: dict[str, dict[str, Any]] = {}
    by_role: dict[str, list[str]] = defaultdict(list)
    for corpus in values:
        if not isinstance(corpus, dict):
            raise PortfolioError("benchmark corpus must be a mapping")
        corpus_id = text(corpus.get("id"), "corpus id")
        if corpus_id in corpora:
            raise PortfolioError(f"duplicate benchmark corpus: {corpus_id}")
        status = corpus.get("status")
        scope = corpus.get("evidence_scope")
        if status not in CORPUS_STATUSES or scope not in EVIDENCE_SCOPES:
            raise PortfolioError(f"benchmark corpus status is invalid: {corpus_id}")
        if corpus.get("real_data") is not True:
            raise PortfolioError(f"benchmark corpus must declare real data: {corpus_id}")
        sample_count = corpus.get("sample_count")
        if not isinstance(sample_count, int) or isinstance(sample_count, bool) or sample_count < 0:
            raise PortfolioError(f"benchmark corpus sample count is invalid: {corpus_id}")
        minimum = corpus.get("minimum_sample_count", sample_count)
        if not isinstance(minimum, int) or isinstance(minimum, bool) or minimum < 1:
            raise PortfolioError(f"benchmark corpus minimum sample count is invalid: {corpus_id}")
        if status == "admitted" and sample_count < minimum:
            raise PortfolioError(f"admitted corpus is below its minimum size: {corpus_id}")
        roles = text_list(corpus.get("roles"), f"{corpus_id} roles")
        text_list(corpus.get("data_types"), f"{corpus_id} data types")
        text(corpus.get("truth_strategy"), f"{corpus_id} truth strategy")
        text(corpus.get("scientific_rationale"), f"{corpus_id} scientific rationale")
        text_list(corpus.get("limitations"), f"{corpus_id} limitations")
        source_specs = text_list(
            corpus.get("source_specs", []), f"{corpus_id} source specs", allow_empty=True
        )
        catalog_ids = text_list(
            corpus.get("catalog_ids", []), f"{corpus_id} catalog ids", allow_empty=True
        )
        if status == "source_defined" and not source_specs:
            raise PortfolioError(f"source-defined corpus has no source spec: {corpus_id}")
        if status == "admitted" and not catalog_ids:
            raise PortfolioError(f"admitted corpus has no data catalog identity: {corpus_id}")
        for source in source_specs:
            path = repository_root / source
            if not path.is_file():
                raise PortfolioError(f"corpus source spec is absent: {source}")
        corpora[corpus_id] = corpus
        for role in roles:
            by_role[role].append(corpus_id)
    return corpora, dict(by_role)


def validate_profiles(
    values: Any, roles: dict[str, list[str]]
) -> dict[str, dict[str, Any]]:
    if not isinstance(values, list) or not values:
        raise PortfolioError("benchmark portfolio contains no scientific profiles")
    profiles: dict[str, dict[str, Any]] = {}
    for profile in values:
        if not isinstance(profile, dict):
            raise PortfolioError("benchmark profile must be a mapping")
        profile_id = text(profile.get("id"), "benchmark profile id")
        if profile_id in profiles:
            raise PortfolioError(f"duplicate benchmark profile: {profile_id}")
        required_roles = text_list(
            profile.get("required_corpus_roles"), f"{profile_id} required corpus roles"
        )
        unknown = sorted(set(required_roles) - set(roles))
        if unknown:
            raise PortfolioError(f"benchmark profile has unknown corpus roles: {unknown}")
        minimum = profile.get("minimum_real_samples")
        if not isinstance(minimum, int) or isinstance(minimum, bool) or minimum < 10:
            raise PortfolioError(
                f"comparative profile requires at least ten real samples: {profile_id}"
            )
        for key in ("accuracy_strategy", "interchangeability", "downstream_probe"):
            text(profile.get(key), f"{profile_id} {key}")
        profiles[profile_id] = profile
    return profiles


def validate_assignments(
    values: Any,
    profiles: dict[str, dict[str, Any]],
    stage_tools: dict[str, set[str]],
) -> dict[str, str]:
    if not isinstance(values, list) or not values:
        raise PortfolioError("benchmark portfolio contains no stage assignments")
    assigned: dict[str, str] = {}
    for assignment in values:
        if not isinstance(assignment, dict):
            raise PortfolioError("benchmark assignment must be a mapping")
        profile_id = text(assignment.get("profile_id"), "assignment profile id")
        if profile_id not in profiles:
            raise PortfolioError(f"assignment uses unknown profile: {profile_id}")
        for stage_id in text_list(assignment.get("stage_ids"), f"{profile_id} stages"):
            if stage_id in assigned:
                raise PortfolioError(f"stage has multiple benchmark profiles: {stage_id}")
            assigned[stage_id] = profile_id
    missing = sorted(set(stage_tools) - set(assigned))
    extra = sorted(set(assigned) - set(stage_tools))
    if missing or extra:
        raise PortfolioError(
            f"benchmark profile closure failed: missing={missing}, extra={extra}"
        )
    return assigned


def validate_stage_runs(value: Any, stages: set[str]) -> dict[str, str]:
    if not isinstance(value, dict):
        raise PortfolioError("benchmark stage run registry must be a mapping")
    missing = sorted(stages - set(value))
    extra = sorted(set(value) - stages)
    if missing or extra:
        raise PortfolioError(
            f"benchmark run identity closure failed: missing={missing}, extra={extra}"
        )
    run_ids: dict[str, str] = {}
    for stage_id, run_id in value.items():
        run = text(run_id, f"{stage_id} benchmark run id")
        if STAGE_RUN.fullmatch(run) is None:
            raise PortfolioError(f"benchmark run id is outside run-2000..2999: {run}")
        if run in run_ids.values():
            raise PortfolioError(f"benchmark run id is assigned more than once: {run}")
        run_ids[str(stage_id)] = run
    return run_ids


def validate_portfolio(
    portfolio: dict[str, Any], catalog: dict[str, Any], repository_root: Path
) -> dict[str, Any]:
    if portfolio.get("schema_version") != PORTFOLIO_SCHEMA:
        raise PortfolioError("benchmark portfolio schema is unsupported")
    measurement = validate_measurement(portfolio.get("measurement"))
    corpora, roles = validate_corpora(portfolio.get("corpora"), repository_root)
    profiles = validate_profiles(portfolio.get("profiles"), roles)
    stage_tools = retained_stage_tools(catalog)
    assignments = validate_assignments(
        portfolio.get("assignments"), profiles, stage_tools
    )
    stage_runs = validate_stage_runs(portfolio.get("stage_runs"), set(stage_tools))
    return {
        "measurement": measurement,
        "corpora": corpora,
        "roles": roles,
        "profiles": profiles,
        "stage_tools": stage_tools,
        "assignments": assignments,
        "stage_runs": stage_runs,
    }


def campaign_index(registry: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if registry.get("schema_version") != REGISTRY_SCHEMA:
        raise PortfolioError("benchmark campaign registry schema is unsupported")
    result: dict[str, dict[str, Any]] = {}
    for campaign in registry.get("campaigns", []):
        if not isinstance(campaign, dict):
            raise PortfolioError("benchmark campaign must be a mapping")
        stage_id = text(campaign.get("stage_id"), "campaign stage id")
        if stage_id in result:
            raise PortfolioError(f"stage has multiple active campaigns: {stage_id}")
        result[stage_id] = campaign
    return result


def validate_interfaces(
    registry: dict[str, Any],
    stage_tools: dict[str, set[str]],
    repository_root: Path,
) -> dict[str, dict[str, Any]]:
    """Validate executable stage boundaries without inventing missing contracts."""
    if registry.get("schema_version") != INTERFACE_SCHEMA:
        raise PortfolioError("benchmark stage-interface schema is unsupported")
    result: dict[str, dict[str, Any]] = {}
    for interface in registry.get("interfaces", []):
        if not isinstance(interface, dict):
            raise PortfolioError("benchmark stage interface must be a mapping")
        stage_id = text(interface.get("stage_id"), "interface stage id")
        if stage_id in result:
            raise PortfolioError(f"stage has multiple interfaces: {stage_id}")
        if stage_id not in stage_tools:
            raise PortfolioError(f"interface uses an unretained stage: {stage_id}")
        accepted_inputs = text_list(
            interface.get("accepted_input_contracts"),
            f"{stage_id} accepted input contracts",
        )
        if not accepted_inputs:
            raise PortfolioError(f"stage interface has no accepted inputs: {stage_id}")
        for key in (
            "output_contract",
            "parameter_schema_version",
            "default_provider",
        ):
            text(interface.get(key), f"{stage_id} {key}")
        for key in ("normalized_result_contract", "parameter_config"):
            relative = Path(text(interface.get(key), f"{stage_id} {key}"))
            if relative.is_absolute() or ".." in relative.parts:
                raise PortfolioError(f"{stage_id} {key} must be repository-relative")
            if not (repository_root / relative).is_file():
                raise PortfolioError(f"{stage_id} {key} is absent: {relative}")
        parameter_config = load_yaml(repository_root / interface["parameter_config"])
        if parameter_config.get("schema_version") != PARAMETER_CONFIG_SCHEMA:
            raise PortfolioError(f"{stage_id} parameter config schema is unsupported")
        configured_stages = parameter_config.get("stages")
        if not isinstance(configured_stages, dict) or stage_id not in configured_stages:
            raise PortfolioError(f"{stage_id} parameter config lacks its stage entry")
        parameter_entry = configured_stages[stage_id]
        if (
            not isinstance(parameter_entry, dict)
            or parameter_entry.get("parameter_schema_version")
            != interface["parameter_schema_version"]
            or parameter_entry.get("defaults_provider") != interface["default_provider"]
            or not isinstance(parameter_entry.get("overrides"), dict)
        ):
            raise PortfolioError(f"{stage_id} parameter config disagrees with its interface")
        declared_tools = set(text_list(interface.get("tool_ids"), f"{stage_id} tools"))
        if declared_tools != stage_tools[stage_id]:
            raise PortfolioError(
                f"stage interface tool closure failed for {stage_id}: "
                f"missing={sorted(stage_tools[stage_id] - declared_tools)}, "
                f"extra={sorted(declared_tools - stage_tools[stage_id])}"
            )
        adapters = interface.get("adapters")
        if not isinstance(adapters, dict) or set(adapters) != declared_tools:
            raise PortfolioError(
                f"stage interface adapters must cover the exact tool closure: {stage_id}"
            )
        for tool_id, adapter in adapters.items():
            text(adapter, f"{stage_id} {tool_id} adapter")
        sources = text_list(
            interface.get("provenance_sources"), f"{stage_id} provenance sources"
        )
        for source in sources:
            path = Path(source)
            if (
                path.is_absolute()
                or ".." in path.parts
                or not (repository_root / path).is_file()
            ):
                raise PortfolioError(f"{stage_id} provenance source is absent: {source}")
        probe = interface.get("downstream_probe")
        if not isinstance(probe, dict):
            raise PortfolioError(f"stage interface lacks downstream probe: {stage_id}")
        for key in ("consumer_stage_id", "input_contract", "source"):
            text(probe.get(key), f"{stage_id} downstream {key}")
        text_list(probe.get("assertions"), f"{stage_id} downstream assertions")
        if probe["input_contract"] != interface["output_contract"]:
            raise PortfolioError(
                f"downstream probe input differs from canonical output: {stage_id}"
            )
        result[stage_id] = interface
    return result


def readiness_report(
    validated: dict[str, Any],
    registry: dict[str, Any],
    interfaces: dict[str, dict[str, Any]] | None = None,
    evidence_root: Path | None = None,
) -> dict[str, Any]:
    campaigns = campaign_index(registry)
    rows = []
    for stage_id, tools in sorted(validated["stage_tools"].items()):
        profile_id = validated["assignments"][stage_id]
        profile = validated["profiles"][profile_id]
        blockers = []
        interface = (interfaces or {}).get(stage_id)
        if interface is None:
            blockers.append("no closed stage interface is defined")
        available_samples: set[tuple[str, int]] = set()
        for role in profile["required_corpus_roles"]:
            candidates = [
                validated["corpora"][corpus_id]
                for corpus_id in validated["roles"][role]
            ]
            ready = [
                corpus
                for corpus in candidates
                if corpus["status"] == "admitted"
                and corpus["evidence_scope"] == "comparative"
            ]
            for corpus in ready:
                available_samples.add(
                    (str(corpus["id"]), int(corpus["sample_count"]))
                )
            if not ready:
                available = ", ".join(
                    f"{corpus['id']}={corpus['status']}/{corpus['evidence_scope']}"
                    for corpus in candidates
                )
                blockers.append(f"corpus role {role} is not comparative-ready ({available})")
        available_real_samples = sum(count for _, count in available_samples)
        if available_real_samples < profile["minimum_real_samples"]:
            blockers.append(
                f"available comparative sample count {available_real_samples} is below "
                f"{profile['minimum_real_samples']}"
            )
        campaign = campaigns.get(stage_id)
        reserved_run_id = validated["stage_runs"][stage_id]
        if campaign is None:
            blockers.append("no execution campaign is defined")
            campaign_tier = None
            run_id = None
        else:
            campaign_tier = campaign.get("evidence_tier")
            run_id = campaign.get("run_id")
            if run_id != reserved_run_id:
                blockers.append(
                    f"campaign run id {run_id} does not match reserved {reserved_run_id}"
                )
            if campaign.get("profile_id") != profile_id:
                blockers.append("campaign does not bind the assigned scientific profile")
            if campaign_tier != "comparative":
                blockers.append("active campaign is qualification-only")
            declared_corpora = campaign.get("corpus_ids")
            if not isinstance(declared_corpora, list) or not declared_corpora:
                blockers.append("campaign does not declare its benchmark corpus portfolio")
            elif campaign_tier == "comparative":
                unknown_corpora = sorted(
                    set(str(item) for item in declared_corpora)
                    - set(validated["corpora"])
                )
                if unknown_corpora:
                    blockers.append(
                        f"campaign declares unknown corpora {unknown_corpora}"
                    )
                declared_records = [
                    validated["corpora"][str(corpus_id)]
                    for corpus_id in declared_corpora
                    if str(corpus_id) in validated["corpora"]
                ]
                invalid_corpora = [
                    str(corpus["id"])
                    for corpus in declared_records
                    if corpus["status"] != "admitted"
                    or corpus["evidence_scope"] != "comparative"
                ]
                if invalid_corpora:
                    blockers.append(
                        "campaign corpora are not comparative-ready: "
                        f"{invalid_corpora}"
                    )
                declared_roles = {
                    str(role)
                    for corpus in declared_records
                    for role in corpus["roles"]
                }
                missing_roles = sorted(
                    set(profile["required_corpus_roles"]) - declared_roles
                )
                if missing_roles:
                    blockers.append(
                        f"campaign corpus portfolio omits roles {missing_roles}"
                    )
                declared_samples = sum(
                    int(corpus["sample_count"])
                    for corpus in declared_records
                    if corpus["status"] == "admitted"
                    and corpus["evidence_scope"] == "comparative"
                )
                if declared_samples < profile["minimum_real_samples"]:
                    blockers.append(
                        f"campaign real sample count {declared_samples} is below "
                        f"{profile['minimum_real_samples']}"
                    )
            if campaign_tier == "comparative":
                evidence_path = (
                    evidence_root / f"{run_id}-{campaign.get('slug')}.json"
                    if evidence_root is not None and run_id is not None
                    else None
                )
                if evidence_path is None or not evidence_path.is_file():
                    blockers.append("no admitted comparative execution evidence")
        rows.append(
            {
                "stage_id": stage_id,
                "domain": stage_id.split(".", 1)[0],
                "profile_id": profile_id,
                "retained_tools": sorted(tools),
                "available_comparative_samples": available_real_samples,
                "campaign_run_id": run_id,
                "reserved_run_id": reserved_run_id,
                "campaign_tier": campaign_tier,
                "interface_status": "closed" if interface is not None else "missing",
                "readiness": "ready" if not blockers else "blocked",
                "blockers": blockers,
            }
        )
    by_domain: dict[str, dict[str, int]] = {}
    for domain in ("fastq", "bam", "vcf"):
        domain_rows = [row for row in rows if row["domain"] == domain]
        by_domain[domain] = {
            "stage_count": len(domain_rows),
            "ready_count": sum(row["readiness"] == "ready" for row in domain_rows),
            "blocked_count": sum(row["readiness"] == "blocked" for row in domain_rows),
        }
    return {
        "schema_version": REPORT_SCHEMA,
        "summary": {
            "stage_count": len(rows),
            "reserved_run_count": len(validated["stage_runs"]),
            "ready_count": sum(row["readiness"] == "ready" for row in rows),
            "blocked_count": sum(row["readiness"] == "blocked" for row in rows),
            "corpus_count": len(validated["corpora"]),
            "admitted_comparative_corpus_count": sum(
                corpus["status"] == "admitted"
                and corpus["evidence_scope"] == "comparative"
                for corpus in validated["corpora"].values()
            ),
            "qualification_campaign_count": sum(
                campaign.get("evidence_tier") == "qualification"
                for campaign in campaigns.values()
            ),
            "comparative_campaign_count": sum(
                campaign.get("evidence_tier") == "comparative"
                for campaign in campaigns.values()
            ),
        },
        "domains": by_domain,
        "measurement": validated["measurement"],
        "corpora": [
            {
                "id": corpus["id"],
                "status": corpus["status"],
                "evidence_scope": corpus["evidence_scope"],
                "sample_count": corpus["sample_count"],
                "truth_strategy": corpus["truth_strategy"],
                "roles": corpus["roles"],
            }
            for corpus in validated["corpora"].values()
        ],
        "stages": rows,
    }


def write_report(report: dict[str, Any], json_path: Path, markdown_path: Path) -> None:
    json_path.parent.mkdir(parents=True, exist_ok=True)
    json_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n")
    summary = report["summary"]
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: benchmarks/benchmark_portfolio.py -->",
        "# Scientific benchmark readiness",
        "",
        "This report refuses to treat container qualification, one-sample timing,",
        "or cross-tool agreement as a complete scientific benchmark. A stage is ready",
        "only when its real corpus roles, minimum cohort, accuracy strategy, complete",
        "tool closure, canonical output contract, and downstream substitution probe",
        "are all represented by a comparative campaign.",
        "",
        f"- Retained FASTQ/BAM/VCF stages: **{summary['stage_count']}**",
        f"- Scientifically ready stages: **{summary['ready_count']}**",
        f"- Blocked stages: **{summary['blocked_count']}**",
        f"- Comparative-ready corpora: **{summary['admitted_comparative_corpus_count']}**",
        f"- Qualification-only campaigns: **{summary['qualification_campaign_count']}**",
        f"- Comparative campaigns: **{summary['comparative_campaign_count']}**",
        "",
        "## Domain coverage",
        "",
        "| Domain | Stages | Ready | Blocked |",
        "|---|---:|---:|---:|",
    ]
    for domain, values in report["domains"].items():
        lines.append(
            f"| {domain.upper()} | {values['stage_count']} | "
            f"{values['ready_count']} | {values['blocked_count']} |"
        )
    lines.extend(
        [
            "",
            "## Corpus portfolio",
            "",
            "| Corpus | Status | Scope | Samples | Truth strategy |",
            "|---|---|---|---:|---|",
        ]
    )
    for corpus in report["corpora"]:
        lines.append(
            f"| `{corpus['id']}` | {corpus['status']} | {corpus['evidence_scope']} "
            f"| {corpus['sample_count']} | `{corpus['truth_strategy']}` |"
        )
    lines.extend(
        [
            "",
            "## Stage readiness",
            "",
            "| Stage | Tools | Profile | Campaign | Readiness | Principal blockers |",
            "|---|---|---|---|---|---|",
        ]
    )
    for row in report["stages"]:
        tools = ", ".join(f"`{tool}`" for tool in row["retained_tools"])
        campaign = f"`{row['reserved_run_id']}`"
        if row["campaign_run_id"]:
            campaign += f" ({row['campaign_tier']})"
        blockers = "<br>".join(row["blockers"]) if row["blockers"] else "—"
        lines.append(
            f"| `{row['stage_id']}` | {tools} | `{row['profile_id']}` | "
            f"{campaign} | {row['readiness']} | {blockers} |"
        )
    markdown_path.write_text("\n".join(lines) + "\n", encoding="utf-8")


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--portfolio", type=Path, required=True)
    value.add_argument("--registry", type=Path, required=True)
    value.add_argument("--interfaces", type=Path, required=True)
    value.add_argument("--catalog", type=Path, required=True)
    value.add_argument("--repository-root", type=Path, required=True)
    value.add_argument("--json-output", type=Path, required=True)
    value.add_argument("--markdown-output", type=Path, required=True)
    value.add_argument("--evidence-root", type=Path)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        validated = validate_portfolio(
            load_toml(args.portfolio),
            load_toml(args.catalog),
            args.repository_root.resolve(),
        )
        interfaces = validate_interfaces(
            load_toml(args.interfaces), validated["stage_tools"], args.repository_root.resolve()
        )
        report = readiness_report(
            validated,
            load_toml(args.registry),
            interfaces,
            args.evidence_root,
        )
        write_report(report, args.json_output, args.markdown_output)
        print(
            "scientific benchmark readiness: "
            f"{report['summary']['ready_count']} ready, "
            f"{report['summary']['blocked_count']} blocked"
        )
        return 0
    except PortfolioError as exc:
        print(f"scientific benchmark readiness failed: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
