#!/usr/bin/env python3
from __future__ import annotations

import argparse
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from benchmark_fastq_corpus.support import (
    parse_corpus_report_args,
    CorpusBenchmarkContract,
    CorpusReportArtifacts,
    CorpusReportContext,
    load_json,
    localize_results_path,
    run_corpus_report,
)


VALIDATE_READS_BENCHMARK_CONTRACT = CorpusBenchmarkContract(
    stage_id="fastq.validate_reads",
    scenario_id="validation_fairness",
    tools=["fastq_scan", "fastqc", "fastqvalidator", "fqtools", "seqtk"],
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.validate_reads benchmark summary.",
        docs_root="docs/benchmark/fastq.validate_reads/corpus-01",
    )


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_fraction(numerator: int, denominator: int) -> float | None:
    if denominator == 0:
        return None
    return numerator / denominator


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.validate_reads` on `corpus-01`")
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(
        f"- Generated: {summary['generated_at_utc']}"
    )
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Run root: `{summary['run_root']}`")
    lines.append(f"- Samples benchmarked: `{summary['samples_total']}`")
    lines.append(
        f"- Layout balance: `{summary['layout_counts'].get('se', 0)}` single-end, `{summary['layout_counts'].get('pe', 0)}` paired-end"
    )
    lines.append(
        f"- Era balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern"
    )
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("highest_pass_rate_tool"):
        lines.append(
            f"- Highest pass rate: `{headline['highest_pass_rate_tool']}` at `{headline['highest_pass_rate']:.1%}`."
        )
    if headline.get("largest_invalid_read_tool"):
        lines.append(
            f"- Most invalid reads reported: `{headline['largest_invalid_read_tool']}` with `{headline['largest_invalid_read_count']}` reads."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Invalid reads | Strict pass rate |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        strict_pass_rate = (
            "n/a"
            if row["strict_pass_rate"] is None
            else f"{row['strict_pass_rate']:.1%}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {row['reads_invalid_total']} | {strict_pass_rate} |"
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
        "- Ancient and modern samples are resolved by matching normalized FASTQ checksums back to raw accession directories and then joining those accessions to `configs/runtime/corpora/corpus-01.toml`."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def build_artifacts(context: CorpusReportContext) -> CorpusReportArtifacts:
    corpus_root = context.runtime.corpus_root
    run_root = context.runtime.run_root
    local_results_root = context.runtime.local_results_root

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
            continue
        report = load_json(report_path)
        for record in report.get("records", []):
            tool = record.get("context", {}).get("tool", "unknown")
            runtime_s = record.get("execution", {}).get("runtime_s")
            exit_code = record.get("execution", {}).get("exit_code")
            row = {
                "sample_id": sample_id,
                "accession": metadata.get("accession"),
                "era": metadata.get("era"),
                "layout": metadata.get("layout", run["layout"]),
                "study_accession": metadata.get("study_accession"),
                "size_band": metadata.get("size_band"),
                "tool": tool,
                "runtime_s": runtime_s,
                "exit_code": exit_code,
                "reads_invalid": normalize_metric(record, "reads_invalid") or 0,
                "reads_total": normalize_metric(record, "reads_total") or 0,
                "strict_pass": normalize_metric(record, "strict_pass"),
                "pair_sync_pass": normalize_metric(record, "pair_sync_pass"),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    tool_summary: list[dict] = []
    for tool, rows in sorted(tool_rows.items()):
        runtimes = [float(row["runtime_s"]) for row in rows if row["runtime_s"] is not None]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        strict_defined = [row for row in rows if row["strict_pass"] is not None]
        strict_pass_count = sum(1 for row in strict_defined if row["strict_pass"])
        summary_row = {
            "tool": tool,
            "records": len(rows),
            "pass_rate": safe_fraction(pass_count, len(rows)),
            "median_runtime_s": safe_median(runtimes),
            "reads_invalid_total": sum(int(row["reads_invalid"]) for row in rows),
            "strict_pass_rate": safe_fraction(strict_pass_count, len(strict_defined)),
        }
        tool_summary.append(summary_row)

    fastest = min(
        (row for row in tool_summary if row["median_runtime_s"] is not None),
        key=lambda row: row["median_runtime_s"],
        default=None,
    )
    highest_pass = max(
        (row for row in tool_summary if row["pass_rate"] is not None),
        key=lambda row: row["pass_rate"],
        default=None,
    )
    largest_invalid = max(
        tool_summary,
        key=lambda row: row["reads_invalid_total"],
        default=None,
    )

    summary = {
        "schema_version": "bijux.fastq.validate_reads.corpus_summary.v1",
        "stage_id": VALIDATE_READS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": VALIDATE_READS_BENCHMARK_CONTRACT.scenario_id,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "tools": run_manifest["tools"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "tool_summary": tool_summary,
        "headline": {
            "fastest_tool": fastest["tool"] if fastest else None,
            "fastest_runtime_s": fastest["median_runtime_s"] if fastest else None,
            "highest_pass_rate_tool": highest_pass["tool"] if highest_pass else None,
            "highest_pass_rate": highest_pass["pass_rate"] if highest_pass else None,
            "largest_invalid_read_tool": largest_invalid["tool"] if largest_invalid else None,
            "largest_invalid_read_count": largest_invalid["reads_invalid_total"]
            if largest_invalid
            else None,
        },
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
            "reads_invalid",
            "reads_total",
            "strict_pass",
            "pair_sync_pass",
        ],
        summary_sort_keys=True,
    )


def main() -> int:
    return run_corpus_report(
        parse_args(),
        contract=VALIDATE_READS_BENCHMARK_CONTRACT,
        metadata_fallback_stage_id=VALIDATE_READS_BENCHMARK_CONTRACT.stage_id,
        build_artifacts=build_artifacts,
    )


if __name__ == "__main__":
    raise SystemExit(main())
