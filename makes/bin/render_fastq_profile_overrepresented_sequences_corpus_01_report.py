#!/usr/bin/env python3
from __future__ import annotations

import argparse
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    parse_corpus_report_args,
    CorpusReportArtifacts,
    CorpusReportContext,
    PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT,
    load_json,
    localize_results_path,
    run_corpus_report,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.profile_overrepresented_sequences benchmark summary.",
        docs_root="docs/benchmark/fastq.profile_overrepresented_sequences/corpus-01",
    )


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def validate_overrepresented_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "overrepresented benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "overrepresented benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"expected stage_id {PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.stage_id}, found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"expected scenario_id {PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.scenario_id}, found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind") != "benchmark":
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    expected_contract = {
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
    }
    for key, expected in expected_contract.items():
        if run_manifest.get(key) != expected:
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"expected {key}={expected!r}, found {run_manifest.get(key)!r}"
            )
    if int(run_manifest.get("top_k", 0)) <= 0:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"top_k must be positive, found {run_manifest.get('top_k')!r}"
        )
    expected_artifacts = [
        "overrepresented_sequences_tsv",
        "overrepresented_sequences_json",
        "report_json",
    ]
    if run_manifest.get("overrepresented_artifacts") != expected_artifacts:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"expected overrepresented_artifacts {expected_artifacts}, found {run_manifest.get('overrepresented_artifacts')!r}"
        )


def tool_output_dir(run_report_path: Path, tool: str) -> Path:
    return run_report_path.parent / "tools" / tool


def validate_artifact_paths(report_path: Path, tool: str) -> dict[str, str]:
    tool_dir = tool_output_dir(report_path, tool)
    output_tsv = tool_dir / "overrepresented_sequences.tsv"
    output_json = tool_dir / "overrepresented_sequences.json"
    report_json = tool_dir / "overrepresented_report.json"
    missing = [
        str(path) for path in [output_tsv, output_json, report_json] if not path.is_file()
    ]
    if missing:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"missing governed overrepresented artifacts for {tool}: {missing}"
        )
    empty = [
        str(path) for path in [output_tsv, output_json, report_json] if path.stat().st_size == 0
    ]
    if empty:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"empty governed overrepresented artifacts for {tool}: {empty}"
        )
    return {
        "overrepresented_sequences_tsv_artifact": str(output_tsv),
        "overrepresented_sequences_json_artifact": str(output_json),
        "report_json_artifact": str(report_json),
    }


def validate_overrepresented_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if int(row["sequence_count"]) < 0:
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"sequence_count must be non-negative for {row['sample_id']}/{row['tool']}"
            )
        if int(row["flagged_sequences"]) < 0:
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"flagged_sequences must be non-negative for {row['sample_id']}/{row['tool']}"
            )
        if int(row["flagged_sequences"]) > int(row["sequence_count"]):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"flagged_sequences must be <= sequence_count for {row['sample_id']}/{row['tool']}"
            )
        if int(row["sequence_count"]) > int(row["top_k"]):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"sequence_count must be <= top_k for {row['sample_id']}/{row['tool']}"
            )
        if not 0.0 <= float(row["top_fraction"]) <= 1.0:
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"top_fraction must be within [0, 1] for {row['sample_id']}/{row['tool']}"
            )
        if int(row["sequence_count"]) == 0 and (
            int(row["flagged_sequences"]) != 0 or float(row["top_fraction"]) != 0.0
        ):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"empty ranked outputs must carry zero flagged_sequences and zero top_fraction for {row['sample_id']}/{row['tool']}"
            )
        if int(row["top_k"]) != int(run_manifest["top_k"]):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"top_k must equal run manifest for {row['sample_id']}/{row['tool']}"
            )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "overrepresented benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.profile_overrepresented_sequences` on `corpus-01`")
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Generated: {summary['generated_at_utc']}")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Run root: `{summary['run_root']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Samples benchmarked: `{summary['samples_total']}`")
    lines.append(
        f"- Layout balance: `{summary['layout_counts'].get('se', 0)}` single-end, `{summary['layout_counts'].get('pe', 0)}` paired-end"
    )
    lines.append(
        f"- Era balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern"
    )
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- report_only: `{summary['report_only']}`")
    lines.append(f"- mutates_fastq: `{summary['mutates_fastq']}`")
    lines.append(f"- may_change_read_count: `{summary['may_change_read_count']}`")
    lines.append(f"- top_k: `{summary['top_k']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("largest_sequence_count_tool"):
        lines.append(
            f"- Highest median profiled sequence count: `{headline['largest_sequence_count_tool']}` at `{headline['largest_sequence_count']:.1f}`."
        )
    if headline.get("highest_top_fraction_tool"):
        lines.append(
            f"- Highest median dominant-sequence fraction: `{headline['highest_top_fraction_tool']}` at `{headline['highest_top_fraction']:.3f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median sequence count | Median flagged sequences | Median top fraction |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        sequence_count = (
            "n/a"
            if row["median_sequence_count"] is None
            else f"{row['median_sequence_count']:.1f}"
        )
        flagged_sequences = (
            "n/a"
            if row["median_flagged_sequences"] is None
            else f"{row['median_flagged_sequences']:.1f}"
        )
        top_fraction = (
            "n/a"
            if row["median_top_fraction"] is None
            else f"{row['median_top_fraction']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {sequence_count} | {flagged_sequences} | {top_fraction} |"
        )
    lines.append("")
    lines.append("## Cohort Coverage")
    lines.append("")
    lines.append("| Cohort | Samples |")
    lines.append("| --- | ---: |")
    for label, count in sorted(summary["cohort_counts"].items()):
        lines.append(f"| `{label}` | {count} |")
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This stage is report-only and non-mutating: governed benchmarking compares overrepresented-sequence ranking behavior without changing the reads."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def build_artifacts(context: CorpusReportContext) -> CorpusReportArtifacts:
    repo_root = context.runtime.repo_root
    corpus_root = context.runtime.corpus_root
    run_root = context.runtime.run_root
    local_results_root = context.runtime.local_results_root

    spec = context.spec
    run_manifest = context.run_manifest
    expected_sample_ids = context.expected_sample_ids
    metadata_by_sample = context.metadata_by_sample

    sample_rows: list[dict] = []
    tool_rows: dict[str, list[dict]] = defaultdict(list)
    cohort_counts: dict[str, int] = defaultdict(int)
    era_counts: dict[str, int] = defaultdict(int)
    layout_counts: dict[str, int] = defaultdict(int)

    for run in context.applicable_runs:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample.get(sample_id, {})
        cohort_key = f"{metadata.get('era', 'unknown')}_{metadata.get('layout', run['layout'])}"
        cohort_counts[cohort_key] += 1
        era_counts[metadata.get("era", "unknown")] += 1
        layout_counts[metadata.get("layout", run["layout"])] += 1

        report_path = localize_results_path(run["report_json"], local_results_root)
        if not report_path.is_file():
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"missing report.json for {sample_id}: {report_path}"
            )
        report = load_json(report_path)
        if not report.get("records"):
            raise SystemExit(
                "overrepresented benchmark report drift: "
                f"report.json for {sample_id} contains no records"
            )
        for record in report.get("records", []):
            tool = record.get("context", {}).get("tool", "unknown")
            artifact_paths = validate_artifact_paths(report_path, tool)
            row = {
                "sample_id": sample_id,
                "accession": metadata.get("accession"),
                "era": metadata.get("era"),
                "layout": metadata.get("layout", run["layout"]),
                "study_accession": metadata.get("study_accession"),
                "size_band": metadata.get("size_band"),
                "tool": tool,
                "runtime_s": record.get("execution", {}).get("runtime_s"),
                "exit_code": record.get("execution", {}).get("exit_code"),
                "sequence_count": normalize_metric(record, "sequence_count") or 0,
                "flagged_sequences": normalize_metric(record, "flagged_sequences") or 0,
                "top_fraction": normalize_metric(record, "top_fraction") or 0.0,
                "top_k": run_manifest["top_k"],
                "overrepresented_sequences_tsv_artifact": artifact_paths[
                    "overrepresented_sequences_tsv_artifact"
                ],
                "overrepresented_sequences_json_artifact": artifact_paths[
                    "overrepresented_sequences_json_artifact"
                ],
                "report_json_artifact": artifact_paths["report_json_artifact"],
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_overrepresented_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median([float(row["runtime_s"]) for row in rows]),
                "median_sequence_count": safe_median(
                    [float(row["sequence_count"]) for row in rows]
                ),
                "median_flagged_sequences": safe_median(
                    [float(row["flagged_sequences"]) for row in rows]
                ),
                "median_top_fraction": safe_median(
                    [float(row["top_fraction"]) for row in rows]
                ),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        largest_sequence_count = max(
            (row for row in tool_summary if row["median_sequence_count"] is not None),
            key=lambda row: row["median_sequence_count"],
        )
        highest_top_fraction = max(
            (row for row in tool_summary if row["median_top_fraction"] is not None),
            key=lambda row: row["median_top_fraction"],
        )
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "largest_sequence_count_tool": largest_sequence_count["tool"],
            "largest_sequence_count": largest_sequence_count["median_sequence_count"],
            "highest_top_fraction_tool": highest_top_fraction["tool"],
            "highest_top_fraction": highest_top_fraction["median_top_fraction"],
        }

    summary = {
        "schema_version": "bijux.fastq.profile_overrepresented_sequences.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": run_manifest["scenario_id"],
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "report_only": run_manifest["report_only"],
        "mutates_fastq": run_manifest["mutates_fastq"],
        "may_change_read_count": run_manifest["may_change_read_count"],
        "top_k": run_manifest["top_k"],
        "headline": headline,
        "tool_summary": tool_summary,
    }

    return CorpusReportArtifacts(
        summary=summary,
        markdown=render_markdown(summary),
        sample_rows=sample_rows,
        sample_fieldnames=[
            "sample_id",
            "accession",
            "era",
            "layout",
            "study_accession",
            "size_band",
            "tool",
            "runtime_s",
            "exit_code",
            "sequence_count",
            "flagged_sequences",
            "top_fraction",
            "top_k",
            "overrepresented_sequences_tsv_artifact",
            "overrepresented_sequences_json_artifact",
            "report_json_artifact",
        ],
        summary_sort_keys=True,
    )


def main() -> int:
    return run_corpus_report(
        parse_args(),
        contract=PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT,
        validate_run_manifest=validate_overrepresented_run_manifest_contract,
        metadata_fallback_stage_id=PROFILE_OVERREPRESENTED_BENCHMARK_CONTRACT.stage_id,
        build_artifacts=build_artifacts,
    )


if __name__ == "__main__":
    raise SystemExit(main())
