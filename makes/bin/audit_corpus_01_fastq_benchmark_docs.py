#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
from collections import defaultdict
from dataclasses import asdict, dataclass
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    CORPUS_01_PUBLICATION_CONTRACTS,
    CORPUS_01_PUBLICATION_EXCLUSIONS,
    CorpusBenchmarkContract,
    CorpusBenchmarkExclusion,
    corpus_01_make_report_target,
    expected_cohort_counts,
    load_corpus_spec,
    resolve_benchmark_tool_roster,
    resolve_stage_toolset,
)


@dataclass(frozen=True)
class StageAuditIssue:
    stage_id: str
    issue_id: str
    severity: str
    detail: str


REQUIRED_STAGE_FILES = [
    "summary.json",
    "sample_results.csv",
    "tool_runtime_summary.csv",
    "cohort_runtime_summary.csv",
    "sample_runtime_outliers.csv",
    "lunarc.md",
]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit corpus-01 FASTQ benchmark publication completeness under docs/benchmark."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root that contains configs/runtime/corpora/corpus-01.toml.",
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark",
        help="Benchmark docs root.",
    )
    parser.add_argument(
        "--json-out",
        default="docs/benchmark/corpus-01-status.json",
        help="Machine-readable audit output path.",
    )
    parser.add_argument(
        "--markdown-out",
        default="docs/benchmark/corpus-01-status.md",
        help="Markdown audit output path.",
    )
    parser.add_argument(
        "--findings-in",
        default="docs/benchmark/corpus-01-publication-findings.json",
        help="Supplemental stage findings to surface alongside automated publication audit issues.",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_csv_rows(path: Path) -> list[dict]:
    with path.open("r", encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def expected_counts_for_scope(spec: dict, sample_scope: str) -> tuple[int, dict[str, int]]:
    full_counts = expected_cohort_counts(spec)
    if sample_scope == "full":
        return sum(full_counts.values()), full_counts
    if sample_scope == "paired":
        paired_counts = {
            "ancient_pe": int(spec["target_ancient_pe"]),
            "modern_pe": int(spec["target_modern_pe"]),
        }
        return sum(paired_counts.values()), paired_counts
    raise SystemExit(f"unsupported corpus publication sample_scope: {sample_scope}")


def append_issue(
    issues: list[StageAuditIssue],
    stage_id: str,
    issue_id: str,
    detail: str,
    *,
    severity: str = "error",
) -> None:
    issues.append(
        StageAuditIssue(
            stage_id=stage_id,
            issue_id=issue_id,
            severity=severity,
            detail=detail,
        )
    )


def makefile_publication_warnings(
    repo_root: Path,
    stage_contracts: list[CorpusBenchmarkContract],
) -> list[str]:
    makefile_path = repo_root / "makes" / "benchmarks-fastq.mk"
    if not makefile_path.is_file():
        return [f"missing benchmark makefile: {makefile_path}"]
    makefile_text = makefile_path.read_text(encoding="utf-8")
    missing_targets = [
        corpus_01_make_report_target(contract.stage_id)
        for contract in stage_contracts
        if f"{corpus_01_make_report_target(contract.stage_id)}:" not in makefile_text
    ]
    if not missing_targets:
        return []
    return [
        "benchmark makefile omits governed publication targets: "
        + ", ".join(missing_targets)
    ]


def load_supplemental_findings(path: Path) -> tuple[dict[str, list[StageAuditIssue]], list[str], str | None]:
    if not path.is_file():
        return {}, [f"missing supplemental findings file: {path}"], None
    payload = json.loads(path.read_text(encoding="utf-8"))
    warnings: list[str] = []
    generated_at_utc = payload.get("generated_at_utc")
    if not isinstance(generated_at_utc, str) or not generated_at_utc.strip():
        warnings.append(
            f"supplemental findings freshness is untracked in {path}; add generated_at_utc"
        )
        generated_at_utc = None
    findings_by_stage: dict[str, list[StageAuditIssue]] = defaultdict(list)
    for finding in payload.get("findings", []):
        stage_id = str(finding.get("stage_id", "")).strip()
        issue_id = str(finding.get("issue_id", "")).strip()
        detail = str(finding.get("detail", "")).strip()
        if not stage_id or not issue_id or not detail:
            raise SystemExit(
                f"invalid supplemental finding in {path}: stage_id, issue_id, and detail are required"
            )
        findings_by_stage[stage_id].append(
            StageAuditIssue(
                stage_id=stage_id,
                issue_id=issue_id,
                severity=str(finding.get("severity", "error")).strip() or "error",
                detail=detail,
            )
        )
    return findings_by_stage, warnings, generated_at_utc


def audit_stage(
    repo_root: Path,
    docs_root: Path,
    contract: CorpusBenchmarkContract,
    supplemental_issues: list[StageAuditIssue] | None = None,
) -> dict:
    spec = load_corpus_spec(repo_root)
    expected_total, expected_cohort_counts_by_scope = expected_counts_for_scope(
        spec,
        contract.sample_scope,
    )
    stage_root = docs_root / contract.stage_id
    method_path = stage_root / "corpus-01-method.md"
    corpus_root = stage_root / "corpus-01"
    issues: list[StageAuditIssue] = []
    expected_tools, roster_resolution_error = resolve_benchmark_tool_roster(
        repo_root,
        contract.stage_id,
        contract.scenario_id,
        contract.tools,
    )
    stage_toolset, stage_toolset_error = resolve_stage_toolset(
        repo_root,
        contract.stage_id,
        contract.tools,
    )

    if roster_resolution_error is not None:
        append_issue(
            issues,
            contract.stage_id,
            "benchmark-tool-roster-unresolved",
            roster_resolution_error,
        )
    elif sorted(expected_tools) != sorted(contract.tools):
        append_issue(
            issues,
            contract.stage_id,
            "contract-tool-roster-drift",
            (
                f"contract tools={sorted(contract.tools)!r} expected governed benchmark "
                f"roster {sorted(expected_tools)!r}"
            ),
        )
    if stage_toolset_error is not None:
        append_issue(
            issues,
            contract.stage_id,
            "stage-toolset-unresolved",
            stage_toolset_error,
        )
    else:
        uncovered_stage_tools = sorted(
            tool for tool in stage_toolset if tool not in contract.tools
        )
        if uncovered_stage_tools:
            append_issue(
                issues,
                contract.stage_id,
                "publication-toolset-subset",
                (
                    f"{contract.stage_id} publication contract covers {sorted(contract.tools)!r} "
                    f"but governed stage toolset also admits {uncovered_stage_tools!r}"
                ),
            )

    if not method_path.is_file():
        append_issue(
            issues,
            contract.stage_id,
            "missing-method-doc",
            f"missing {method_path.relative_to(docs_root.parent)}",
        )

    if not corpus_root.is_dir():
        append_issue(
            issues,
            contract.stage_id,
            "missing-corpus-dir",
            f"missing {corpus_root.relative_to(docs_root.parent)}",
        )
    else:
        for file_name in REQUIRED_STAGE_FILES:
            artifact_path = corpus_root / file_name
            if not artifact_path.is_file():
                append_issue(
                    issues,
                    contract.stage_id,
                    f"missing-{file_name.replace('.', '-')}",
                    f"missing {artifact_path.relative_to(docs_root.parent)}",
                )
                continue
            if artifact_path.stat().st_size == 0:
                append_issue(
                    issues,
                    contract.stage_id,
                    f"empty-{file_name.replace('.', '-')}",
                    f"empty {artifact_path.relative_to(docs_root.parent)}",
                )

        summary_path = corpus_root / "summary.json"
        sample_results_path = corpus_root / "sample_results.csv"
        tool_runtime_summary_path = corpus_root / "tool_runtime_summary.csv"
        cohort_runtime_summary_path = corpus_root / "cohort_runtime_summary.csv"
        sample_runtime_outliers_path = corpus_root / "sample_runtime_outliers.csv"

        if summary_path.is_file() and summary_path.stat().st_size > 0:
            summary = load_json(summary_path)
            if summary.get("stage_id") != contract.stage_id:
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-stage-id-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} stage_id="
                        f"{summary.get('stage_id')!r}"
                    ),
                )
            if summary.get("scenario_id") != contract.scenario_id:
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-scenario-id-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} scenario_id="
                        f"{summary.get('scenario_id')!r}"
                    ),
                )
            actual_tools = summary.get("tools")
            if sorted(actual_tools or []) != sorted(expected_tools):
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-tool-roster-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} tools="
                        f"{actual_tools!r} expected {expected_tools!r}"
                    ),
                )
            if int(summary.get("samples_total", 0) or 0) != expected_total:
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-sample-count-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} samples_total="
                        f"{summary.get('samples_total')!r} expected {expected_total}"
                    ),
                )
            if int(summary.get("samples_failed", 0) or 0) != 0:
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-sample-failures",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} samples_failed="
                        f"{summary.get('samples_failed')!r}"
                    ),
                )
            if dict(sorted((summary.get("cohort_counts") or {}).items())) != dict(
                sorted(expected_cohort_counts_by_scope.items())
            ):
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-cohort-count-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} cohort_counts="
                        f"{summary.get('cohort_counts')!r} expected "
                        f"{expected_cohort_counts_by_scope!r}"
                    ),
                )
            summary_tool_rows = summary.get("tool_summary") or []
            summary_tool_ids = sorted(
                {
                    row.get("tool")
                    for row in summary_tool_rows
                    if isinstance(row, dict) and row.get("tool")
                }
            )
            if summary_tool_ids != sorted(expected_tools):
                append_issue(
                    issues,
                    contract.stage_id,
                    "summary-tool-summary-drift",
                    (
                        f"{summary_path.relative_to(docs_root.parent)} tool_summary tools="
                        f"{summary_tool_ids!r} expected {sorted(expected_tools)!r}"
                    ),
                )

        if sample_results_path.is_file() and sample_results_path.stat().st_size > 0:
            sample_rows = load_csv_rows(sample_results_path)
            if not sample_rows:
                append_issue(
                    issues,
                    contract.stage_id,
                    "empty-sample-results-rows",
                    f"no CSV rows in {sample_results_path.relative_to(docs_root.parent)}",
                )
            else:
                per_sample_tools: dict[str, list[str]] = defaultdict(list)
                sample_metadata: dict[str, tuple[str, str, str, str, str]] = {}
                cohort_counts_by_rows: dict[str, int] = defaultdict(int)
                observed_tools: set[str] = set()

                for row in sample_rows:
                    sample_id = row.get("sample_id", "").strip()
                    tool = row.get("tool", "").strip()
                    if not sample_id or not tool:
                        append_issue(
                            issues,
                            contract.stage_id,
                            "sample-results-missing-sample-or-tool",
                            f"invalid row in {sample_results_path.relative_to(docs_root.parent)}",
                        )
                        continue
                    observed_tools.add(tool)
                    per_sample_tools[sample_id].append(tool)

                    metadata_tuple = (
                        row.get("accession", "").strip(),
                        row.get("era", "").strip(),
                        row.get("layout", "").strip(),
                        row.get("study_accession", "").strip(),
                        row.get("size_band", "").strip(),
                    )
                    if sample_id not in sample_metadata:
                        sample_metadata[sample_id] = metadata_tuple
                        cohort_key = f"{metadata_tuple[1]}_{metadata_tuple[2]}"
                        cohort_counts_by_rows[cohort_key] += 1
                    elif sample_metadata[sample_id] != metadata_tuple:
                        append_issue(
                            issues,
                            contract.stage_id,
                            "sample-results-metadata-drift",
                            (
                                f"{sample_results_path.relative_to(docs_root.parent)} "
                                f"sample {sample_id} metadata differs across rows"
                            ),
                        )

                if sorted(observed_tools) != sorted(expected_tools):
                    append_issue(
                        issues,
                        contract.stage_id,
                        "sample-results-tool-roster-drift",
                        (
                            f"{sample_results_path.relative_to(docs_root.parent)} tools="
                            f"{sorted(observed_tools)!r} expected {sorted(expected_tools)!r}"
                        ),
                    )
                if len(sample_metadata) != expected_total:
                    append_issue(
                        issues,
                        contract.stage_id,
                        "sample-results-sample-count-drift",
                        (
                            f"{sample_results_path.relative_to(docs_root.parent)} unique_samples="
                            f"{len(sample_metadata)!r} expected {expected_total}"
                        ),
                    )
                if dict(sorted(cohort_counts_by_rows.items())) != dict(
                    sorted(expected_cohort_counts_by_scope.items())
                ):
                    append_issue(
                        issues,
                        contract.stage_id,
                        "sample-results-cohort-count-drift",
                        (
                            f"{sample_results_path.relative_to(docs_root.parent)} cohort_counts="
                            f"{dict(sorted(cohort_counts_by_rows.items()))!r} expected "
                            f"{expected_cohort_counts_by_scope!r}"
                        ),
                    )
                for sample_id, tools in sorted(per_sample_tools.items()):
                    if sorted(tools) != sorted(expected_tools):
                        append_issue(
                            issues,
                            contract.stage_id,
                            "sample-results-tool-coverage-drift",
                            (
                                f"{sample_results_path.relative_to(docs_root.parent)} sample "
                                f"{sample_id} tools={sorted(tools)!r} expected "
                                f"{sorted(expected_tools)!r}"
                            ),
                        )
                expected_row_count = expected_total * len(expected_tools)
                if len(sample_rows) != expected_row_count:
                    append_issue(
                        issues,
                        contract.stage_id,
                        "sample-results-row-count-drift",
                        (
                            f"{sample_results_path.relative_to(docs_root.parent)} row_count="
                            f"{len(sample_rows)!r} expected {expected_row_count}"
                        ),
                    )

        if tool_runtime_summary_path.is_file() and tool_runtime_summary_path.stat().st_size > 0:
            tool_rows = load_csv_rows(tool_runtime_summary_path)
            observed_tools = sorted(
                row.get("tool", "").strip()
                for row in tool_rows
                if row.get("tool", "").strip()
            )
            if observed_tools != sorted(expected_tools):
                append_issue(
                    issues,
                    contract.stage_id,
                    "tool-runtime-summary-drift",
                    (
                        f"{tool_runtime_summary_path.relative_to(docs_root.parent)} tools="
                        f"{observed_tools!r} expected {sorted(expected_tools)!r}"
                    ),
                )

        if cohort_runtime_summary_path.is_file() and cohort_runtime_summary_path.stat().st_size > 0:
            cohort_rows = load_csv_rows(cohort_runtime_summary_path)
            era_layout_rows = [
                row
                for row in cohort_rows
                if row.get("dimension", "").strip() in ("", "era_layout")
            ]
            observed_cohorts = sorted(
                {
                    row.get("cohort", "").strip()
                    for row in era_layout_rows
                    if row.get("cohort", "").strip()
                }
            )
            if observed_cohorts != sorted(expected_cohort_counts_by_scope):
                append_issue(
                    issues,
                    contract.stage_id,
                    "cohort-runtime-summary-drift",
                    (
                        f"{cohort_runtime_summary_path.relative_to(docs_root.parent)} cohorts="
                        f"{observed_cohorts!r} expected {sorted(expected_cohort_counts_by_scope)!r}"
                    ),
                )

        if (
            sample_runtime_outliers_path.is_file()
            and sample_runtime_outliers_path.stat().st_size > 0
        ):
            outlier_rows = load_csv_rows(sample_runtime_outliers_path)
            unique_sample_ids = {
                row.get("sample_id", "").strip()
                for row in outlier_rows
                if row.get("sample_id", "").strip()
            }
            if len(unique_sample_ids) != expected_total:
                append_issue(
                    issues,
                    contract.stage_id,
                    "sample-runtime-outlier-coverage-drift",
                    (
                        f"{sample_runtime_outliers_path.relative_to(docs_root.parent)} unique_samples="
                        f"{len(unique_sample_ids)!r} expected {expected_total}"
                    ),
                )

    for supplemental_issue in supplemental_issues or []:
        issues.append(supplemental_issue)

    return {
        "stage_id": contract.stage_id,
        "scenario_id": contract.scenario_id,
        "sample_scope": contract.sample_scope,
        "contract_tool_roster": contract.tools,
        "expected_tool_roster": expected_tools,
        "method_path": str(method_path.relative_to(docs_root.parent)),
        "corpus_path": str(corpus_root.relative_to(docs_root.parent)),
        "status": "complete" if not issues else "incomplete",
        "issue_count": len(issues),
        "issues": [asdict(issue) for issue in issues],
    }


def audit_docs(
    docs_root: Path,
    *,
    repo_root: Path | None = None,
    stage_contracts: list[CorpusBenchmarkContract] | None = None,
    exclusions: list[CorpusBenchmarkExclusion] | None = None,
    supplemental_findings: dict[str, list[StageAuditIssue]] | None = None,
    audit_warnings: list[str] | None = None,
    supplemental_findings_generated_at_utc: str | None = None,
) -> dict:
    repo_root = repo_root or docs_root.parent.parent
    stage_contracts = stage_contracts or CORPUS_01_PUBLICATION_CONTRACTS
    exclusions = exclusions or CORPUS_01_PUBLICATION_EXCLUSIONS
    supplemental_findings = supplemental_findings or {}
    audit_warnings = audit_warnings or []
    audit_warnings = audit_warnings + makefile_publication_warnings(
        repo_root,
        stage_contracts,
    )
    stage_reports = [
        audit_stage(
            repo_root,
            docs_root,
            contract,
            supplemental_issues=supplemental_findings.get(contract.stage_id, []),
        )
        for contract in stage_contracts
    ]
    return {
        "corpus_id": "corpus-01",
        "docs_root": str(docs_root),
        "benchmarkable_stage_count": len(stage_contracts) + len(exclusions),
        "applicable_stage_count": len(stage_reports),
        "completed_stage_count": sum(
            1 for report in stage_reports if report["status"] == "complete"
        ),
        "incomplete_stage_count": sum(
            1 for report in stage_reports if report["status"] != "complete"
        ),
        "excluded_stage_count": len(exclusions),
        "issue_count": sum(report["issue_count"] for report in stage_reports),
        "audit_warning_count": len(audit_warnings),
        "audit_warnings": audit_warnings,
        "supplemental_findings_generated_at_utc": supplemental_findings_generated_at_utc,
        "excluded_stages": [asdict(exclusion) for exclusion in exclusions],
        "stages": stage_reports,
    }


def render_markdown(report: dict) -> str:
    audit_warning_count = int(report.get("audit_warning_count", 0) or 0)
    audit_warnings = list(report.get("audit_warnings", []) or [])
    lines = [
        "# `corpus-01` FASTQ benchmark publication status",
        "",
        f"- Benchmarkable governed stages: `{report['benchmarkable_stage_count']}`",
        f"- Corpus-applicable publication stages: `{report['applicable_stage_count']}`",
        f"- Completed stage dossiers: `{report['completed_stage_count']}`",
        f"- Incomplete stage dossiers: `{report['incomplete_stage_count']}`",
        f"- Excluded stages: `{report['excluded_stage_count']}`",
        f"- Publication issues: `{report['issue_count']}`",
        f"- Audit warnings: `{audit_warning_count}`",
        "",
        "## Stage status",
        "",
    ]
    for stage in report["stages"]:
        lines.append(
            f"- `{stage['stage_id']}`: `{stage['status']}`"
            f" (`{stage['issue_count']}` issues, scope `{stage['sample_scope']}`)"
        )
        if stage["issues"]:
            for issue in stage["issues"]:
                lines.append(f"  - `{issue['issue_id']}`: {issue['detail']}")
    if audit_warnings:
        lines.extend(["", "## Audit Warnings", ""])
        for warning in audit_warnings:
            lines.append(f"- {warning}")
    lines.extend(
        [
            "",
            "## Excluded Stages",
            "",
        ]
    )
    for exclusion in report["excluded_stages"]:
        lines.append(f"- `{exclusion['stage_id']}`: {exclusion['reason']}")
    lines.extend(
        [
            "",
            "## Contract",
            "",
            "A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`.",
            "Published summaries must also match the governed scenario id, exact benchmark tool roster, expected corpus scope (`full` or `paired`), zero sample failures, and complete sample-by-tool coverage.",
        ]
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    docs_root = Path(args.docs_root).resolve()
    findings_path = Path(args.findings_in).resolve()
    supplemental_findings, audit_warnings, findings_generated_at_utc = load_supplemental_findings(
        findings_path
    )
    report = audit_docs(
        docs_root,
        repo_root=repo_root,
        supplemental_findings=supplemental_findings,
        audit_warnings=audit_warnings,
        supplemental_findings_generated_at_utc=findings_generated_at_utc,
    )

    json_out = Path(args.json_out).resolve()
    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    markdown_out = Path(args.markdown_out).resolve()
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.write_text(render_markdown(report), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
