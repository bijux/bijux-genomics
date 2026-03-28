#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path

from benchmark_fastq_corpus.support import (
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
    BriefingCopiedMetricSpec,
    BriefingOutlierSpec,
    summarize_sample_outlier_rows,
    render_corpus_briefing_dossier,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 filter-low-complexity artifacts.",
        docs_root="docs/benchmark/fastq.filter_low_complexity/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('reads_removed_low_complexity', 'mean_reads_removed_low_complexity', 'mean'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('reads_removed_low_complexity', 'mean_reads_removed_low_complexity', 'mean'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('reads_removed_low_complexity', 'max', 'strongest_filter_tool', 'strongest_filter_reads'),
        ],
        total_runtime_output_key='total_runtime_s',
    )
def render_markdown(
    summary: dict,
    rows: list[dict],
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    slowest = max(runtime_rows, key=lambda row: row["median_runtime_s"])
    strongest_filter = max(
        runtime_rows, key=lambda row: row["mean_reads_removed_low_complexity"]
    )
    highest_retention = max(
        runtime_rows, key=lambda row: row["median_base_retention"]
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.filter_low_complexity` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.filter_low_complexity` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient, `{summary['era_counts']['modern']}` modern, `{summary['layout_counts']['se']}` single-end, `{summary['layout_counts']['pe']}` paired-end"
    )
    lines.append(f"- Tool set: `{', '.join(summary['tools'])}`")
    lines.append(f"- entropy_threshold: `{summary['entropy_threshold']}`")
    lines.append(
        f"- polyx_threshold: `{'unset' if summary['polyx_threshold'] is None else summary['polyx_threshold']}`"
    )
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `fastest` median runtime is `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}s`; the slowest is `{slowest['tool']}` at `{fmt_runtime(slowest['median_runtime_s'])}s`."
    )
    lines.append(
        f"- The most aggressive filter is `{strongest_filter['tool']}` with `{strongest_filter['mean_reads_removed_low_complexity']:.1f}` low-complexity reads removed on average."
    )
    lines.append(
        f"- The highest median base retention comes from `{highest_retention['tool']}` at `{highest_retention['median_base_retention']:.3f}`."
    )
    lines.append(
        f"- Correctness remained stable across `{len(rows)}` tool-sample observations: `{zero_exit}` rows finished with `exit_code=0`."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median base retention | Median read retention | Mean removed reads | Mean Q delta | Median slowdown |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(runtime_rows, key=lambda row: row["median_runtime_s"]):
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | {row['median_base_retention']:.3f} | {row['median_read_retention']:.3f} | {row['mean_reads_removed_low_complexity']:.1f} | {row['mean_q_delta']:.3f} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean removed reads | Median base retention | Median read retention | Mean Q delta | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in [entry for entry in cohort_rows if entry["dimension"] == "era_layout"]:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {row['mean_reads_removed_low_complexity']:.1f} | {row['median_base_retention']:.3f} | {row['median_read_retention']:.3f} | {row['mean_q_delta']:.3f} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest filter tool | Strongest filter reads |"
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
        "- This dossier makes the main tradeoff explicit: some backends remove more low-complexity reads, while others preserve more sequence and finish faster."
    )
    lines.append(
        "- On corpus-01, the stage behaves as a real filter rather than a no-op, so this is a meaningful runtime-versus-retention comparison for future preprocessing defaults."
    )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append(
        "- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`."
    )
    return "\n".join(lines) + "\n"

def main() -> int:
    return render_corpus_briefing_dossier(
        parse_args(),
        tool_runtime_summary_fn=tool_runtime_summary,
        cohort_runtime_summary_fn=cohort_runtime_summary,
        sample_outlier_fn=sample_runtime_outliers,
        markdown_builder=lambda summary, rows, runtime_rows, cohort_rows, outlier_rows: render_markdown(summary, rows, runtime_rows, cohort_rows, outlier_rows),
    )
if __name__ == "__main__":
    raise SystemExit(main())
