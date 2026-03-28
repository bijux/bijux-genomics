#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from benchmark_fastq_corpus.support import CORPUS_01_PUBLICATION_CONTRACTS


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a machine-readable remediation queue for corpus-01 FASTQ benchmark publication."
    )
    parser.add_argument(
        "--status-json",
        default="docs/benchmark/corpus-01-status.json",
        help="Publication status ledger.",
    )
    parser.add_argument(
        "--results-json",
        default="docs/benchmark/corpus-01-results-status.json",
        help="Local results mirror audit ledger.",
    )
    parser.add_argument(
        "--findings-json",
        default="docs/benchmark/corpus-01-publication-findings.json",
        help="Supplemental publication findings ledger.",
    )
    parser.add_argument(
        "--dossier-index-json",
        default="docs/benchmark/corpus-01-dossier-index.json",
        help="Dossier freshness index.",
    )
    parser.add_argument(
        "--json-out",
        default="docs/benchmark/corpus-01-remediation-queue.json",
        help="Machine-readable remediation queue output path.",
    )
    parser.add_argument(
        "--markdown-out",
        default="docs/benchmark/corpus-01-remediation-queue.md",
        help="Markdown remediation queue output path.",
    )
    return parser.parse_args()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def classify_recommended_action(issue_ids: list[str]) -> str:
    sync_issue_ids = {
        "missing-local-run-root",
        "missing-stage-run-manifest",
        "missing-localized-report-json",
        "duplicate-result-root-ambiguity",
    }
    publish_issue_ids = {
        "missing-published-summary",
        "missing-corpus-dir",
        "missing-summary-json",
        "missing-lunarc-md",
        "missing-sample-results-csv",
        "missing-tool-runtime-summary-csv",
        "missing-cohort-runtime-summary-csv",
        "missing-sample-runtime-outliers-csv",
    }
    rerun_issue_fragments = (
        "sample-failures",
        "dry-run",
        "sample-limit",
    )
    if any(issue_id in sync_issue_ids for issue_id in issue_ids):
        return "sync-or-normalize-results"
    if any(issue_id in publish_issue_ids for issue_id in issue_ids):
        return "render-or-publish-dossier"
    if any(fragment in issue_id for issue_id in issue_ids for fragment in rerun_issue_fragments):
        return "rerun-benchmark-stage"
    return "repair-benchmark-contract"


def summarize_issue_groups(issues: list[dict]) -> list[dict]:
    grouped: dict[str, dict] = {}
    for issue in issues:
        issue_id = str(issue["issue_id"])
        group = grouped.setdefault(
            issue_id,
            {
                "issue_id": issue_id,
                "count": 0,
                "sources": set(),
                "details": [],
                "severity": issue.get("severity", "error"),
            },
        )
        group["count"] += 1
        group["sources"].add(str(issue.get("source", "unknown")))
        detail = str(issue.get("detail", "")).strip()
        if detail and detail not in group["details"]:
            group["details"].append(detail)

    summaries: list[dict] = []
    for issue_id in sorted(grouped):
        group = grouped[issue_id]
        summaries.append(
            {
                "issue_id": issue_id,
                "count": group["count"],
                "sources": sorted(group["sources"]),
                "severity": group["severity"],
                "example_details": group["details"][:3],
                "additional_detail_count": max(len(group["details"]) - 3, 0),
            }
        )
    return summaries


def build_queue(
    publication_status: dict,
    results_status: dict,
    findings_payload: dict,
    dossier_index: dict,
) -> dict:
    publication_by_stage = {
        stage["stage_id"]: stage for stage in publication_status.get("stages", [])
    }
    results_by_stage = {stage["stage_id"]: stage for stage in results_status.get("stages", [])}
    findings_by_stage: dict[str, list[dict]] = {}
    for finding in findings_payload.get("findings", []):
        findings_by_stage.setdefault(str(finding.get("stage_id")), []).append(
            {
                "issue_id": str(finding.get("issue_id")),
                "detail": str(finding.get("detail")),
                "severity": str(finding.get("severity", "error")),
                "source": "findings",
            }
        )
    dossier_by_stage = {stage["stage_id"]: stage for stage in dossier_index.get("stages", [])}

    stages: list[dict] = []
    for contract in CORPUS_01_PUBLICATION_CONTRACTS:
        stage_id = contract.stage_id
        publication_stage = publication_by_stage.get(stage_id, {})
        results_stage = results_by_stage.get(stage_id, {})
        dossier_stage = dossier_by_stage.get(stage_id, {})
        issues = [
            {
                "issue_id": issue["issue_id"],
                "detail": issue["detail"],
                "severity": issue.get("severity", "error"),
                "source": "publication",
            }
            for issue in publication_stage.get("issues", [])
        ]
        issues.extend(
            {
                "issue_id": issue["issue_id"],
                "detail": issue["detail"],
                "severity": issue.get("severity", "error"),
                "source": "results",
            }
            for issue in results_stage.get("issues", [])
        )
        issues.extend(findings_by_stage.get(stage_id, []))
        issue_ids = [issue["issue_id"] for issue in issues]
        issue_groups = summarize_issue_groups(issues)
        stages.append(
            {
                "stage_id": stage_id,
                "owner": "benchmark-governance",
                "status": "open" if issues else "clear",
                "issue_count": len(issues),
                "issue_group_count": len(issue_groups),
                "recommended_action": classify_recommended_action(issue_ids) if issues else "none",
                "publication_status": publication_stage.get("status", "missing"),
                "results_status": results_stage.get("status", "missing"),
                "sample_scope": contract.sample_scope,
                "published_generated_at_utc": dossier_stage.get("generated_at_utc"),
                "run_root_source": dossier_stage.get("run_root_source"),
                "issue_groups": issue_groups,
                "issues": issues,
            }
        )

    return {
        "corpus_id": "corpus-01",
        "stage_count": len(stages),
        "open_stage_count": sum(1 for stage in stages if stage["status"] == "open"),
        "clear_stage_count": sum(1 for stage in stages if stage["status"] == "clear"),
        "stages": stages,
    }


def render_markdown(queue: dict) -> str:
    lines = [
        "# `corpus-01` FASTQ remediation queue",
        "",
        f"- Governed publication stages: `{queue['stage_count']}`",
        f"- Open stages: `{queue['open_stage_count']}`",
        f"- Clear stages: `{queue['clear_stage_count']}`",
        "",
        "## Stage queue",
        "",
    ]
    for stage in queue["stages"]:
        lines.append(
            f"- `{stage['stage_id']}`: `{stage['status']}` via `{stage['recommended_action']}`"
        )
        lines.append(
            f"  - publication `{stage['publication_status']}`, results `{stage['results_status']}`, owner `{stage['owner']}`"
        )
        if stage["published_generated_at_utc"]:
            lines.append(
                f"  - dossier `{stage['published_generated_at_utc']}` from `{stage['run_root_source']}`"
            )
        for group in stage.get("issue_groups", []):
            lines.append(
                f"  - issue group `{group['issue_id']}` x{group['count']} from {', '.join(group['sources'])}"
            )
            for detail in group["example_details"]:
                lines.append(f"    - {detail}")
            if group["additional_detail_count"]:
                lines.append(
                    f"    - (+{group['additional_detail_count']} more detail rows)"
                )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    publication_status = load_json(Path(args.status_json).resolve())
    results_status = load_json(Path(args.results_json).resolve())
    findings_payload = load_json(Path(args.findings_json).resolve())
    dossier_index = load_json(Path(args.dossier_index_json).resolve())
    queue = build_queue(
        publication_status,
        results_status,
        findings_payload,
        dossier_index,
    )

    json_out = Path(args.json_out).resolve()
    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(queue, indent=2) + "\n", encoding="utf-8")

    markdown_out = Path(args.markdown_out).resolve()
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.write_text(render_markdown(queue), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
