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
        description="Render an enriched benchmark briefing from corpus-01 correct-errors artifacts.",
        docs_root="docs/benchmark/fastq.correct_errors/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('read_retention', 'median_read_retention', 'median'),
            BriefingMetricSpec('corrected_reads', 'mean_corrected_reads', 'mean'),
            BriefingMetricSpec('kmer_fix_rate', 'mean_kmer_fix_rate', 'mean'),
            BriefingMetricSpec('mean_q_delta', 'mean_quality_uplift', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('kmer_fix_rate', 'mean_kmer_fix_rate', 'mean'),
            BriefingMetricSpec('mean_q_delta', 'mean_quality_uplift', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest = max(sample_rows, key=lambda row: float(row["mean_q_delta"]))
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
                "strongest_quality_uplift": float(strongest["mean_q_delta"]),
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
    best_kmer_fix = max(runtime_rows, key=lambda row: row["mean_kmer_fix_rate"])
    best_quality_uplift = max(
        runtime_rows, key=lambda row: row["mean_quality_uplift"]
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.correct_errors` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.correct_errors` stage across the paired-end `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `{len(runtime_rows)}` governed correction backends were benchmarked across `{summary['samples_total']}` samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Highest mean k-mer fix rate: `{best_kmer_fix['tool']}` at `{fmt_metric(best_kmer_fix['mean_kmer_fix_rate'])}`."
    )
    lines.append(
        f"- Highest mean quality uplift: `{best_quality_uplift['tool']}` at `{fmt_metric(best_quality_uplift['mean_quality_uplift'])}`."
    )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- quality_encoding: `{summary['quality_encoding']}`")
    lines.append(f"- kmer_size: `{summary['kmer_size']}`")
    lines.append(f"- musket_kmer_budget: `{summary['musket_kmer_budget']}`")
    lines.append(f"- genome_size: `{summary['genome_size']}`")
    lines.append(f"- max_memory_gb: `{summary['max_memory_gb']}`")
    lines.append(f"- trusted_kmer_artifact_digest: `{summary['trusted_kmer_artifact_digest']}`")
    lines.append(f"- conservative_mode: `{summary['conservative_mode']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median read retention | Mean corrected reads | Mean k-mer fix rate | Mean quality uplift | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_metric(row['median_read_retention'])} | {fmt_metric(row['mean_corrected_reads'])} | {fmt_metric(row['mean_kmer_fix_rate'])} | {fmt_metric(row['mean_quality_uplift'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Mean k-mer fix rate | Mean quality uplift | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_metric(row['mean_kmer_fix_rate'])} | {fmt_metric(row['mean_quality_uplift'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) | Strongest quality-uplift tool | Quality uplift |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_stage_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} | `{row['strongest_tool']}` | {fmt_metric(row['strongest_quality_uplift'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- Correction stages often keep read counts constant, so this briefing emphasizes k-mer-fix rate and quality uplift instead of pretending count preservation means identical outputs."
    )
    lines.append(
        "- The published CSV artifacts keep the correction contract explicit so later reruns can spot policy drift without reverse-engineering runner defaults."
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
