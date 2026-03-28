#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render an enriched benchmark briefing from corpus-01 deplete-reference-contaminants artifacts."
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.deplete_reference_contaminants/corpus-01",
    )
    return parser.parse_args()


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


def fmt_metric(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}"


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
        read_retentions = [float(row["read_retention"]) for row in tool_rows]
        base_retentions = [float(row["base_retention"]) for row in tool_rows]
        contamination_removed = [
            float(row["contaminant_fraction_removed"]) for row in tool_rows
        ]
        reads_removed = [float(row["reads_removed"]) for row in tool_rows]
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
                "median_read_retention": safe_median(read_retentions),
                "median_base_retention": safe_median(base_retentions),
                "mean_contaminant_fraction_removed": safe_mean(contamination_removed),
                "mean_reads_removed": safe_mean(reads_removed),
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
    read_retentions = [float(row["read_retention"]) for row in cohort_rows]
    base_retentions = [float(row["base_retention"]) for row in cohort_rows]
    contamination_removed = [
        float(row["contaminant_fraction_removed"]) for row in cohort_rows
    ]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "median_read_retention": safe_median(read_retentions),
        "median_base_retention": safe_median(base_retentions),
        "mean_contaminant_fraction_removed": safe_mean(contamination_removed),
    }


def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest = max(
            sample_rows,
            key=lambda row: float(row["contaminant_fraction_removed"]),
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
                "strongest_depletion_tool": strongest["tool"],
                "strongest_contaminant_fraction_removed": float(
                    strongest["contaminant_fraction_removed"]
                ),
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
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    summary = load_json(docs_root / "summary.json")
    rows = load_rows(docs_root / "sample_results.csv")

    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outliers = sample_runtime_outliers(rows)

    with (docs_root / "tool_runtime_summary.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(runtime_rows[0].keys()))
        writer.writeheader()
        writer.writerows(
            {key: fmt_csv_value(value) for key, value in row.items()}
            for row in runtime_rows
        )

    with (docs_root / "cohort_runtime_summary.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(cohort_rows[0].keys()))
        writer.writeheader()
        writer.writerows(
            {key: fmt_csv_value(value) for key, value in row.items()}
            for row in cohort_rows
        )

    with (docs_root / "sample_runtime_outliers.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(outliers[0].keys()))
        writer.writeheader()
        writer.writerows(
            {key: fmt_csv_value(value) for key, value in row.items()}
            for row in outliers
        )

    (docs_root / "benchmark.md").write_text(
        render_markdown(summary, rows, runtime_rows, cohort_rows, outliers),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
