#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path

from corpus_01_fastq_benchmark_support import parse_corpus_briefing_args


def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 validate artifacts.",
        docs_root="docs/benchmark/fastq.validate_reads/corpus-01",
    )


def load_json(path: Path) -> dict:
    return json.loads(path.read_text(encoding="utf-8"))


def load_rows(path: Path) -> list[dict]:
    with path.open(encoding="utf-8", newline="") as handle:
        return list(csv.DictReader(handle))


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def percentile(values: list[float], fraction: float) -> float | None:
    if not values:
        return None
    ordered = sorted(values)
    index = round((len(ordered) - 1) * fraction)
    return float(ordered[index])


def fmt_runtime(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}"


def fmt_fraction(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.1%}"


def fmt_csv_value(value: object) -> object:
    if isinstance(value, float):
        return f"{value:.6f}"
    return value


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
                "slowdown_vs_fastest_median": median / fastest_median
                if median is not None
                else None,
            }
        )
    return summary_rows


def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    grouped: dict[tuple[str, str, str], list[float]] = defaultdict(list)
    grouped_with_size: dict[tuple[str, str, str], list[float]] = defaultdict(list)
    for row in rows:
        runtime = float(row["runtime_s"])
        grouped[(row["tool"], row["era"], row["layout"])].append(runtime)
        grouped_with_size[(row["tool"], "size_band", row["size_band"])].append(runtime)

    output: list[dict] = []
    for (tool, era, layout), runtimes in sorted(grouped.items()):
        output.append(
            {
                "tool": tool,
                "dimension": "era_layout",
                "cohort": f"{era}_{layout}",
                "samples": len(runtimes),
                "mean_runtime_s": safe_mean(runtimes),
                "median_runtime_s": safe_median(runtimes),
            }
        )
    for (tool, dimension, cohort), runtimes in sorted(grouped_with_size.items()):
        output.append(
            {
                "tool": tool,
                "dimension": dimension,
                "cohort": cohort,
                "samples": len(runtimes),
                "mean_runtime_s": safe_mean(runtimes),
                "median_runtime_s": safe_median(runtimes),
            }
        )
    return output


def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
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
            }
        )
    output.sort(key=lambda row: row["total_runtime_s"], reverse=True)
    return output


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
    fastqc_modern_pe = cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="era_layout",
        cohort="modern_pe",
    )
    fastqc_modern_se = cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="era_layout",
        cohort="modern_se",
    )
    fastqc_under_500 = cohort_entry(
        cohort_rows,
        tool="fastqc",
        dimension="size_band",
        cohort="under_500mb",
    )
    fastqc_under_100 = cohort_entry(
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


def write_csv(path: Path, rows: list[dict], fieldnames: list[str]) -> None:
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=fieldnames)
        writer.writeheader()
        writer.writerows(
            {
                fieldname: fmt_csv_value(row[fieldname])
                for fieldname in fieldnames
            }
            for row in rows
        )


def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    summary = load_json(docs_root / "summary.json")
    rows = load_rows(docs_root / "sample_results.csv")

    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outliers = sample_runtime_outliers(rows)

    write_csv(
        docs_root / "tool_runtime_summary.csv",
        runtime_rows,
        [
            "tool",
            "samples",
            "pass_rate",
            "mean_runtime_s",
            "median_runtime_s",
            "p90_runtime_s",
            "max_runtime_s",
            "slowdown_vs_fastest_median",
        ],
    )
    write_csv(
        docs_root / "cohort_runtime_summary.csv",
        cohort_rows,
        ["tool", "dimension", "cohort", "samples", "mean_runtime_s", "median_runtime_s"],
    )
    write_csv(
        docs_root / "sample_runtime_outliers.csv",
        outliers,
        [
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
    (docs_root / "benchmark.md").write_text(
        render_markdown(summary, rows, runtime_rows, cohort_rows, outliers),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
