#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    parse_corpus_report_args,
    TRIM_POLYG_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_report_runtime,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.trim_polyg_tails benchmark summary.",
        docs_root="docs/benchmark/fastq.trim_polyg_tails/corpus-01",
    )


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def safe_fraction(numerator: int, denominator: int) -> float | None:
    if denominator == 0:
        return None
    return numerator / denominator


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def validate_trim_polyg_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
) -> None:
    expected_tools = run_manifest["tools"]
    expected_formats = {
        "fastp": "fastp_json",
        "bbduk": "bbduk_stats",
    }
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        expected_format = expected_formats.get(row["tool"])
        if expected_format and row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "trim-polyg benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )
        if row["polyx_preset"] != run_manifest["polyx_preset"]:
            raise SystemExit(
                "trim-polyg benchmark report drift: "
                f"expected polyx_preset {run_manifest['polyx_preset']}, "
                f"found {row['polyx_preset']} for {row['sample_id']}/{row['tool']}"
            )
        if row["min_polyg_run"] != run_manifest["min_polyg_run"]:
            raise SystemExit(
                "trim-polyg benchmark report drift: "
                f"expected min_polyg_run {run_manifest['min_polyg_run']}, "
                f"found {row['min_polyg_run']} for {row['sample_id']}/{row['tool']}"
            )
        if run_manifest["trim_polyg"] is not None and row["trim_polyg"] != run_manifest["trim_polyg"]:
            raise SystemExit(
                "trim-polyg benchmark report drift: "
                f"expected trim_polyg {run_manifest['trim_polyg']}, "
                f"found {row['trim_polyg']} for {row['sample_id']}/{row['tool']}"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "trim-polyg benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )


def validate_trim_polyg_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit("trim-polyg benchmark report rendering requires an executed run, not --dry-run output")
    stage_id = run_manifest.get("stage_id", TRIM_POLYG_BENCHMARK_CONTRACT.stage_id)
    scenario_id = run_manifest.get(
        "scenario_id", TRIM_POLYG_BENCHMARK_CONTRACT.scenario_id
    )
    tool_kind = run_manifest.get("tool_kind", "benchmark")
    if stage_id != TRIM_POLYG_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "trim-polyg benchmark report drift: "
            f"expected stage_id {TRIM_POLYG_BENCHMARK_CONTRACT.stage_id}, found {stage_id}"
        )
    if scenario_id != TRIM_POLYG_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "trim-polyg benchmark report drift: "
            f"expected scenario_id {TRIM_POLYG_BENCHMARK_CONTRACT.scenario_id}, found {scenario_id}"
        )
    if tool_kind != "benchmark":
        raise SystemExit(
            f"trim-polyg benchmark report drift: expected tool_kind benchmark, found {tool_kind}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.trim_polyg_tails` on `corpus-01`")
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Generated: {summary['generated_at_utc']}")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Run root: `{summary['run_root']}`")
    lines.append(f"- Samples benchmarked: `{summary['samples_total']}`")
    lines.append(
        f"- Layout balance: `{summary['layout_counts'].get('se', 0)}` single-end, `{summary['layout_counts'].get('pe', 0)}` paired-end"
    )
    lines.append(
        f"- Era balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern"
    )
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- PolyX preset: `{summary['polyx_preset']}`")
    lines.append(f"- min_polyg_run: `{summary['min_polyg_run']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("largest_polyg_trim_tool"):
        lines.append(
            f"- Highest mean polyG trimming: `{headline['largest_polyg_trim_tool']}` with `{headline['largest_polyg_trim_bases']:.1f}` bases removed on average."
        )
    if headline.get("best_base_retention_tool"):
        lines.append(
            f"- Highest median base retention: `{headline['best_base_retention_tool']}` at `{headline['best_base_retention']:.3f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Mean bases trimmed | Mean Q delta |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        base_retention = (
            "n/a"
            if row["median_base_retention"] is None
            else f"{row['median_base_retention']:.3f}"
        )
        mean_bases_trimmed = (
            "n/a"
            if row["mean_bases_trimmed_polyg"] is None
            else f"{row['mean_bases_trimmed_polyg']:.1f}"
        )
        mean_q_delta = (
            "n/a" if row["mean_q_delta"] is None else f"{row['mean_q_delta']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {base_retention} | {mean_bases_trimmed} | {mean_q_delta} |"
        )
    lines.append("")
    lines.append("## Cohort Coverage")
    lines.append("")
    lines.append("| Cohort | Samples |")
    lines.append("| --- | ---: |")
    for label, count in sorted(summary["cohort_counts"].items()):
        lines.append(f"| `{label}` | {count} |")
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- Ancient and modern samples are resolved by matching normalized FASTQ checksums back to raw accession directories and then joining those accessions to `configs/runtime/corpora/corpus-01.toml`."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    runtime = resolve_corpus_report_runtime(
        args,
        stage_id=TRIM_POLYG_BENCHMARK_CONTRACT.stage_id,
    )
    repo_root = runtime.repo_root
    corpus_root = runtime.corpus_root
    run_root = runtime.run_root
    docs_root = runtime.docs_root
    local_results_root = runtime.local_results_root

    spec = load_corpus_spec(repo_root)
    run_manifest = runtime.run_manifest
    validate_trim_polyg_run_manifest_contract(run_manifest)
    expected_sample_ids = [run["sample_id"] for run in run_manifest["runs"]]
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=expected_sample_ids,
        fallback_stage_id=TRIM_POLYG_BENCHMARK_CONTRACT.stage_id,
    )

    sample_rows: list[dict] = []
    tool_rows: dict[str, list[dict]] = defaultdict(list)
    cohort_counts: dict[str, int] = defaultdict(int)
    era_counts: dict[str, int] = defaultdict(int)
    layout_counts: dict[str, int] = defaultdict(int)

    for run in run_manifest["runs"]:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample.get(sample_id, {})
        cohort_key = f"{metadata.get('era', 'unknown')}_{metadata.get('layout', run['layout'])}"
        cohort_counts[cohort_key] += 1
        era_counts[metadata.get("era", "unknown")] += 1
        layout_counts[metadata.get("layout", run["layout"])] += 1

        report_path = localize_results_path(run["report_json"], local_results_root)
        if not report_path.is_file():
            continue
        report = load_json(report_path)
        for record in report.get("records", []):
            tool = record.get("context", {}).get("tool", "unknown")
            row = {
                "sample_id": sample_id,
                "accession": metadata.get("accession"),
                "era": metadata.get("era"),
                "layout": metadata.get("layout", run["layout"]),
                "study_accession": metadata.get("study_accession"),
                "size_band": metadata.get("size_band"),
                "tool": tool,
                "runtime_s": record.get("execution", {}).get("runtime_s"),
                "exit_code": record.get("execution", {}).get("exit_code"),
                "reads_in": normalize_metric(record, "reads_in") or 0,
                "reads_out": normalize_metric(record, "reads_out") or 0,
                "bases_in": normalize_metric(record, "bases_in") or 0,
                "bases_out": normalize_metric(record, "bases_out") or 0,
                "base_retention": normalize_metric(record, "delta_metrics").get(
                    "base_retention", 0.0
                )
                if isinstance(normalize_metric(record, "delta_metrics"), dict)
                else 0.0,
                "read_retention": normalize_metric(record, "delta_metrics").get(
                    "read_retention", 0.0
                )
                if isinstance(normalize_metric(record, "delta_metrics"), dict)
                else 0.0,
                "mean_q_delta": normalize_metric(record, "delta_metrics").get(
                    "mean_q_delta", 0.0
                )
                if isinstance(normalize_metric(record, "delta_metrics"), dict)
                else 0.0,
                "bases_trimmed_polyg": normalize_metric(record, "bases_trimmed_polyg") or 0,
                "polyx_preset": normalize_metric(record, "polyx_preset"),
                "trim_polyg": normalize_metric(record, "trim_polyg"),
                "min_polyg_run": normalize_metric(record, "min_polyg_run"),
                "paired_mode": normalize_metric(record, "paired_mode"),
                "raw_backend_report_format": normalize_metric(
                    record, "raw_backend_report_format"
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_trim_polyg_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
    )

    tool_summary: list[dict] = []
    for tool, rows in sorted(tool_rows.items()):
        runtimes = [float(row["runtime_s"]) for row in rows if row["runtime_s"] is not None]
        base_retentions = [float(row["base_retention"]) for row in rows]
        bases_trimmed = [float(row["bases_trimmed_polyg"]) for row in rows]
        mean_q_deltas = [float(row["mean_q_delta"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_base_retention": safe_median(base_retentions),
                "mean_bases_trimmed_polyg": safe_mean(bases_trimmed),
                "mean_q_delta": safe_mean(mean_q_deltas),
            }
        )

    fastest = min(
        (row for row in tool_summary if row["median_runtime_s"] is not None),
        key=lambda row: row["median_runtime_s"],
        default=None,
    )
    largest_polyg_trim = max(
        (row for row in tool_summary if row["mean_bases_trimmed_polyg"] is not None),
        key=lambda row: row["mean_bases_trimmed_polyg"],
        default=None,
    )
    best_base_retention = max(
        (row for row in tool_summary if row["median_base_retention"] is not None),
        key=lambda row: row["median_base_retention"],
        default=None,
    )

    summary = {
        "schema_version": "bijux.fastq.trim_polyg_tails.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "tools": run_manifest["tools"],
        "polyx_preset": run_manifest["polyx_preset"],
        "trim_polyg": run_manifest["trim_polyg"],
        "min_polyg_run": run_manifest["min_polyg_run"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "tool_summary": tool_summary,
        "headline": {
            "fastest_tool": fastest["tool"] if fastest else None,
            "fastest_runtime_s": fastest["median_runtime_s"] if fastest else None,
            "largest_polyg_trim_tool": largest_polyg_trim["tool"]
            if largest_polyg_trim
            else None,
            "largest_polyg_trim_bases": largest_polyg_trim["mean_bases_trimmed_polyg"]
            if largest_polyg_trim
            else None,
            "best_base_retention_tool": best_base_retention["tool"]
            if best_base_retention
            else None,
            "best_base_retention": best_base_retention["median_base_retention"]
            if best_base_retention
            else None,
        },
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (docs_root / "benchmark.md").write_text(
        render_markdown(summary),
        encoding="utf-8",
    )

    with (docs_root / "sample_results.csv").open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "sample_id",
                "accession",
                "era",
                "layout",
                "study_accession",
                "size_band",
                "tool",
                "runtime_s",
                "exit_code",
                "reads_in",
                "reads_out",
                "bases_in",
                "bases_out",
                "read_retention",
                "base_retention",
                "mean_q_delta",
                "bases_trimmed_polyg",
                "polyx_preset",
                "trim_polyg",
                "min_polyg_run",
                "paired_mode",
                "raw_backend_report_format",
            ],
        )
        writer.writeheader()
        writer.writerows(sample_rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
