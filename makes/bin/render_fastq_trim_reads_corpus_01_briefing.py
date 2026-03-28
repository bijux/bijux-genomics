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
        description="Render an enriched benchmark briefing from corpus-01 trim-reads artifacts.",
        docs_root="docs/benchmark/fastq.trim_reads/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('mean_q_delta', 'mean_q_delta', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
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
            BriefingOutlierSpec('base_retention', 'min', 'lowest_retention_tool', 'lowest_base_retention'),
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
    def format_trim_setting(value):
        return "governed tool default" if value is None else value

    def format_slowdown(value):
        return "n/a" if value is None else f"{value:.2f}x"

    tool_lookup = {row["tool"]: row for row in runtime_rows}
    ranked_runtime_rows = [
        row for row in runtime_rows if row["median_runtime_s"] is not None
    ]
    fastest = min(ranked_runtime_rows, key=lambda row: row["median_runtime_s"])
    slowest = max(ranked_runtime_rows, key=lambda row: row["median_runtime_s"])
    best_retention = max(runtime_rows, key=lambda row: row["median_base_retention"])
    lowest_retention = min(runtime_rows, key=lambda row: row["median_base_retention"])
    modern_pe_fastp = find_cohort_entry(
        cohort_rows,
        tool="fastp",
        dimension="era_layout",
        cohort="modern_pe",
    )
    ancient_se_fastp = find_cohort_entry(
        cohort_rows,
        tool="fastp",
        dimension="era_layout",
        cohort="ancient_se",
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.trim_reads` on `corpus-01`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed trim backends were benchmarked across `{summary['samples_total']}` human samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Slowest median runtime: `{slowest['tool']}` at `{fmt_runtime(slowest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Highest median base retention: `{best_retention['tool']}` at `{best_retention['median_base_retention']:.3f}`."
    )
    lines.append(
        f"- Lowest median base retention: `{lowest_retention['tool']}` at `{lowest_retention['median_base_retention']:.3f}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- min_length: `{format_trim_setting(summary['min_length'])}`")
    lines.append(
        f"- quality_cutoff: `{format_trim_setting(summary['quality_cutoff'])}`"
    )
    lines.append(f"- n_policy: `{summary['n_policy']}`")
    lines.append(f"- adapter_policy: `{summary['adapter_policy']}`")
    lines.append(f"- polyx_policy: `{summary['polyx_policy']}`")
    lines.append(f"- contaminant_policy: `{summary['contaminant_policy']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Median runtime (s) | p90 runtime (s) | Median base retention | Median read retention | Mean Q delta | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(
        runtime_rows,
        key=lambda item: (item["median_runtime_s"] is None, item["median_runtime_s"] or 0.0),
    ):
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {row['median_base_retention']:.3f} | {row['median_read_retention']:.3f} | {row['mean_q_delta']:.3f} | {format_slowdown(row['slowdown_vs_fastest_median'])} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    if modern_pe_fastp and ancient_se_fastp:
        lines.append(
            f"- For `fastp`, `modern_pe` samples ran at `{fmt_runtime(modern_pe_fastp['median_runtime_s'])}` seconds median versus `{fmt_runtime(ancient_se_fastp['median_runtime_s'])}` for `ancient_se`."
        )
    lines.append(
        f"- The fastest backend `{fastest['tool']}` and highest-retention backend `{best_retention['tool']}` are {'the same' if fastest['tool'] == best_retention['tool'] else 'different'}, which matters when choosing a default objective."
    )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append("| Sample | Accession | Cohort | Total runtime (s) | Slowest tool | Slowest runtime (s) | Lowest-retention tool | Lowest base retention |")
    lines.append("| --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}_{row['layout']}` | {row['total_runtime_s']:.3f} | `{row['slowest_tool']}` | {row['slowest_runtime_s']:.3f} | `{row['lowest_retention_tool']}` | {row['lowest_base_retention']:.3f} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- This corpus benchmark is intentionally bank-free for trim adapters/polyX/contaminants so the governed fairness cohort stays comparable across all included backends."
    )
    lines.append(
        "- The strongest choice for production depends on whether we prefer latency, base retention, or quality uplift; this dossier exposes those tradeoffs instead of collapsing them into one score."
    )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append("- `summary.json`, `sample_results.csv`, and the CSV analysis tables in this directory are generated artifacts from the same corpus run.")
    lines.append("- `sample_results.csv` preserves one row per sample/tool execution for independent reanalysis.")
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
