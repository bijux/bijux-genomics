#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from dataclasses import asdict, dataclass
from pathlib import Path


@dataclass(frozen=True)
class StagePublicationContract:
    stage_id: str
    scenario_id: str


@dataclass(frozen=True)
class StageAuditIssue:
    stage_id: str
    issue_id: str
    severity: str
    detail: str


STAGE_CONTRACTS = [
    StagePublicationContract("fastq.validate_reads", "validation_fairness"),
    StagePublicationContract("fastq.detect_adapters", "detect_adapters_fairness"),
    StagePublicationContract("fastq.merge_pairs", "merge_fairness"),
    StagePublicationContract("fastq.profile_reads", "profile_reads_fairness"),
    StagePublicationContract("fastq.profile_read_lengths", "read_length_fairness"),
    StagePublicationContract(
        "fastq.profile_overrepresented_sequences",
        "overrepresented_sequence_fairness",
    ),
    StagePublicationContract("fastq.trim_polyg_tails", "polyg_trim_fairness"),
    StagePublicationContract("fastq.trim_reads", "trim_fairness"),
    StagePublicationContract(
        "fastq.trim_terminal_damage",
        "terminal_damage_fairness",
    ),
    StagePublicationContract("fastq.report_qc", "qc_aggregation_fairness"),
]

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
    return parser.parse_args()


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def audit_stage(docs_root: Path, contract: StagePublicationContract) -> dict:
    stage_root = docs_root / contract.stage_id
    method_path = stage_root / "corpus-01-method.md"
    corpus_root = stage_root / "corpus-01"
    issues: list[StageAuditIssue] = []

    if not method_path.is_file():
        issues.append(
            StageAuditIssue(
                stage_id=contract.stage_id,
                issue_id="missing-method-doc",
                severity="error",
                detail=f"missing {method_path.relative_to(docs_root.parent)}",
            )
        )

    if not corpus_root.is_dir():
        issues.append(
            StageAuditIssue(
                stage_id=contract.stage_id,
                issue_id="missing-corpus-dir",
                severity="error",
                detail=f"missing {corpus_root.relative_to(docs_root.parent)}",
            )
        )
    else:
        for file_name in REQUIRED_STAGE_FILES:
            artifact_path = corpus_root / file_name
            if not artifact_path.is_file():
                issues.append(
                    StageAuditIssue(
                        stage_id=contract.stage_id,
                        issue_id=f"missing-{file_name.replace('.', '-')}",
                        severity="error",
                        detail=f"missing {artifact_path.relative_to(docs_root.parent)}",
                    )
                )
                continue
            if artifact_path.stat().st_size == 0:
                issues.append(
                    StageAuditIssue(
                        stage_id=contract.stage_id,
                        issue_id=f"empty-{file_name.replace('.', '-')}",
                        severity="error",
                        detail=f"empty {artifact_path.relative_to(docs_root.parent)}",
                    )
                )

        summary_path = corpus_root / "summary.json"
        if summary_path.is_file() and summary_path.stat().st_size > 0:
            summary = load_json(summary_path)
            if summary.get("stage_id") != contract.stage_id:
                issues.append(
                    StageAuditIssue(
                        stage_id=contract.stage_id,
                        issue_id="summary-stage-id-drift",
                        severity="error",
                        detail=(
                            f"{summary_path.relative_to(docs_root.parent)} stage_id="
                            f"{summary.get('stage_id')!r}"
                        ),
                    )
                )
            if summary.get("scenario_id") != contract.scenario_id:
                issues.append(
                    StageAuditIssue(
                        stage_id=contract.stage_id,
                        issue_id="summary-scenario-id-drift",
                        severity="error",
                        detail=(
                            f"{summary_path.relative_to(docs_root.parent)} scenario_id="
                            f"{summary.get('scenario_id')!r}"
                        ),
                    )
                )

    return {
        "stage_id": contract.stage_id,
        "scenario_id": contract.scenario_id,
        "method_path": str(method_path.relative_to(docs_root.parent)),
        "corpus_path": str(corpus_root.relative_to(docs_root.parent)),
        "status": "complete" if not issues else "incomplete",
        "issue_count": len(issues),
        "issues": [asdict(issue) for issue in issues],
    }


def audit_docs(docs_root: Path) -> dict:
    stage_reports = [audit_stage(docs_root, contract) for contract in STAGE_CONTRACTS]
    return {
        "corpus_id": "corpus-01",
        "docs_root": str(docs_root),
        "stage_count": len(stage_reports),
        "completed_stage_count": sum(
            1 for report in stage_reports if report["status"] == "complete"
        ),
        "issue_count": sum(report["issue_count"] for report in stage_reports),
        "stages": stage_reports,
    }


def render_markdown(report: dict) -> str:
    lines = [
        "# `corpus-01` FASTQ benchmark publication status",
        "",
        f"- Stage count: `{report['stage_count']}`",
        f"- Completed stage dossiers: `{report['completed_stage_count']}`",
        f"- Publication issues: `{report['issue_count']}`",
        "",
        "## Stage status",
        "",
    ]
    for stage in report["stages"]:
        lines.append(
            f"- `{stage['stage_id']}`: `{stage['status']}`"
            f" (`{stage['issue_count']}` issues)"
        )
        if stage["issues"]:
            for issue in stage["issues"]:
                lines.append(
                    f"  - `{issue['issue_id']}`: {issue['detail']}"
                )
    lines.append("")
    lines.append("## Contract")
    lines.append("")
    lines.append(
        "A complete published corpus dossier requires `corpus-01-method.md`, `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`, and `lunarc.md`."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    report = audit_docs(docs_root)

    json_out = Path(args.json_out).resolve()
    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")

    markdown_out = Path(args.markdown_out).resolve()
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.write_text(render_markdown(report), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
