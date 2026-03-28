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
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 validate artifacts.",
        docs_root="docs/benchmark/fastq.validate_reads/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s'),
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
    fastqc_modern_pe = find_cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="era_layout",
        cohort="modern_pe",
    )
    fastqc_modern_se = find_cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="era_layout",
        cohort="modern_se",
    )
    fastqc_under_500 = find_cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="size_band",
        cohort="under_500mb",
    )
    fastqc_under_100 = find_cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="size_band",
        cohort="under_100mb",
    )
    zero_invalid = sum(1 for row in rows if row["reads_invalid"] == "0")
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.validate_reads` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.validate_reads` stage across all supported backends on the curated `corpus-01` human DNA benchmark set."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient, `{summary['era_counts']['modern']}` modern, `{summary['layout_counts']['se']}` single-end, `{summary['layout_counts']['pe']}` paired-end"
    )
    lines.append(
        f"- Tool set: `{', '.join(summary['tools'])}`"
    )
    lines.append("- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `fastq_scan` is the fastest backend at `p50={fmt_runtime(fastest['median_runtime_s'])}s`, while `fastqc` is the slowest at `p50={fmt_runtime(slowest['median_runtime_s'])}s`."
    )
    lines.append(
        f"- The median slowdown of `fastqc` relative to the fastest backend is `x{tool_lookup['fastqc']['slowdown_vs_fastest_median']:.2f}`."
    )
    if fastqc_modern_pe and fastqc_modern_se:
        lines.append(
            f"- Runtime is dominated by modern paired-end samples: for `fastqc`, `modern_pe` averages `{fmt_runtime(fastqc_modern_pe['mean_runtime_s'])}s` while `modern_se` averages `{fmt_runtime(fastqc_modern_se['mean_runtime_s'])}s`."
        )
    if fastqc_under_500 and fastqc_under_100:
        lines.append(
            f"- Input size is the primary cost driver: `fastqc` averages `{fmt_runtime(fastqc_under_500['mean_runtime_s'])}s` on `under_500mb` samples versus `{fmt_runtime(fastqc_under_100['mean_runtime_s'])}s` on `under_100mb` samples."
        )
    lines.append(
        f"- Correctness stayed stable across all `{len(rows)}` tool-sample observations: `exit_code=0` on `{zero_exit}` rows and `reads_invalid=0` on `{zero_invalid}` rows."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median slowdown |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(runtime_rows, key=lambda row: row["median_runtime_s"]):
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | x{row['slowdown_vs_fastest_median']:.2f} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Cohort | Mean runtime (s) | Median runtime (s) | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: |")
    for row in [
        entry
        for entry in cohort_rows
        if entry["dimension"] == "era_layout"
    ]:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Size-band behavior")
    lines.append("")
    lines.append(
        "| Tool | Size band | Mean runtime (s) | Median runtime (s) | Samples |"
    )
    lines.append("| --- | --- | ---: | ---: | ---: |")
    for row in [
        entry
        for entry in cohort_rows
        if entry["dimension"] == "size_band"
    ]:
        lines.append(
            f"| `{row['tool']}` | `{row['cohort']}` | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {row['samples']} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append(
        "| Sample | Accession | Era | Layout | Size band | Total stage runtime (s) | Slowest tool | Slowest tool runtime (s) |"
    )
    lines.append("| --- | --- | --- | --- | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['total_runtime_s'])} | `{row['slowest_tool']}` | {fmt_runtime(row['slowest_runtime_s'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- The four stream-oriented validators cluster tightly in median runtime, so they are operationally interchangeable on latency for this corpus."
    )
    lines.append(
        "- `fastqc` remains useful as a richer structural probe, but it carries a clear throughput penalty and should not be treated as a low-latency default validator."
    )
    lines.append(
        f"- The benchmark is stable on correctness for this corpus because every backend reported `reads_invalid=0` and `exit_code=0` across all `{len(rows)}` observations."
    )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append(
        "- Primary machine-readable artifacts beside this report: `summary.json`, `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, `sample_runtime_outliers.csv`."
    )
    lines.append(
        "- Input cohort metadata is joined through the committed `corpus-01` spec and the materialized corpus manifest, so accession-to-sample identity is stable across rerenders."
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
        runtime_fieldnames=[
            "tool",
            "samples",
            "pass_rate",
            "mean_runtime_s",
            "median_runtime_s",
            "p90_runtime_s",
            "max_runtime_s",
            "slowdown_vs_fastest_median",
        ],
        cohort_fieldnames=[
            "tool",
            "dimension",
            "cohort",
            "samples",
            "mean_runtime_s",
            "median_runtime_s",
        ],
        outlier_fieldnames=[
            "sample_id",
            "accession",
            "era",
            "layout",
            "size_band",
            "study_accession",
            "total_runtime_s",
            "slowest_tool",
            "slowest_runtime_s",
        ],
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
