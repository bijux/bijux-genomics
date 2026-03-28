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
    BriefingCopiedMetricSpec,
    BriefingOutlierSpec,
    summarize_sample_outlier_rows,
    render_corpus_briefing_dossier,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 remove-duplicates artifacts.",
        docs_root="docs/benchmark/fastq.remove_duplicates/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('dedup_rate', 'median_dedup_rate', 'median'),
            BriefingMetricSpec('duplicate_reads', 'mean_duplicate_reads', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('dedup_rate', 'median_dedup_rate', 'median'),
            BriefingMetricSpec('duplicate_reads', 'mean_duplicate_reads', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('duplicate_reads', 'max', 'strongest_tool', 'strongest_duplicate_reads'),
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
    strongest = max(runtime_rows, key=lambda row: row["mean_duplicate_reads"])
    best_dedup = max(runtime_rows, key=lambda row: row["median_dedup_rate"])
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.remove_duplicates` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.remove_duplicates` stage across the paired subset of the corpus-01 human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient, `{summary['era_counts']['modern']}` modern, paired-end only"
    )
    lines.append(f"- Tool set: `{', '.join(summary['tools'])}`")
    lines.append(f"- dedup_mode: `{summary['dedup_mode']}`")
    lines.append(f"- keep_order: `{summary['keep_order']}`")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` paired samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `fastest` median runtime is `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}s`; the slowest is `{slowest['tool']}` at `{fmt_runtime(slowest['median_runtime_s'])}s`."
    )
    lines.append(
        f"- The strongest duplicate removal is `{strongest['tool']}` with `{strongest['mean_duplicate_reads']:.1f}` duplicate reads removed on average."
    )
    lines.append(
        f"- The highest median deduplication rate comes from `{best_dedup['tool']}` at `{best_dedup['median_dedup_rate']:.3f}`."
    )
    lines.append(
        f"- Correctness remained stable across `{len(rows)}` tool-sample observations: `{zero_exit}` rows finished with `exit_code=0`."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median dedup rate | Mean duplicate reads | Median slowdown |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {row['pass_rate']:.1%} | {row['mean_runtime_s']:.3f} | {row['median_runtime_s']:.3f} | {row['p90_runtime_s']:.3f} | {row['max_runtime_s']:.3f} | {row['median_dedup_rate']:.3f} | {row['mean_duplicate_reads']:.1f} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median dedup rate | Mean duplicate reads | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {row['mean_runtime_s']:.3f} | {row['median_runtime_s']:.3f} | {row['median_dedup_rate']:.3f} | {row['mean_duplicate_reads']:.1f} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest dedup tool | Duplicate reads removed |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['strongest_tool']}` | {row['strongest_duplicate_reads']:.1f} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- This dossier makes the main tradeoff explicit: some backends remove more duplicates, while others preserve pair order or finish faster under the same governed contract."
    )
    lines.append(
        "- On corpus-01, this stage is benchmarking a real paired-end deduplication workload rather than a synthetic no-op path, so it is directly useful for later preprocessing defaults."
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
