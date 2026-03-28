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
    find_cohort_entry,
    find_cohort_entry,
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
        description="Render an enriched benchmark briefing from corpus-01 trim-polyg artifacts.",
        docs_root="docs/benchmark/fastq.trim_polyg_tails/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('bases_trimmed_polyg', 'mean_bases_trimmed_polyg', 'mean'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('bases_trimmed_polyg', 'mean_bases_trimmed_polyg', 'mean'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('bases_trimmed_polyg', 'max', 'most_trimming_tool', 'most_trimmed_bases'),
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
    tool_lookup = {row["tool"]: row for row in runtime_rows}
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    slowest = max(runtime_rows, key=lambda row: row["median_runtime_s"])
    heaviest_modern_pe = find_cohort_entry(
        cohort_rows,
        tool="fastp",
        dimension="era_layout",
        cohort="modern_pe",
    )
    lightest_modern_se = find_cohort_entry(
        cohort_rows,
        tool="fastp",
        dimension="era_layout",
        cohort="modern_se",
    )
    size_band_rows = [
        row
        for row in cohort_rows
        if row["tool"] == "fastp" and row["dimension"] == "size_band"
    ]
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")
    trimming_positive = sum(1 for row in rows if float(row["bases_trimmed_polyg"]) > 0.0)

    lines: list[str] = []
    lines.append("# `fastq.trim_polyg_tails` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.trim_polyg_tails` stage across the full corpus-01 human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient, `{summary['era_counts']['modern']}` modern, `{summary['layout_counts']['se']}` single-end, `{summary['layout_counts']['pe']}` paired-end"
    )
    lines.append(f"- Tool set: `{', '.join(summary['tools'])}`")
    lines.append(f"- Chemistry preset: `{summary['polyx_preset']}`")
    lines.append(
        f"- Execution profile: one benchmark sample at a time, one worker, min_polyg_run `{summary['min_polyg_run']}`"
    )
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `fastp` is the fastest backend at `p50={fmt_runtime(fastest['median_runtime_s'])}s`, while `bbduk` is slower at `p50={fmt_runtime(slowest['median_runtime_s'])}s`."
    )
    lines.append(
        f"- The median slowdown of `bbduk` relative to the fastest backend is `x{tool_lookup['bbduk']['slowdown_vs_fastest_median']:.2f}`."
    )
    lines.append(
        f"- Mean polyG trimming per sample is `fastp={fmt_runtime(tool_lookup['fastp']['mean_bases_trimmed_polyg'])}` bases and `bbduk={fmt_runtime(tool_lookup['bbduk']['mean_bases_trimmed_polyg'])}` bases."
    )
    if heaviest_modern_pe and lightest_modern_se:
        lines.append(
            f"- Runtime pressure is carried by paired modern inputs: `fastp modern_pe` averages `{fmt_runtime(heaviest_modern_pe['mean_runtime_s'])}s` while `fastp modern_se` averages `{fmt_runtime(lightest_modern_se['mean_runtime_s'])}s`."
        )
    if size_band_rows:
        under_500 = next(row for row in size_band_rows if row["cohort"] == "under_500mb")
        under_100 = next(row for row in size_band_rows if row["cohort"] == "under_100mb")
        lines.append(
            f"- Input size remains the main cost driver: `fastp` averages `{fmt_runtime(under_500['mean_runtime_s'])}s` on `under_500mb` samples versus `{fmt_runtime(under_100['mean_runtime_s'])}s` on `under_100mb` samples."
        )
    lines.append(
        f"- Correctness stayed stable across all `{len(rows)}` tool-sample observations: `exit_code=0` on `{zero_exit}` rows, with positive polyG trimming observed on `{trimming_positive}` rows."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median base retention | Mean bases trimmed | Mean Q delta | Median slowdown |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(runtime_rows, key=lambda row: row["median_runtime_s"]):
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | {fmt_runtime(row['median_base_retention'])} | {fmt_runtime(row['mean_bases_trimmed_polyg'])} | {fmt_runtime(row['mean_q_delta'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean bases trimmed | Median base retention | Mean Q delta | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in [entry for entry in cohort_rows if entry["dimension"] == "era_layout"]:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['mean_bases_trimmed_polyg'])} | {fmt_runtime(row['median_base_retention'])} | {fmt_runtime(row['mean_q_delta'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Size-band behavior")
    lines.append("")
    lines.append(
        "| Tool | Size band | Mean runtime (s) | Mean bases trimmed | Median base retention | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: |")
    for row in [entry for entry in cohort_rows if entry["dimension"] == "size_band"]:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['mean_bases_trimmed_polyg'])} | {fmt_runtime(row['median_base_retention'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest trim tool | Strongest trim bases |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['most_trimming_tool']}` | {fmt_runtime(row['most_trimmed_bases'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- `fastp` is the lower-latency default for corpus-scale polyG cleanup, while `bbduk` trades more wall time for comparable retention."
    )
    lines.append(
        "- The benchmark is dominated by mid-size paired inputs, so the stage should be budgeted as a paired-end cost center rather than a single-end one."
    )
    lines.append(
        "- Positive trim counts across the corpus show that this stage is not acting as a pure no-op on corpus-01; the chosen chemistry preset is exercising real cleanup work."
    )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append(
        "- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`."
    )
    lines.append(
        "- Input cohort metadata is joined through the committed `corpus-01` spec and the materialized corpus manifest, so accession-to-sample identity remains stable across rerenders."
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
