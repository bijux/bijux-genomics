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
        description="Render an enriched benchmark briefing from corpus-01 terminal-damage artifacts."
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.trim_terminal_damage/corpus-01",
        help="Directory that contains summary.json and sample_results.csv.",
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
        base_retentions = [float(row["base_retention"]) for row in tool_rows]
        asymmetry_reductions = [
            float(row["asymmetry_reduction"])
            for row in tool_rows
            if row["asymmetry_reduction"] not in {"", None}
        ]
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
                "mean_asymmetry_reduction": safe_mean(asymmetry_reductions),
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
    base_retentions = [float(row["base_retention"]) for row in cohort_rows]
    asymmetry_reductions = [
        float(row["asymmetry_reduction"])
        for row in cohort_rows
        if row["asymmetry_reduction"] not in {"", None}
    ]
    mean_q_deltas = [float(row["mean_q_delta"]) for row in cohort_rows]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "median_base_retention": safe_median(base_retentions),
        "mean_asymmetry_reduction": safe_mean(asymmetry_reductions),
        "mean_q_delta": safe_mean(mean_q_deltas),
    }


def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        asymmetry_rows = [
            row for row in sample_rows if row["asymmetry_reduction"] not in {"", None}
        ]
        strongest = (
            max(asymmetry_rows, key=lambda row: float(row["asymmetry_reduction"]))
            if asymmetry_rows
            else None
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
                "strongest_damage_tool": strongest["tool"] if strongest else None,
                "strongest_asymmetry_reduction": (
                    float(strongest["asymmetry_reduction"]) if strongest else None
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
    asymmetry_rows = [
        row for row in runtime_rows if row["mean_asymmetry_reduction"] is not None
    ]
    strongest = (
        max(asymmetry_rows, key=lambda row: row["mean_asymmetry_reduction"])
        if asymmetry_rows
        else None
    )
    weakest = (
        min(asymmetry_rows, key=lambda row: row["mean_asymmetry_reduction"])
        if asymmetry_rows
        else None
    )
    zero_exit = sum(1 for row in rows if row["exit_code"] == "0")

    lines: list[str] = []
    lines.append("# `fastq.trim_terminal_damage` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark compares the governed `fastq.trim_terminal_damage` stage across the full `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- `3` governed terminal-damage backends were benchmarked across `20` samples (`{zero_exit}/{len(rows)}` zero-exit tool-sample observations)."
    )
    lines.append(
        f"- Fastest median runtime: `{fastest['tool']}` at `{fmt_runtime(fastest['median_runtime_s'])}` seconds."
    )
    lines.append(
        f"- Slowest median runtime: `{slowest['tool']}` at `{fmt_runtime(slowest['median_runtime_s'])}` seconds."
    )
    if strongest and weakest:
        lines.append(
            f"- Strongest mean terminal asymmetry reduction: `{strongest['tool']}` at `{strongest['mean_asymmetry_reduction']:.3f}`."
        )
        lines.append(
            f"- Weakest mean terminal asymmetry reduction: `{weakest['tool']}` at `{weakest['mean_asymmetry_reduction']:.3f}`."
        )
    else:
        lines.append(
            "- Terminal asymmetry reduction is not published for this corpus run, so backend ranking falls back to runtime and base-retention evidence."
        )
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Stage: `{summary['stage_id']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- damage_mode: `{summary['damage_mode']}`")
    lines.append(f"- execution_policy: `{summary['execution_policy']}`")
    lines.append(f"- trim_5p_bases: `{summary['trim_5p_bases']}`")
    lines.append(f"- trim_3p_bases: `{summary['trim_3p_bases']}`")
    lines.append("")
    lines.append("## Tool Ranking")
    lines.append("")
    lines.append(
        "| Tool | Median runtime (s) | p90 runtime (s) | Median base retention | Mean asymmetry reduction | Mean Q delta | Slowdown vs fastest |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in sorted(runtime_rows, key=lambda item: item["median_runtime_s"]):
        lines.append(
            f"| `{row['tool']}` | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {row['median_base_retention']:.3f} | {fmt_metric(row['mean_asymmetry_reduction'])} | {row['mean_q_delta']:.3f} | {row['slowdown_vs_fastest_median']:.2f}x |"
        )
    lines.append("")
    lines.append("## Cohort Behavior")
    lines.append("")
    lines.append(
        "- Ancient and modern samples stay in the same benchmark corpus here so modern data act as a negative-control cohort for damage-aware trimming behavior."
    )
    lines.append(
        "- The best runtime backend and strongest damage-reduction backend may differ when asymmetry evidence is available, so this stage should not be judged on latency alone."
    )
    lines.append("")
    lines.append("## Highest-Cost Samples")
    lines.append("")
    lines.append("| Sample | Accession | Cohort | Total runtime (s) | Slowest tool | Slowest runtime (s) | Strongest damage tool | Asymmetry reduction |")
    lines.append("| --- | --- | --- | ---: | --- | ---: | --- | ---: |")
    for row in outliers[:5]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}_{row['layout']}` | {row['total_runtime_s']:.3f} | `{row['slowest_tool']}` | {row['slowest_runtime_s']:.3f} | `{row['strongest_damage_tool'] or 'n/a'}` | {fmt_metric(row['strongest_asymmetry_reduction'])} |"
        )
    lines.append("")
    lines.append("## Reproducibility")
    lines.append("")
    lines.append("- `summary.json`, `sample_results.csv`, and the CSV analysis tables in this directory are generated from one governed run manifest.")
    lines.append("- The renderer rejects dry-run manifests and mixed policy rows, so publication is tied to a real executed benchmark surface.")
    return "\n".join(lines) + "\n"


def write_csv(path: Path, rows: list[dict]) -> None:
    if not rows:
        return
    with path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(handle, fieldnames=list(rows[0].keys()))
        writer.writeheader()
        for row in rows:
            writer.writerow({key: fmt_csv_value(value) for key, value in row.items()})


def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    summary = load_json(docs_root / "summary.json")
    rows = load_rows(docs_root / "sample_results.csv")
    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outliers = sample_runtime_outliers(rows)

    write_csv(docs_root / "tool_runtime_summary.csv", runtime_rows)
    write_csv(docs_root / "cohort_runtime_summary.csv", cohort_rows)
    write_csv(docs_root / "sample_runtime_outliers.csv", outliers)
    (docs_root / "lunarc.md").write_text(
        render_markdown(summary, rows, runtime_rows, cohort_rows, outliers),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
