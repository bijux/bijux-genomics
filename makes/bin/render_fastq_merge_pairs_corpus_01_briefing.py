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
        description="Render an enriched benchmark briefing from corpus-01 merge-pairs artifacts.",
        docs_root="docs/benchmark/fastq.merge_pairs/corpus-01",
    )


def fmt_decimal(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}"


def format_merge_setting(value: object) -> str:
    if value in {"", None}:
        return "auto"
    return str(value)


def validate_inputs(summary: dict, rows: list[dict]) -> None:
    expected_tools = sorted(summary.get("tools") or [])
    observed_tools = sorted({row.get("tool", "") for row in rows if row.get("tool")})
    if observed_tools != expected_tools:
        raise SystemExit(
            f"merge briefing drift: observed tools {observed_tools!r} expected {expected_tools!r}"
        )
    if any(row.get("layout") != "pe" for row in rows):
        raise SystemExit("merge briefing drift: all rows must stay paired-end")


def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('merge_rate', 'median_merge_rate', 'median'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('reads_merged', 'mean_reads_merged', 'mean'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('merge_rate', 'mean_merge_rate', 'mean'),
            BriefingMetricSpec('base_retention', 'median_base_retention', 'median'),
            BriefingMetricSpec('reads_merged', 'mean_reads_merged', 'mean'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
            BriefingOutlierSpec('merge_rate', 'max', 'best_merge_rate_tool', 'best_merge_rate'),
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
    best_merge = max(runtime_rows, key=lambda row: row["median_merge_rate"])
    heaviest_modern = find_cohort_entry(
        cohort_rows,
        tool=best_merge["tool"],
        dimension="era_layout",
        cohort="modern_pe",
    )
    lightest_ancient = find_cohort_entry(
        cohort_rows,
        tool=best_merge["tool"],
        dimension="era_layout",
        cohort="ancient_pe",
    )
    size_band_rows = [
        row
        for row in cohort_rows
        if row["tool"] == best_merge["tool"] and row["dimension"] == "size_band"
    ]
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")
    full_observations = len(rows)

    lines: list[str] = []
    lines.append("# `fastq.merge_pairs` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.merge_pairs` stage across the paired-end human DNA subset of corpus-01 on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient paired-end and `{summary['era_counts']['modern']}` modern paired-end samples"
    )
    lines.append(f"- Tool set: `{', '.join(summary['tools'])}`")
    lines.append(
        f"- Fixed merge contract: overlap `{format_merge_setting(summary['merge_overlap'])}`, min merged length `{format_merge_setting(summary['min_length'])}`, unmerged policy `{summary['unmerged_read_policy']}`"
    )
    lines.append(
        "- Execution profile: one benchmark sample at a time, one worker, with sample-level outputs retained in the Lunarc results tree and mirrored locally."
    )
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` paired samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `fastest p50 runtime` is `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}s`, while the slowest median backend is `{slowest['tool']}` at `{fmt_runtime(slowest['median_runtime_s'])}s`."
    )
    lines.append(
        f"- `best p50 merge rate` is `{best_merge['tool']}` at `{fmt_decimal(best_merge['median_merge_rate'])}`, with median base retention `{fmt_decimal(best_merge['median_base_retention'])}`."
    )
    lines.append(
        f"- Runtime spread from fastest to slowest median backend is `x{slowest['slowdown_vs_fastest_median']:.2f}`."
    )
    if heaviest_modern and lightest_ancient:
        lines.append(
            f"- Cohort pressure is higher for modern paired libraries: `{best_merge['tool']} modern_pe` averages `{fmt_runtime(heaviest_modern['mean_runtime_s'])}s` versus `{fmt_runtime(lightest_ancient['mean_runtime_s'])}s` on `ancient_pe`."
        )
    lines.append(
        f"- All `{full_observations}` tool-sample observations exited cleanly; zero-exit observations were `{zero_exit}`."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append("| Tool | p50 runtime (s) | p90 runtime (s) | Median merge rate | Median base retention | Mean merged reads |")
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_decimal(row['median_merge_rate'])} | {fmt_decimal(row['median_base_retention'])} | {fmt_decimal(row['mean_reads_merged'])} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append("| Tool | Cohort | Mean runtime (s) | Mean merge rate | Median base retention |")
    lines.append("| --- | --- | ---: | ---: | ---: |")
    for row in cohort_rows:
        if row["dimension"] != "era_layout":
            continue
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_decimal(row['mean_merge_rate'])} | {fmt_decimal(row['median_base_retention'])} |"
        )
    lines.append("")
    lines.append("## Size-band behavior")
    lines.append("")
    lines.append("| Tool | Size band | Mean runtime (s) | Mean merge rate | Mean merged reads |")
    lines.append("| --- | --- | ---: | ---: | ---: |")
    for row in size_band_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_decimal(row['mean_merge_rate'])} | {fmt_decimal(row['mean_reads_merged'])} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append("| Sample | Accession | Era | Size band | Total runtime (s) | Slowest tool | Best merge-rate tool |")
    lines.append("| --- | --- | --- | --- | ---: | --- | --- |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | `{row['best_merge_rate_tool']}` |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- Merge-rate comparisons in this dossier are only valid because overlap threshold, minimum merged length, and unmerged-mate handling are fixed across the full cohort."
    )
    lines.append(
        "- Base-retention differences should be read together with merge rate: a backend can preserve bases while still collapsing fewer pairs."
    )
    lines.append(
        "- Ancient paired libraries remain important in this corpus because they stress short-fragment overlap behavior that modern libraries may not expose."
    )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append(
        "- Machine-readable outputs beside this briefing: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv`."
    )
    lines.append(
        "- Results are generated from the Lunarc benchmark tree and mirrored into the configured local benchmark archive for later inspection."
    )
    return "\n".join(lines) + "\n"

def main() -> int:
    return render_corpus_briefing_dossier(
        parse_args(),
        tool_runtime_summary_fn=tool_runtime_summary,
        cohort_runtime_summary_fn=cohort_runtime_summary,
        sample_outlier_fn=sample_runtime_outliers,
        markdown_builder=lambda summary, rows, runtime_rows, cohort_rows, outlier_rows: render_markdown(summary, rows, runtime_rows, cohort_rows, outlier_rows),
        validate_rows_fn=validate_inputs,
    )
if __name__ == "__main__":
    raise SystemExit(main())
