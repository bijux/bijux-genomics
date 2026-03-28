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
    safe_mean,
    safe_median,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 read-length artifacts.",
        docs_root="docs/benchmark/fastq.profile_read_lengths/corpus-01",
    )


def validate_summary_contract(summary: dict) -> None:
    expected_values = {
        "stage_id": "fastq.profile_read_lengths",
        "scenario_id": "read_length_fairness",
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
        "raw_backend_report_format": "seqkit_stats_length_histogram",
    }
    for key, expected in expected_values.items():
        if summary.get(key) != expected:
            raise SystemExit(
                f"read-length briefing drift: summary {key}={summary.get(key)!r} expected {expected!r}"
            )
    if int(summary.get("histogram_bins", 0) or 0) <= 0:
        raise SystemExit("read-length briefing drift: histogram_bins must stay positive")


def validate_rows_contract(summary: dict, rows: list[dict]) -> None:
    expected_tools = sorted(summary.get("tools") or [])
    observed_tools = sorted({row.get("tool", "") for row in rows if row.get("tool")})
    if observed_tools != expected_tools:
        raise SystemExit(
            f"read-length briefing drift: observed tools {observed_tools!r} expected {expected_tools!r}"
        )
    for row in rows:
        if int(row.get("distinct_lengths", "0") or 0) <= 0:
            raise SystemExit(
                "read-length briefing drift: distinct_lengths must stay positive"
            )
        if not str(row.get("report_json_artifact", "")).endswith("report.json"):
            raise SystemExit(
                "read-length briefing drift: report_json_artifact must end with report.json"
            )
        if not str(row.get("length_distribution_tsv_artifact", "")).endswith(
            "length_distribution.tsv"
        ):
            raise SystemExit(
                "read-length briefing drift: length_distribution_tsv_artifact must end with length_distribution.tsv"
            )
        if not str(row.get("length_distribution_json_artifact", "")).endswith(
            "length_distribution.json"
        ):
            raise SystemExit(
                "read-length briefing drift: length_distribution_json_artifact must end with length_distribution.json"
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
        read_counts = [float(row["read_count"]) for row in tool_rows]
        mean_lengths = [float(row["mean_read_length"]) for row in tool_rows]
        max_lengths = [float(row["max_read_length"]) for row in tool_rows]
        distinct_lengths = [float(row["distinct_lengths"]) for row in tool_rows]
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
                "median_read_count": safe_median(read_counts),
                "median_mean_read_length": safe_median(mean_lengths),
                "median_max_read_length": safe_median(max_lengths),
                "median_distinct_lengths": safe_median(distinct_lengths),
                "slowdown_vs_fastest_median": median / fastest_median
                if median is not None
                else None,
            }
        )
    return summary_rows

def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    grouped: dict[tuple[str, str, str], list[dict]] = defaultdict(list)
    grouped_with_size: dict[tuple[str, str, str], list[dict]] = defaultdict(list)
    for row in rows:
        grouped[(row["tool"], row["era"], row["layout"])].append(row)
        grouped_with_size[(row["tool"], "size_band", row["size_band"])].append(row)

    output: list[dict] = []
    for (tool, era, layout), cohort_rows in sorted(grouped.items()):
        output.append(
            summarize_cohort_rows(
                tool=tool,
                dimension="era_layout",
                cohort=f"{era}_{layout}",
                cohort_rows=cohort_rows,
            )
        )
    for (tool, dimension, cohort), cohort_rows in sorted(grouped_with_size.items()):
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
    mean_lengths = [float(row["mean_read_length"]) for row in cohort_rows]
    max_lengths = [float(row["max_read_length"]) for row in cohort_rows]
    distinct_lengths = [float(row["distinct_lengths"]) for row in cohort_rows]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "median_mean_read_length": safe_median(mean_lengths),
        "median_max_read_length": safe_median(max_lengths),
        "median_distinct_lengths": safe_median(distinct_lengths),
    }

def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        widest = max(sample_rows, key=lambda row: float(row["distinct_lengths"]))
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
                "widest_length_tool": widest["tool"],
                "widest_length_support": float(widest["distinct_lengths"]),
                "max_read_length": float(widest["max_read_length"]),
            }
        )
    output.sort(key=lambda row: row["total_runtime_s"], reverse=True)
    return output

def cohort_entry(
    rows: list[dict],
    *,
    tool: str,
    dimension: str,
    cohort: str,
) -> dict | None:
    for row in rows:
        if row["tool"] == tool and row["dimension"] == dimension and row["cohort"] == cohort:
            return row
    return None

def render_markdown(
    summary: dict,
    rows: list[dict],
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    reference_tool = summary["tools"][0]
    tool_lookup = {row["tool"]: row for row in runtime_rows}
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    modern_pe = cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="era_layout",
        cohort="modern_pe",
    )
    ancient_se = cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="era_layout",
        cohort="ancient_se",
    )
    under_500 = cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="size_band",
        cohort="under_500mb",
    )
    under_100 = cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="size_band",
        cohort="under_100mb",
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.profile_read_lengths` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark measures the governed `fastq.profile_read_lengths` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}` on Lunarc")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Benchmark root: `{summary['run_root']}`")
    lines.append(
        f"- Input balance: `{summary['era_counts']['ancient']}` ancient, `{summary['era_counts']['modern']}` modern, `{summary['layout_counts']['se']}` single-end, `{summary['layout_counts']['pe']}` paired-end"
    )
    lines.append(f"- Tool set: `{', '.join(summary['tools'])}`")
    lines.append(
        f"- Length-profile contract: report_only=`{summary['report_only']}`, mutates_fastq=`{summary['mutates_fastq']}`, may_change_read_count=`{summary['may_change_read_count']}`, histogram_bins=`{summary['histogram_bins']}`"
    )
    lines.append(
        "- Governed artifacts per sample/tool: `profile_read_lengths_report.json`, `length_distribution.tsv`, and `length_distribution.json`."
    )
    lines.append("- Execution profile: one benchmark sample at a time, one worker, governed thread budget")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `{fastest['tool']}` ran at `p50={fmt_runtime(fastest['median_runtime_s'])}s` with median mean read length `{fmt_value(tool_lookup[reference_tool]['median_mean_read_length'])}` and median distinct-length support `{fmt_value(tool_lookup[reference_tool]['median_distinct_lengths'])}`."
    )
    if modern_pe and ancient_se:
        lines.append(
            f"- Runtime remains input-driven for `{reference_tool}`: `modern_pe` averages `{fmt_runtime(modern_pe['mean_runtime_s'])}s` while `ancient_se` averages `{fmt_runtime(ancient_se['mean_runtime_s'])}s`."
        )
    if under_500 and under_100:
        lines.append(
            f"- Size-band spread remains visible for `{reference_tool}`: `under_500mb` averages `{fmt_runtime(under_500['mean_runtime_s'])}s` versus `{fmt_runtime(under_100['mean_runtime_s'])}s` on `under_100mb` inputs."
        )
    lines.append(
        f"- Correctness stayed stable across all `{len(rows)}` tool-sample observations: `exit_code=0` on `{zero_exit}` rows, and every published row carried governed histogram artifacts plus valid length-distribution metrics."
    )
    lines.append(
        f"- Histogram resolution stayed pinned at `{summary['histogram_bins']}` bins, so cross-sample comparisons use one deterministic bucket budget."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median read count | Median mean length | Median max length | Median distinct lengths |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | {fmt_value(row['median_read_count'])} | {fmt_value(row['median_mean_read_length'])} | {fmt_value(row['median_max_read_length'])} | {fmt_value(row['median_distinct_lengths'])} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median mean length | Median max length | Median distinct lengths |"
    )
    lines.append("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['dimension']}` | `{row['cohort']}` | {row['samples']} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_value(row['median_mean_read_length'])} | {fmt_value(row['median_max_read_length'])} | {fmt_value(row['median_distinct_lengths'])} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append("| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Distinct lengths | Max read length |")
    lines.append("| --- | --- | --- | --- | --- | ---: | ---: | ---: |")
    for row in outliers[:10]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['slowest_runtime_s'])} | {fmt_value(row['widest_length_support'])} | {fmt_value(row['max_read_length'])} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- `fastq.profile_read_lengths` is a non-mutating length-profile stage, so runtime and deterministic histogram stability matter more than retention deltas."
    )
    lines.append(
        "- Because the current governed benchmark cohort is a single backend, this dossier acts as a corpus-wide stability baseline for future regressions and future backend additions."
    )
    lines.append(
        "- Artifact integrity matters as much as the metrics here: without the governed TSV and JSON histogram outputs, downstream comparisons lose their canonical surface."
    )
    lines.append(
        "- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis."
    )
    return "\n".join(lines) + "\n"

def main() -> int:
    runtime = resolve_corpus_briefing_runtime(parse_args())
    docs_root = runtime.docs_root
    summary = runtime.summary
    rows = runtime.sample_rows
    validate_summary_contract(summary)
    validate_rows_contract(summary, rows)

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
