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
    fmt_metric,
    fmt_runtime,
    fmt_value,
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
        description="Render an enriched benchmark briefing from corpus-01 normalize-primers artifacts.",
        docs_root="docs/benchmark/fastq.normalize_primers/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('primer_trimmed_fraction', 'mean_primer_trimmed_fraction', 'mean'),
            BriefingMetricSpec('orientation_forward_fraction', 'median_orientation_forward_fraction', 'median'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('primer_trimmed_fraction', 'mean_primer_trimmed_fraction', 'mean'),
            BriefingMetricSpec('orientation_forward_fraction', 'median_orientation_forward_fraction', 'median'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('primer_trimmed_fraction', 'max', 'strongest_trim_tool', 'strongest_trim_fraction'),
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
    strongest_trim = max(
        runtime_rows,
        key=lambda row: row["mean_primer_trimmed_fraction"],
    )
    most_forward = max(
        runtime_rows,
        key=lambda row: row["median_orientation_forward_fraction"],
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.normalize_primers` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.normalize_primers` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed primer-normalization backends were benchmarked across `{summary['samples_total']}` samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Highest mean primer-trimmed fraction: `{strongest_trim['tool']}` at `{fmt_metric(strongest_trim['mean_primer_trimmed_fraction'])}`."
    )
    lines.append(
        f"- Highest median forward-orientation fraction: `{most_forward['tool']}` at `{fmt_metric(most_forward['median_orientation_forward_fraction'])}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- primer_set_id: `{summary['primer_set_id']}`")
    lines.append(f"- orientation_policy: `{summary['orientation_policy']}`")
    lines.append(f"- max_mismatch_rate: `{summary['max_mismatch_rate']}`")
    lines.append(f"- min_overlap_bp: `{summary['min_overlap_bp']}`")
    lines.append(f"- strict_5p_anchor: `{summary['strict_5p_anchor']}`")
    lines.append(f"- allow_iupac_codes: `{summary['allow_iupac_codes']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['mean_primer_trimmed_fraction'])} | {fmt_metric(row['median_orientation_forward_fraction'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['mean_primer_trimmed_fraction'])} | {fmt_metric(row['median_orientation_forward_fraction'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest trim tool | Primer-trimmed fraction |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['strongest_trim_tool']}` | {fmt_metric(row['strongest_trim_fraction'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- Because `corpus-01` is not an amplicon challenge set, non-zero primer trimming here should be interpreted as false-positive behavior unless independently justified by sequence evidence."
    )
    lines.append(
        "- The published CSV artifacts keep the governed primer policy and per-sample outcomes explicit so future reruns can audit drift instead of relying on narrative summaries alone."
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
