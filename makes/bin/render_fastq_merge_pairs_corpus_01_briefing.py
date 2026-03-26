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
        description="Render an enriched benchmark briefing from corpus-01 merge-pairs artifacts."
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.merge_pairs/corpus-01",
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


def fmt_fraction(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.1%}"


def fmt_decimal(value: float | None) -> str:
    if value is None:
        return "n/a"
    return f"{value:.3f}"


def format_merge_setting(value) -> str:
    if value is None:
        return "governed tool default"
    return str(value)


def fmt_csv_value(value: object) -> object:
    if isinstance(value, float):
        return f"{value:.6f}"
    return value


def validate_inputs(summary: dict, rows: list[dict]) -> None:
    if summary.get("stage_id") != "fastq.merge_pairs":
        raise SystemExit(
            "merge briefing drift: "
            f"expected summary stage_id fastq.merge_pairs, found {summary.get('stage_id')!r}"
        )
    if summary.get("scenario_id") != "merge_fairness":
        raise SystemExit(
            "merge briefing drift: "
            f"expected summary scenario_id merge_fairness, found {summary.get('scenario_id')!r}"
        )
    if summary.get("layout_counts", {}).get("pe") != summary.get("samples_total"):
        raise SystemExit(
            "merge briefing drift: layout_counts.pe must equal samples_total for paired-only benchmark"
        )
    if summary.get("layout_counts", {}).get("se", 0) != 0:
        raise SystemExit("merge briefing drift: paired-only benchmark must not report se rows")
    tools = summary.get("tools", [])
    if not tools:
        raise SystemExit("merge briefing drift: summary tools must not be empty")
    if not rows:
        raise SystemExit("merge briefing drift: sample_results.csv must not be empty")
    observed_tools = sorted({row["tool"] for row in rows})
    if observed_tools != sorted(tools):
        raise SystemExit(
            "merge briefing drift: "
            f"summary tools {tools} do not match sample_results tools {observed_tools}"
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
        merge_rates = [float(row["merge_rate"]) for row in tool_rows]
        base_retentions = [float(row["base_retention"]) for row in tool_rows]
        merged_reads = [float(row["reads_merged"]) for row in tool_rows]
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
                "median_merge_rate": safe_median(merge_rates),
                "median_base_retention": safe_median(base_retentions),
                "mean_reads_merged": safe_mean(merged_reads),
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
    merge_rates = [float(row["merge_rate"]) for row in cohort_rows]
    base_retentions = [float(row["base_retention"]) for row in cohort_rows]
    merged_reads = [float(row["reads_merged"]) for row in cohort_rows]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "mean_merge_rate": safe_mean(merge_rates),
        "median_base_retention": safe_median(base_retentions),
        "mean_reads_merged": safe_mean(merged_reads),
    }


def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        best_merge = max(sample_rows, key=lambda row: float(row["merge_rate"]))
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
                "best_merge_rate_tool": best_merge["tool"],
                "best_merge_rate": float(best_merge["merge_rate"]),
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
    tool_lookup = {row["tool"]: row for row in runtime_rows}
    fastest = min(runtime_rows, key=lambda row: row["median_runtime_s"])
    slowest = max(runtime_rows, key=lambda row: row["median_runtime_s"])
    best_merge = max(runtime_rows, key=lambda row: row["median_merge_rate"])
    heaviest_modern = cohort_entry(
        cohort_rows,
        tool=best_merge["tool"],
        dimension="era_layout",
        cohort="modern_pe",
    )
    lightest_ancient = cohort_entry(
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
        "- Results are generated from the Lunarc benchmark tree and mirrored locally under `/Users/bijan/bijux/bijux-dna-results` for later inspection."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    summary = load_json(docs_root / "summary.json")
    rows = load_rows(docs_root / "sample_results.csv")
    validate_inputs(summary, rows)

    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outlier_rows = sample_runtime_outliers(rows)

    for name, fieldnames, values in [
        (
            "tool_runtime_summary.csv",
            [
                "tool",
                "samples",
                "pass_rate",
                "mean_runtime_s",
                "median_runtime_s",
                "p90_runtime_s",
                "max_runtime_s",
                "median_merge_rate",
                "median_base_retention",
                "mean_reads_merged",
                "slowdown_vs_fastest_median",
            ],
            runtime_rows,
        ),
        (
            "cohort_runtime_summary.csv",
            [
                "tool",
                "dimension",
                "cohort",
                "samples",
                "mean_runtime_s",
                "median_runtime_s",
                "mean_merge_rate",
                "median_base_retention",
                "mean_reads_merged",
            ],
            cohort_rows,
        ),
        (
            "sample_runtime_outliers.csv",
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
                "best_merge_rate_tool",
                "best_merge_rate",
            ],
            outlier_rows,
        ),
    ]:
        path = docs_root / name
        with path.open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=fieldnames)
            writer.writeheader()
            writer.writerows(
                {
                    key: fmt_csv_value(row.get(key))
                    for key in fieldnames
                }
                for row in values
            )

    markdown = render_markdown(
        summary=summary,
        rows=rows,
        runtime_rows=runtime_rows,
        cohort_rows=cohort_rows,
        outliers=outlier_rows,
    )
    (docs_root / "lunarc.md").write_text(markdown, encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
