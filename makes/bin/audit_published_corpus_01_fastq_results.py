#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
from dataclasses import asdict, dataclass
from pathlib import Path

from benchmark_fastq_corpus.support import (
    CORPUS_01_PUBLICATION_CONTRACTS,
    configured_stage_run_roots,
    load_json,
    localize_results_path,
    run_root_observed_timestamp,
    select_stage_run_root,
)


@dataclass(frozen=True)
class StageResultIssue:
    stage_id: str
    issue_id: str
    detail: str


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit local result mirror integrity for published corpus-01 FASTQ benchmark dossiers."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument(
        "--json-out",
        default="docs/benchmark/corpus-01-results-status.json",
    )
    parser.add_argument(
        "--markdown-out",
        default="docs/benchmark/corpus-01-results-status.md",
    )
    return parser.parse_args()


def append_issue(
    issues: list[StageResultIssue],
    stage_id: str,
    issue_id: str,
    detail: str,
) -> None:
    issues.append(StageResultIssue(stage_id=stage_id, issue_id=issue_id, detail=detail))


TOOL_LITERAL_PATTERN = re.compile(r'"tool"\s*:\s*"([^"]+)"')


def observed_tools_from_report(path: Path) -> list[str]:
    text = path.read_text(encoding="utf-8")
    return sorted({match.group(1) for match in TOOL_LITERAL_PATTERN.finditer(text)})


def audit_stage(repo_root: Path, stage_id: str, scenario_id: str, tools: list[str]) -> dict:
    docs_root = repo_root / "docs" / "benchmark" / stage_id / "corpus-01"
    summary_path = docs_root / "summary.json"
    issues: list[StageResultIssue] = []
    if not summary_path.is_file():
        append_issue(
            issues,
            stage_id,
            "missing-published-summary",
            f"missing {summary_path}",
        )
        return {
            "stage_id": stage_id,
            "status": "incomplete",
            "issue_count": len(issues),
            "issues": [asdict(issue) for issue in issues],
        }
    summary = load_json(summary_path)
    expected_tools = sorted(tools)

    corpus_root = Path(str(summary.get("corpus_root", "")))
    configured_roots = configured_stage_run_roots(corpus_root, stage_id)
    canonical_run_root = configured_roots[0].path
    legacy_run_root = configured_roots[1].path
    reported_run_root = Path(str(summary.get("run_root", "")))
    selected_run_root = select_stage_run_root(corpus_root, stage_id).selected.path
    existing_roots = [root for root in [reported_run_root, *(candidate.path for candidate in configured_roots)] if root.is_dir()]
    unique_existing_roots: list[Path] = []
    for root in existing_roots:
        if root not in unique_existing_roots:
            unique_existing_roots.append(root)
    if canonical_run_root.is_dir() and legacy_run_root.is_dir():
        append_issue(
            issues,
            stage_id,
            "duplicate-result-root-ambiguity",
            f"both {canonical_run_root} and {legacy_run_root} exist",
        )
    resolved_run_root = (
        reported_run_root if reported_run_root.is_dir() else selected_run_root
    )
    newest_available_run_root = resolved_run_root
    newest_available_timestamp = run_root_observed_timestamp(resolved_run_root)
    for candidate_root in unique_existing_roots:
        candidate_timestamp = run_root_observed_timestamp(candidate_root)
        if candidate_timestamp is None:
            continue
        if newest_available_timestamp is None or candidate_timestamp > newest_available_timestamp:
            newest_available_run_root = candidate_root
            newest_available_timestamp = candidate_timestamp
    if reported_run_root != canonical_run_root and not reported_run_root.is_dir():
        append_issue(
            issues,
            stage_id,
            "summary-run-root-drift",
            f"summary run_root={reported_run_root} expected {canonical_run_root}",
        )
    if (
        newest_available_timestamp is not None
        and newest_available_run_root != resolved_run_root
    ):
        append_issue(
            issues,
            stage_id,
            "newer-run-root-available",
            "published dossier selected "
            f"{resolved_run_root} but newer mirrored run exists at {newest_available_run_root}",
        )
    if not resolved_run_root.is_dir():
        append_issue(
            issues,
            stage_id,
            "missing-local-run-root",
            "local mirror missing: "
            f"selected={resolved_run_root}; "
            f"summary_run_root={reported_run_root}; "
            f"expected_local_mirror={canonical_run_root}",
        )
    else:
        polluting_files = sorted(
            path for path in resolved_run_root.rglob(".DS_Store") if path.is_file()
        )
        if polluting_files:
            append_issue(
                issues,
                stage_id,
                "polluting-mirror-artifact",
                f"mirror contains {len(polluting_files)} .DS_Store files under {resolved_run_root}",
            )
    stage_run_manifest = resolved_run_root / "run_manifest.json"
    if not stage_run_manifest.is_file():
        append_issue(
            issues,
            stage_id,
            "missing-stage-run-manifest",
            f"missing {stage_run_manifest}",
        )
    else:
        run_manifest = load_json(stage_run_manifest)
        if run_manifest.get("stage_id") != stage_id:
            append_issue(
                issues,
                stage_id,
                "run-manifest-stage-id-drift",
                f"run_manifest stage_id={run_manifest.get('stage_id')!r}",
            )
        if run_manifest.get("scenario_id") != scenario_id:
            append_issue(
                issues,
                stage_id,
                "run-manifest-scenario-id-drift",
                f"run_manifest scenario_id={run_manifest.get('scenario_id')!r}",
            )
        if sorted(run_manifest.get("tools") or []) != expected_tools:
            append_issue(
                issues,
                stage_id,
                "run-manifest-tool-roster-drift",
                f"run_manifest tools={run_manifest.get('tools')!r} expected {expected_tools!r}",
            )
        if run_manifest.get("dry_run"):
            append_issue(
                issues,
                stage_id,
                "run-manifest-dry-run",
                "run_manifest recorded dry_run=true",
            )
        if run_manifest.get("sample_limit") is not None:
            append_issue(
                issues,
                stage_id,
                "run-manifest-sample-limit",
                f"run_manifest sample_limit={run_manifest.get('sample_limit')!r}",
            )
        if int(run_manifest.get("samples_failed", 0) or 0) != 0:
            append_issue(
                issues,
                stage_id,
                "run-manifest-sample-failures",
                f"run_manifest samples_failed={run_manifest.get('samples_failed')!r}",
            )
        local_results_root = resolved_run_root.parents[2]
        missing_report_count = 0
        tool_roster_drift_samples: list[str] = []
        for run in run_manifest.get("runs", []):
            report_json = run.get("report_json")
            if not report_json:
                missing_report_count += 1
                continue
            localized_report = localize_results_path(str(report_json), local_results_root)
            if not localized_report.is_file():
                missing_report_count += 1
                continue
            observed_tools = observed_tools_from_report(localized_report)
            if observed_tools != expected_tools:
                tool_roster_drift_samples.append(
                    f"{run.get('sample_id')} observed {observed_tools!r}"
                )
        if missing_report_count:
            append_issue(
                issues,
                stage_id,
                "missing-localized-report-json",
                f"{missing_report_count} run rows do not resolve to a local report.json",
            )
        if tool_roster_drift_samples:
            append_issue(
                issues,
                stage_id,
                "report-tool-roster-drift",
                "; ".join(tool_roster_drift_samples[:3])
                + (
                    f" (+{len(tool_roster_drift_samples) - 3} more)"
                    if len(tool_roster_drift_samples) > 3
                    else ""
                ),
            )

    return {
        "stage_id": stage_id,
        "status": "complete" if not issues else "incomplete",
        "issue_count": len(issues),
        "reported_run_root": str(reported_run_root),
        "selected_run_root": str(resolved_run_root),
        "newest_available_run_root": str(newest_available_run_root),
        "selected_run_root_is_newest": newest_available_run_root == resolved_run_root,
        "available_run_roots": [str(root) for root in unique_existing_roots],
        "issues": [asdict(issue) for issue in issues],
    }


def audit_published_results(repo_root: Path) -> dict:
    stage_reports = []
    for contract in CORPUS_01_PUBLICATION_CONTRACTS:
        stage_reports.append(
            audit_stage(repo_root, contract.stage_id, contract.scenario_id, contract.tools)
        )
    return {
        "corpus_id": "corpus-01",
        "applicable_stage_count": len(CORPUS_01_PUBLICATION_CONTRACTS),
        "published_stage_count": sum(
            1
            for contract in CORPUS_01_PUBLICATION_CONTRACTS
            if (repo_root / "docs" / "benchmark" / contract.stage_id / "corpus-01" / "summary.json").is_file()
        ),
        "complete_stage_count": sum(1 for report in stage_reports if report["status"] == "complete"),
        "incomplete_stage_count": sum(
            1 for report in stage_reports if report["status"] != "complete"
        ),
        "issue_count": sum(report["issue_count"] for report in stage_reports),
        "stages": stage_reports,
    }


def render_markdown(report: dict) -> str:
    lines = [
        "# `corpus-01` published result mirror status",
        "",
        f"- Governed publication stages: `{report['applicable_stage_count']}`",
        f"- Published stages audited: `{report['published_stage_count']}`",
        f"- Complete mirrored stages: `{report['complete_stage_count']}`",
        f"- Incomplete mirrored stages: `{report['incomplete_stage_count']}`",
        f"- Mirror issues: `{report['issue_count']}`",
        "",
        "## Stage status",
        "",
    ]
    for stage in report["stages"]:
        lines.append(
            f"- `{stage['stage_id']}`: `{stage['status']}` (`{stage['issue_count']}` issues)"
        )
        if stage.get("selected_run_root"):
            lines.append(f"  - selected run root: `{stage['selected_run_root']}`")
        if stage.get("newest_available_run_root"):
            lines.append(
                "  - newest available run root: "
                f"`{stage['newest_available_run_root']}` "
                f"(selected newest=`{stage['selected_run_root_is_newest']}`)"
            )
        if stage.get("available_run_roots"):
            roots = ", ".join(f"`{root}`" for root in stage["available_run_roots"])
            lines.append(f"  - available run roots: {roots}")
        for issue in stage["issues"]:
            lines.append(f"  - `{issue['issue_id']}`: {issue['detail']}")
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    report = audit_published_results(repo_root)
    json_out = (repo_root / args.json_out).resolve()
    markdown_out = (repo_root / args.markdown_out).resolve()
    json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    markdown_out.write_text(render_markdown(report), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
