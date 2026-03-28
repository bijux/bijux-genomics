#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    fmt_csv_value,
    fmt_fraction,
    fmt_runtime,
    load_csv_rows,
    load_json,
    parse_corpus_briefing_args,
    publish_corpus_briefing_artifacts,
    percentile,
    resolve_corpus_briefing_runtime,
    iter_cohort_row_groups,
    safe_mean,
    safe_median,
    BriefingMetricSpec,
    summarize_tool_runtime_rows,
    summarize_cohort_metric_rows,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 filter-reads artifacts.",
        docs_root="docs/benchmark/fastq.filter_reads/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('reads_dropped', 'mean_reads_dropped', 'mean'),
            BriefingMetricSpec('reads_removed_low_complexity', 'mean_reads_removed_low_complexity', 'mean'),
            BriefingMetricSpec('reads_removed_by_n', 'mean_reads_removed_by_n', 'mean'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('reads_dropped', 'mean_reads_dropped', 'mean'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest_filter = max(sample_rows, key=lambda row: float(row["reads_dropped"]))
        output.append(
            {
                "sample_id": sample_id,
                "accession": sample_rows[0]["accession"],
                "era": sample_rows[0]["era"],
                "layout": sample_rows[0]["layout"],
                "size_band": sample_rows[0]["size_band"],
                "study_accession": sample_rows[0]["study_accession"],
                "total_runtime_s": sum(float(row["runtime_s"]) for row in sample_rows),
                "slowest_tool": slowest["tool"],
                "slowest_runtime_s": float(slowest["runtime_s"]),
                "strongest_filter_tool": strongest_filter["tool"],
                "strongest_filter_reads": float(strongest_filter["reads_dropped"]),
            }
        )
    output.sort(key=lambda row: row["total_runtime_s"], reverse=True)
    return output

def render_markdown(
    summary: dict,
    rows: list[dict],
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    strongest_filter = max(runtime_rows, key=lambda row: row["mean_reads_dropped"])
    highest_retention = max(
        runtime_rows, key=lambda row: row["median_base_retention"]
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.filter_reads` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.filter_reads` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed filter backends were benchmarked across `{summary['samples_total']}` samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Highest median base retention: `{highest_retention['tool']}` at `{fmt_fraction(highest_retention['median_base_retention'])}`."
    )
    lines.append(
        f"- Highest mean reads dropped: `{strongest_filter['tool']}` at `{strongest_filter['mean_reads_dropped']:.1f}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- max_n: `{summary['max_n']}`")
    lines.append(f"- max_n_count: `{summary['max_n_count']}`")
    lines.append(
        f"- low_complexity_threshold: `{summary['low_complexity_threshold']}`"
    )
    lines.append(f"- entropy_threshold: `{summary['entropy_threshold']}`")
    lines.append(f"- kmer_ref: `{summary['kmer_ref']}`")
    lines.append(f"- polyx_policy: `{summary['polyx_policy']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median base retention | Median read retention | Mean reads dropped | Mean low-complexity removals | Mean N removals | Mean Q delta | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_fraction(row['median_base_retention'])} | {fmt_fraction(row['median_read_retention'])} | {row['mean_reads_dropped']:.1f} | {row['mean_reads_removed_low_complexity']:.1f} | {row['mean_reads_removed_by_n']:.1f} | {row['mean_q_delta']:.3f} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median base retention | Median read retention | Mean reads dropped | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_fraction(row['median_base_retention'])} | {fmt_fraction(row['median_read_retention'])} | {row['mean_reads_dropped']:.1f} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest filter tool | Reads dropped |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['strongest_filter_tool']}` | {row['strongest_filter_reads']:.1f} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- Because `corpus-01` is a human DNA cohort rather than a synthetic junk-read challenge set, aggressive filtering here should be weighed carefully against retention loss."
    )
    lines.append(
        "- The published CSV artifacts preserve per-tool removal counters so later audits can distinguish N-filtering, low-complexity filtering, and backend-specific retention behavior."
    )
    return "\n".join(lines) + "\n"

def main() -> int:
    runtime = resolve_corpus_briefing_runtime(parse_args())
    docs_root = runtime.docs_root
    summary = runtime.summary
    rows = runtime.sample_rows

    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outliers = sample_runtime_outliers(rows)

    publish_corpus_briefing_artifacts(
        docs_root,
        markdown=render_markdown(summary, rows, runtime_rows, cohort_rows, outliers),
        runtime_rows=runtime_rows,
        cohort_rows=cohort_rows,
        outlier_rows=outliers,
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
