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
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 extract-umis artifacts.",
        docs_root="docs/benchmark/fastq.extract_umis/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('reads_with_umi', 'mean_reads_with_umi', 'mean'),
            BriefingMetricSpec('reads_with_umi_fraction', 'mean_reads_with_umi_fraction', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('reads_with_umi_fraction', 'mean_reads_with_umi_fraction', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest = max(
            sample_rows, key=lambda row: float(row["reads_with_umi_fraction"])
        )
        output.append(
            {
                "sample_id": sample_id,
                "accession": sample_rows[0]["accession"],
                "era": sample_rows[0]["era"],
                "layout": sample_rows[0]["layout"],
                "size_band": sample_rows[0]["size_band"],
                "study_accession": sample_rows[0]["study_accession"],
                "total_stage_runtime_s": sum(
                    float(row["runtime_s"]) for row in sample_rows
                ),
                "slowest_tool": slowest["tool"],
                "slowest_runtime_s": float(slowest["runtime_s"]),
                "strongest_tool": strongest["tool"],
                "reads_with_umi_fraction": float(strongest["reads_with_umi_fraction"]),
            }
        )
    output.sort(key=lambda row: row["total_stage_runtime_s"], reverse=True)
    return output

def render_markdown(
    summary: dict,
    rows: list[dict],
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    best_detection = max(
        runtime_rows, key=lambda row: row["mean_reads_with_umi_fraction"]
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.extract_umis` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.extract_umis` stage across the paired subset of the `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed UMI extractor was benchmarked across `{summary['samples_total']}` paired samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Mean reads-with-UMI fraction: `{best_detection['tool']}` at `{fmt_metric(best_detection['mean_reads_with_umi_fraction'])}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- umi_pattern: `{summary['umi_pattern']}`")
    lines.append(
        f"- allow_missing_umi_headers: `{summary['allow_missing_umi_headers']}`"
    )
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Mean reads with UMI | Mean reads with UMI fraction | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['mean_reads_with_umi'])} | {fmt_metric(row['mean_reads_with_umi_fraction'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Median read retention | Mean reads with UMI fraction | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['mean_reads_with_umi_fraction'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Reads-with-UMI fraction |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_stage_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | {fmt_metric(row['reads_with_umi_fraction'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- This paired-only briefing keeps the UMI pattern explicit so later barcode-policy changes cannot masquerade as benchmark regressions."
    )
    lines.append(
        "- Missing-header bypass is recorded in the run contract because `corpus-01` is a human DNA cohort rather than a native UMI corpus."
    )
    lines.append(
        "- The per-sample CSV artifacts make it easy to inspect whether runtime outliers coincide with weaker read retention or weaker UMI detection."
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
