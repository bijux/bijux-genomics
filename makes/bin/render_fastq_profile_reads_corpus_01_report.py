#!/usr/bin/env python3
from __future__ import annotations

import argparse
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from benchmark_fastq_corpus.support import (
    parse_corpus_report_args,
    CorpusReportArtifacts,
    CorpusReportContext,
    PROFILE_READS_BENCHMARK_CONTRACT,
    load_json,
    localize_results_path,
    run_corpus_report,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.profile_reads benchmark summary.",
        docs_root="docs/benchmark/fastq.profile_reads/corpus-01",
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


def validate_profile_reads_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "profile-reads benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "profile-reads benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != PROFILE_READS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "profile-reads benchmark report drift: "
            f"expected stage_id {PROFILE_READS_BENCHMARK_CONTRACT.stage_id}, found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != PROFILE_READS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "profile-reads benchmark report drift: "
            f"expected scenario_id {PROFILE_READS_BENCHMARK_CONTRACT.scenario_id}, found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind") != "benchmark":
        raise SystemExit(
            "profile-reads benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    expected_contract = {
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
        "raw_backend_report_format": "seqkit_stats_tsv",
        "length_histogram_source": "seqkit_fx2tab",
    }
    for key, expected in expected_contract.items():
        if run_manifest.get(key) != expected:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"expected {key}={expected!r}, found {run_manifest.get(key)!r}"
            )


def derived_histogram_metrics(length_histogram: list[dict]) -> dict[str, float | int | None]:
    if not length_histogram:
        return {
            "histogram_bin_count": 0,
            "max_observed_length": None,
            "mean_read_length": None,
        }
    total_reads = sum(int(bin_row["count"]) for bin_row in length_histogram)
    total_bases = sum(int(bin_row["length"]) * int(bin_row["count"]) for bin_row in length_histogram)
    return {
        "histogram_bin_count": len(length_histogram),
        "max_observed_length": max(int(bin_row["length"]) for bin_row in length_histogram),
        "mean_read_length": (total_bases / total_reads) if total_reads else None,
    }


def validate_profile_reads_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if int(row["reads_total"]) <= 0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"reads_total must be positive for {row['sample_id']}/{row['tool']}"
            )
        if int(row["bases_total"]) <= 0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"bases_total must be positive for {row['sample_id']}/{row['tool']}"
            )
        if not 0.0 <= float(row["mean_q"]) <= 45.0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"mean_q must be within [0, 45] for {row['sample_id']}/{row['tool']}"
            )
        if not 0.0 <= float(row["gc_percent"]) <= 100.0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"gc_percent must be within [0, 100] for {row['sample_id']}/{row['tool']}"
            )
        if int(row["histogram_bin_count"]) <= 0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"histogram_bin_count must be positive for {row['sample_id']}/{row['tool']}"
            )
        mean_read_length = row["mean_read_length"]
        max_observed_length = row["max_observed_length"]
        if mean_read_length is None or float(mean_read_length) <= 0.0:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"mean_read_length must be positive for {row['sample_id']}/{row['tool']}"
            )
        if max_observed_length is None or float(max_observed_length) < float(mean_read_length):
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"max_observed_length must be >= mean_read_length for {row['sample_id']}/{row['tool']}"
            )
        expected_mean_read_length = float(row["bases_total"]) / float(row["reads_total"])
        if abs(expected_mean_read_length - float(mean_read_length)) > 1e-6:
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"mean_read_length must equal bases_total/reads_total for {row['sample_id']}/{row['tool']}"
            )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "profile-reads benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.profile_reads` on `corpus-01`")
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
    lines.append(f"- raw_backend_report_format: `{summary['raw_backend_report_format']}`")
    lines.append(f"- length_histogram_source: `{summary['length_histogram_source']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("highest_mean_q_tool"):
        lines.append(
            f"- Highest median mean Q: `{headline['highest_mean_q_tool']}` at `{headline['highest_mean_q']:.3f}`."
        )
    if headline.get("widest_histogram_tool"):
        lines.append(
            f"- Widest median histogram support: `{headline['widest_histogram_tool']}` at `{headline['widest_histogram_bins']}` bins."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median reads | Median bases | Median mean Q | Median GC % | Median read length | Median histogram bins |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        reads_total = "n/a" if row["median_reads_total"] is None else f"{row['median_reads_total']:.1f}"
        bases_total = "n/a" if row["median_bases_total"] is None else f"{row['median_bases_total']:.1f}"
        mean_q = "n/a" if row["median_mean_q"] is None else f"{row['median_mean_q']:.3f}"
        gc_percent = "n/a" if row["median_gc_percent"] is None else f"{row['median_gc_percent']:.3f}"
        read_length = "n/a" if row["median_read_length"] is None else f"{row['median_read_length']:.3f}"
        histogram_bins = "n/a" if row["median_histogram_bin_count"] is None else f"{row['median_histogram_bin_count']:.1f}"
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {reads_total} | {bases_total} | {mean_q} | {gc_percent} | {read_length} | {histogram_bins} |"
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
        "- This stage is report-only and non-mutating: governed benchmarking confirms runtime and profile-report stability without changing the reads."
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
                "profile-reads benchmark report drift: "
                f"missing report.json for {sample_id}: {report_path}"
            )
        report = load_json(report_path)
        if not report.get("records"):
            raise SystemExit(
                "profile-reads benchmark report drift: "
                f"report.json for {sample_id} contains no records"
            )
        for record in report.get("records", []):
            tool = record.get("context", {}).get("tool", "unknown")
            length_histogram = normalize_metric(record, "length_histogram") or []
            derived = derived_histogram_metrics(length_histogram)
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
                "reads_total": normalize_metric(record, "reads_total") or 0,
                "bases_total": normalize_metric(record, "bases_total") or 0,
                "mean_q": normalize_metric(record, "mean_q") or 0.0,
                "gc_percent": normalize_metric(record, "gc_percent") or 0.0,
                "histogram_bin_count": derived["histogram_bin_count"],
                "max_observed_length": derived["max_observed_length"],
                "mean_read_length": derived["mean_read_length"],
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_profile_reads_row_contract(
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
                "median_reads_total": safe_median([float(row["reads_total"]) for row in rows]),
                "median_bases_total": safe_median([float(row["bases_total"]) for row in rows]),
                "median_mean_q": safe_median([float(row["mean_q"]) for row in rows]),
                "median_gc_percent": safe_median([float(row["gc_percent"]) for row in rows]),
                "median_read_length": safe_median(
                    [
                        float(row["mean_read_length"])
                        for row in rows
                        if row["mean_read_length"] is not None
                    ]
                ),
                "median_histogram_bin_count": safe_median(
                    [float(row["histogram_bin_count"]) for row in rows]
                ),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        highest_mean_q = max(
            (row for row in tool_summary if row["median_mean_q"] is not None),
            key=lambda row: row["median_mean_q"],
        )
        widest_histogram = max(
            (row for row in tool_summary if row["median_histogram_bin_count"] is not None),
            key=lambda row: row["median_histogram_bin_count"],
        )
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "highest_mean_q_tool": highest_mean_q["tool"],
            "highest_mean_q": highest_mean_q["median_mean_q"],
            "widest_histogram_tool": widest_histogram["tool"],
            "widest_histogram_bins": widest_histogram["median_histogram_bin_count"],
        }

    summary = {
        "schema_version": "bijux.fastq.profile_reads.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": PROFILE_READS_BENCHMARK_CONTRACT.stage_id,
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
        "raw_backend_report_format": run_manifest["raw_backend_report_format"],
        "length_histogram_source": run_manifest["length_histogram_source"],
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
            "reads_total",
            "bases_total",
            "mean_q",
            "gc_percent",
            "histogram_bin_count",
            "max_observed_length",
            "mean_read_length",
        ],
        summary_sort_keys=True,
    )


def main() -> int:
    return run_corpus_report(
        parse_args(),
        contract=PROFILE_READS_BENCHMARK_CONTRACT,
        validate_run_manifest=validate_profile_reads_run_manifest_contract,
        metadata_fallback_stage_id=PROFILE_READS_BENCHMARK_CONTRACT.stage_id,
        build_artifacts=build_artifacts,
    )


if __name__ == "__main__":
    raise SystemExit(main())
