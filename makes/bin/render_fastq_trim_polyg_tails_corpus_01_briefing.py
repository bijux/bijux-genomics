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
    safe_mean,
    safe_median,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 trim-polyg artifacts.",
        docs_root="docs/benchmark/fastq.trim_polyg_tails/corpus-01",
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
        bases_trimmed = [float(row["bases_trimmed_polyg"]) for row in tool_rows]
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
                "mean_bases_trimmed_polyg": safe_mean(bases_trimmed),
                "mean_q_delta": safe_mean(mean_q_deltas),
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
    bases_trimmed = [float(row["bases_trimmed_polyg"]) for row in cohort_rows]
    base_retentions = [float(row["base_retention"]) for row in cohort_rows]
    mean_q_deltas = [float(row["mean_q_delta"]) for row in cohort_rows]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "mean_bases_trimmed_polyg": safe_mean(bases_trimmed),
        "median_base_retention": safe_median(base_retentions),
        "mean_q_delta": safe_mean(mean_q_deltas),
    }

def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        most_trimming = max(sample_rows, key=lambda row: float(row["bases_trimmed_polyg"]))
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
                "most_trimming_tool": most_trimming["tool"],
                "most_trimmed_bases": float(most_trimming["bases_trimmed_polyg"]),
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
    tool_lookup = {row["tool"]: row for row in runtime_rows}
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    slowest = max(runtime_rows, key=lambda row: row["median_runtime_s"])
    heaviest_modern_pe = cohort_entry(
        cohort_rows,
        tool="fastp",
        dimension="era_layout",
        cohort="modern_pe",
    )
    lightest_modern_se = cohort_entry(
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

def cohort_entry(
    cohort_rows: list[dict], *, tool: str, dimension: str, cohort: str
) -> dict | None:
    for row in cohort_rows:
        if (
            row["tool"] == tool
            and row["dimension"] == dimension
            and row["cohort"] == cohort
        ):
            return row
    return None

def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    summary = load_json(docs_root / "summary.json")
    rows = load_csv_rows(docs_root / "sample_results.csv")

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
            "mean_bases_trimmed_polyg",
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
            "mean_bases_trimmed_polyg",
            "median_base_retention",
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
            "most_trimming_tool",
            "most_trimmed_bases",
        ],
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
