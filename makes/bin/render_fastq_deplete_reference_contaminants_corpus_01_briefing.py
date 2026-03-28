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
        description="Render an enriched benchmark briefing from corpus-01 deplete-reference-contaminants artifacts.",
        docs_root="docs/benchmark/fastq.deplete_reference_contaminants/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('contaminant_fraction_removed', 'mean_contaminant_fraction_removed', 'mean'),
            BriefingMetricSpec('reads_removed', 'mean_reads_removed', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('contaminant_fraction_removed', 'mean_contaminant_fraction_removed', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('contaminant_fraction_removed', 'max', 'strongest_depletion_tool', 'strongest_contaminant_fraction_removed'),
        ],
        total_runtime_output_key='total_stage_runtime_s',
    )
def render_markdown(
    summary: dict,
    rows: list[dict],
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    strongest = max(
        runtime_rows, key=lambda row: row["mean_contaminant_fraction_removed"]
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.deplete_reference_contaminants` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.deplete_reference_contaminants` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed contaminant-depletion backends were benchmarked across `{summary['samples_total']}` samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Highest mean contaminant fraction removed: `{strongest['tool']}` at `{fmt_metric(strongest['mean_contaminant_fraction_removed'])}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- reference_index_digest: `{summary['reference_index_digest']}`")
    lines.append(f"- reference_catalog_id: `{summary['reference_catalog_id']}`")
    lines.append(f"- reference_index_backend: `{summary['reference_index_backend']}`")
    lines.append(f"- decoy_mode: `{summary['decoy_mode']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Mean reads removed | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['median_base_retention'])} | {fmt_metric(row['mean_contaminant_fraction_removed'])} | {fmt_metric(row['mean_reads_removed'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['median_base_retention'])} | {fmt_metric(row['mean_contaminant_fraction_removed'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest depletion tool | Contaminant fraction removed |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_stage_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['strongest_depletion_tool']}` | {fmt_metric(row['strongest_contaminant_fraction_removed'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- Because this cohort is human DNA, substantial contaminant depletion usually signals aggressive technical over-removal unless independently justified by the reference set."
    )
    lines.append(
        "- The published CSV artifacts keep reference lineage and governed decoy policy explicit so later reruns can audit reference drift instead of relying on narrative summaries."
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
