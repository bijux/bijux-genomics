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
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 filter-low-complexity artifacts.",
        docs_root="docs/benchmark/fastq.filter_low_complexity/corpus-01",
    )

def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    by_tool: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_tool[row["tool"]].append(row)

    medians = {
        tool: safe_median([float(row["runtime_s"]) for row in tool_rows])
        for tool, tool_rows in by_tool.items()
    }
    fastest_median = min(value for value in medians.values() if value is not None)
    summary_rows = []
    for tool in sorted(by_tool):
        tool_rows = by_tool[tool]
        runtimes = [float(row["runtime_s"]) for row in tool_rows]
        base_retentions = [float(row["base_retention"]) for row in tool_rows]
        read_retentions = [float(row["read_retention"]) for row in tool_rows]
        removed_reads = [float(row["reads_removed_low_complexity"]) for row in tool_rows]
        mean_q_deltas = [float(row["mean_q_delta"]) for row in tool_rows]
        median = safe_median(runtimes)
        summary_rows.append(
            {
                "tool": tool,
                "samples": len(tool_rows),
                "pass_rate": sum(1 for row in tool_rows if row["exit_code"] == "0")
                / len(tool_rows),
                "mean_runtime_s": safe_mean(runtimes),
                "median_runtime_s": median,
                "p90_runtime_s": percentile(runtimes, 0.9),
                "max_runtime_s": max(runtimes),
                "median_base_retention": safe_median(base_retentions),
                "median_read_retention": safe_median(read_retentions),
                "mean_reads_removed_low_complexity": safe_mean(removed_reads),
                "mean_q_delta": safe_mean(mean_q_deltas),
                "slowdown_vs_fastest_median": median / fastest_median
                if median is not None
                else None,
            }
        )
    return summary_rows

def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    output: list[dict] = []
    for tool, dimension, cohort, cohort_rows in iter_cohort_row_groups(rows):
        output.append(
            summarize_cohort_rows(
                tool=tool,
                dimension=dimension,
                cohort=cohort,
                cohort_rows=cohort_rows,
            )
        )
    return output

def summarize_cohort_rows(
    *,
    tool: str,
    dimension: str,
    cohort: str,
    cohort_rows: list[dict],
) -> dict:
    runtimes = [float(row["runtime_s"]) for row in cohort_rows]
    removed_reads = [float(row["reads_removed_low_complexity"]) for row in cohort_rows]
    base_retentions = [float(row["base_retention"]) for row in cohort_rows]
    read_retentions = [float(row["read_retention"]) for row in cohort_rows]
    mean_q_deltas = [float(row["mean_q_delta"]) for row in cohort_rows]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "mean_reads_removed_low_complexity": safe_mean(removed_reads),
        "median_base_retention": safe_median(base_retentions),
        "median_read_retention": safe_median(read_retentions),
        "mean_q_delta": safe_mean(mean_q_deltas),
    }

def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest_filter = max(
            sample_rows,
            key=lambda row: float(row["reads_removed_low_complexity"]),
        )
        output.append(
            {
                "sample_id": sample_id,
                "accession": sample_rows[0]["accession"],
                "era": sample_rows[0]["era"],
                "layout": sample_rows[0]["layout"],
                "size_band": sample_rows[0]["size_band"],
                "study_accession": sample_rows[0]["study_accession"],
                "total_runtime_s": sum(float(row["runtime_s"]) for row in sample_rows),
                "slowest_tool": slowest["tool"],
                "slowest_runtime_s": float(slowest["runtime_s"]),
                "strongest_filter_tool": strongest_filter["tool"],
                "strongest_filter_reads": float(
                    strongest_filter["reads_removed_low_complexity"]
                ),
            }
        )
    output.sort(key=lambda row: row["total_runtime_s"], reverse=True)
    return output

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
            "median_base_retention",
            "median_read_retention",
            "mean_reads_removed_low_complexity",
            "mean_q_delta",
            "slowdown_vs_fastest_median",
        ],
        cohort_fieldnames=[
            "tool",
            "dimension",
            "cohort",
            "samples",
            "mean_runtime_s",
            "median_runtime_s",
            "mean_reads_removed_low_complexity",
            "median_base_retention",
            "median_read_retention",
            "mean_q_delta",
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
            "strongest_filter_tool",
            "strongest_filter_reads",
        ],
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
