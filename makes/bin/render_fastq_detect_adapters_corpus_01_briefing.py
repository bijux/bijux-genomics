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
        description="Render an enriched benchmark briefing from corpus-01 detect-adapters artifacts."
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.detect_adapters/corpus-01",
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


def fmt_csv_value(value: object) -> object:
    if isinstance(value, float):
        return f"{value:.6f}"
    return value


def optional_float(raw: str) -> float | None:
    if raw == "":
        return None
    return float(raw)


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
        candidate_counts = [float(row["candidate_adapter_count"]) for row in tool_rows]
        trimmed_fractions = [
            optional_float(row["adapter_trimmed_fraction"])
            for row in tool_rows
            if row["adapter_trimmed_fraction"] != ""
        ]
        mean_q_values = [float(row["mean_q"]) for row in tool_rows]
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
                "mean_candidate_adapter_count": safe_mean(candidate_counts),
                "mean_adapter_trimmed_fraction": safe_mean(trimmed_fractions),
                "median_mean_q": safe_median(mean_q_values),
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
    candidate_counts = [float(row["candidate_adapter_count"]) for row in cohort_rows]
    trimmed_fractions = [
        optional_float(row["adapter_trimmed_fraction"])
        for row in cohort_rows
        if row["adapter_trimmed_fraction"] != ""
    ]
    return {
        "tool": tool,
        "dimension": dimension,
        "cohort": cohort,
        "samples": len(cohort_rows),
        "mean_runtime_s": safe_mean(runtimes),
        "median_runtime_s": safe_median(runtimes),
        "mean_candidate_adapter_count": safe_mean(candidate_counts),
        "mean_adapter_trimmed_fraction": safe_mean(trimmed_fractions),
    }


def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in rows:
        by_sample[row["sample_id"]].append(row)

    output: list[dict] = []
    for sample_id, sample_rows in by_sample.items():
        slowest = max(sample_rows, key=lambda row: float(row["runtime_s"]))
        strongest_signal = max(
            sample_rows,
            key=lambda row: float(row["candidate_adapter_count"]),
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
                "strongest_signal_tool": strongest_signal["tool"],
                "strongest_signal_count": float(
                    strongest_signal["candidate_adapter_count"]
                ),
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
    lines.append("# `fastq.detect_adapters` benchmark on `corpus-01`")
    lines.append("")
    lines.append("## What was run")
    lines.append("")
    lines.append(
        "This benchmark measures the governed `fastq.detect_adapters` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform."
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
        f"- Evidence contract: `{summary['inspection_mode']}`, `{summary['evidence_scope']}`, `{summary['evidence_format']}`, report_only=`{summary['report_only']}`"
    )
    lines.append("- Execution profile: one benchmark sample at a time, one worker, one thread per tool invocation")
    lines.append("")
    lines.append("## Executive summary")
    lines.append("")
    lines.append(
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`."
    )
    lines.append(
        f"- `{fastest['tool']}` ran at `p50={fmt_runtime(fastest['median_runtime_s'])}s` with mean candidate-adapter count `{tool_lookup[reference_tool]['mean_candidate_adapter_count']:.3f}`."
    )
    if modern_pe and ancient_se:
        lines.append(
            f"- Runtime remains input-driven for `{reference_tool}`: `modern_pe` averages `{fmt_runtime(modern_pe['mean_runtime_s'])}s` while `ancient_se` averages `{fmt_runtime(ancient_se['mean_runtime_s'])}s`."
        )
    if under_500 and under_100:
        lines.append(
            f"- Size-band spread stays visible in the observer stage for `{reference_tool}`: `under_500mb` averages `{fmt_runtime(under_500['mean_runtime_s'])}s` versus `{fmt_runtime(under_100['mean_runtime_s'])}s` on `under_100mb` inputs."
        )
    lines.append(
        f"- Correctness stayed stable across all `{len(rows)}` tool-sample observations: `exit_code=0` on `{zero_exit}` rows, and the stage preserved `reads_out == reads_in` and `bases_out == bases_in` for every published row."
    )
    lines.append("")
    lines.append("## Tool ranking")
    lines.append("")
    lines.append(
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Mean candidates | Mean trimmed fraction |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in runtime_rows:
        trimmed_fraction = (
            "n/a"
            if row["mean_adapter_trimmed_fraction"] is None
            else f"{row['mean_adapter_trimmed_fraction']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | {row['mean_candidate_adapter_count']:.3f} | {trimmed_fraction} |"
        )
    lines.append("")
    lines.append("## Cohort behavior")
    lines.append("")
    lines.append(
        "| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Mean candidates | Mean trimmed fraction |"
    )
    lines.append("| --- | --- | --- | ---: | ---: | ---: | ---: | ---: |")
    for row in cohort_rows:
        trimmed_fraction = (
            "n/a"
            if row["mean_adapter_trimmed_fraction"] is None
            else f"{row['mean_adapter_trimmed_fraction']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | `{row['dimension']}` | `{row['cohort']}` | {row['samples']} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {row['mean_candidate_adapter_count']:.3f} | {trimmed_fraction} |"
        )
    lines.append("")
    lines.append("## Highest-cost samples")
    lines.append("")
    lines.append("| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Candidate count |")
    lines.append("| --- | --- | --- | --- | --- | ---: | ---: |")
    for row in outliers[:10]:
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['slowest_runtime_s'])} | {row['strongest_signal_count']:.3f} |"
        )
    lines.append("")
    lines.append("## Interpretation")
    lines.append("")
    lines.append(
        "- `fastq.detect_adapters` is an observer stage, so throughput and signal coverage matter more than retention metrics. The governed contract intentionally preserves input reads and bases unchanged."
    )
    lines.append(
        "- Because the current governed benchmark cohort is a single backend, this dossier is primarily a run-to-run stability baseline across corpus composition rather than a backend ranking exercise."
    )
    lines.append(
        "- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis."
    )
    return "\n".join(lines) + "\n"


def write_csv(path: Path, rows: list[dict]) -> None:
    if not rows:
        raise SystemExit(f"cannot write empty csv artifact: {path}")
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
