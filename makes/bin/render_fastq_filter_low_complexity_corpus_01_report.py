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
    FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT,
    filter_low_complexity_benchmark_defaults,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.filter_low_complexity benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="/home/bijan/bijux/corpus_01")
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.filter_low_complexity/corpus-01",
    )
    return parser.parse_args()


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


def delta_metric(record: dict, key: str) -> float:
    delta_metrics = normalize_metric(record, "delta_metrics")
    if not isinstance(delta_metrics, dict):
        return 0.0
    return float(delta_metrics.get(key, 0.0))


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {
        "bbduk": "bbduk_stats",
        "prinseq": None,
    }.get(tool)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = filter_low_complexity_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "filter-low-complexity benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "filter-low-complexity benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "filter-low-complexity benchmark report drift: "
            f"expected stage_id {FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "filter-low-complexity benchmark report drift: "
            f"expected scenario_id {FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "filter-low-complexity benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    run_manifest.setdefault("entropy_threshold", defaults["entropy_threshold"])
    run_manifest.setdefault("polyx_threshold", defaults["polyx_threshold"])


def validate_row_contract(*, run_manifest: dict, sample_rows: list[dict]) -> None:
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["entropy_threshold"] != run_manifest["entropy_threshold"]:
            raise SystemExit(
                "filter-low-complexity benchmark report drift: "
                f"expected entropy_threshold {run_manifest['entropy_threshold']}, "
                f"found {row['entropy_threshold']} for {row['sample_id']}/{row['tool']}"
            )
        if row["polyx_threshold"] != run_manifest["polyx_threshold"]:
            raise SystemExit(
                "filter-low-complexity benchmark report drift: "
                f"expected polyx_threshold {run_manifest['polyx_threshold']}, "
                f"found {row['polyx_threshold']} for {row['sample_id']}/{row['tool']}"
            )
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "filter-low-complexity benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "filter-low-complexity benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )


def render_markdown(summary: dict) -> str:
    polyx_threshold = (
        "unset"
        if summary["polyx_threshold"] is None
        else str(summary["polyx_threshold"])
    )
    lines: list[str] = []
    lines.append("# `fastq.filter_low_complexity` on `corpus-01`")
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Generated: {summary['generated_at_utc']}")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Run root: `{summary['run_root']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Samples benchmarked: `{summary['samples_total']}`")
    lines.append(
        f"- Layout balance: `{summary['layout_counts'].get('se', 0)}` single-end, `{summary['layout_counts'].get('pe', 0)}` paired-end"
    )
    lines.append(
        f"- Era balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern"
    )
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- entropy_threshold: `{summary['entropy_threshold']}`")
    lines.append(f"- polyx_threshold: `{polyx_threshold}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary["headline"]
    lines.append(
        f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest median base retention: `{headline['best_base_retention_tool']}` at `{headline['best_base_retention']:.3f}`."
    )
    lines.append(
        f"- Highest mean low-complexity removals: `{headline['strongest_filter_tool']}` at `{headline['strongest_filter_reads']:.1f}` reads."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean removed reads | Mean Q delta |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_base_retention']:.3f} | {row['median_read_retention']:.3f} | {row['mean_reads_removed_low_complexity']:.1f} | {row['mean_q_delta']:.3f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This corpus benchmark keeps one fixed low-complexity contract across the full roster so any removal differences remain attributable to backend behavior rather than threshold drift."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser().resolve()
        if args.run_root
        else preferred_report_run_root(
            corpus_root,
            FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.stage_id,
        )
    )
    local_results_root = run_root.parents[2]
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)

    spec = load_corpus_spec(repo_root)
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_run_manifest_contract(run_manifest)
    expected_sample_ids = [run["sample_id"] for run in run_manifest["runs"]]
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=expected_sample_ids,
        fallback_stage_id=FILTER_LOW_COMPLEXITY_BENCHMARK_CONTRACT.stage_id,
    )

    sample_rows: list[dict] = []
    tool_rows: dict[str, list[dict]] = defaultdict(list)
    cohort_counts: dict[str, int] = defaultdict(int)
    era_counts: dict[str, int] = defaultdict(int)
    layout_counts: dict[str, int] = defaultdict(int)

    for run in run_manifest["runs"]:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample[sample_id]
        cohort_key = f"{metadata['era']}_{metadata['layout']}"
        cohort_counts[cohort_key] += 1
        era_counts[metadata["era"]] += 1
        layout_counts[metadata["layout"]] += 1

        report_path = localize_results_path(run["report_json"], local_results_root)
        if not report_path.is_file():
            raise SystemExit(f"missing report.json for {sample_id}: {report_path}")
        report = load_json(report_path)
        records = report.get("records", [])
        if not records:
            raise SystemExit(f"report.json for {sample_id} contains no records")
        for record in records:
            tool = record.get("context", {}).get("tool", "unknown")
            row = {
                "sample_id": sample_id,
                "accession": metadata["accession"],
                "era": metadata["era"],
                "layout": metadata["layout"],
                "study_accession": metadata["study_accession"],
                "size_band": metadata["size_band"],
                "tool": tool,
                "runtime_s": record.get("execution", {}).get("runtime_s"),
                "exit_code": record.get("execution", {}).get("exit_code"),
                "reads_in": normalize_metric(record, "reads_in") or 0,
                "reads_out": normalize_metric(record, "reads_out") or 0,
                "bases_in": normalize_metric(record, "bases_in") or 0,
                "bases_out": normalize_metric(record, "bases_out") or 0,
                "base_retention": delta_metric(record, "base_retention"),
                "read_retention": delta_metric(record, "read_retention"),
                "mean_q_delta": delta_metric(record, "mean_q_delta"),
                "entropy_threshold": normalize_metric(record, "entropy_threshold"),
                "polyx_threshold": normalize_metric(record, "polyx_threshold"),
                "reads_removed_low_complexity": normalize_metric(
                    record, "reads_removed_low_complexity"
                )
                or 0,
                "paired_mode": normalize_metric(record, "paired_mode"),
                "raw_backend_report_format": normalize_metric(
                    record, "raw_backend_report_format"
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_row_contract(run_manifest=run_manifest, sample_rows=sample_rows)

    tool_summary: list[dict] = []
    for tool, rows in sorted(tool_rows.items()):
        runtimes = [float(row["runtime_s"]) for row in rows]
        base_retentions = [float(row["base_retention"]) for row in rows]
        read_retentions = [float(row["read_retention"]) for row in rows]
        removed_reads = [float(row["reads_removed_low_complexity"]) for row in rows]
        mean_q_deltas = [float(row["mean_q_delta"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_base_retention": safe_median(base_retentions),
                "median_read_retention": safe_median(read_retentions),
                "mean_reads_removed_low_complexity": safe_mean(removed_reads),
                "mean_q_delta": safe_mean(mean_q_deltas),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    best_base_retention = max(tool_summary, key=lambda row: row["median_base_retention"])
    strongest_filter = max(
        tool_summary,
        key=lambda row: row["mean_reads_removed_low_complexity"],
    )
    summary = {
        "schema_version": "bijux.fastq.filter_low_complexity.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "platform": run_manifest["platform"],
        "stage_id": run_manifest["stage_id"],
        "scenario_id": run_manifest["scenario_id"],
        "corpus_root": run_manifest["corpus_root"],
        "run_root": str(run_root),
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "tools": run_manifest["tools"],
        "entropy_threshold": run_manifest["entropy_threshold"],
        "polyx_threshold": run_manifest["polyx_threshold"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "tool_summary": tool_summary,
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_base_retention_tool": best_base_retention["tool"],
            "best_base_retention": best_base_retention["median_base_retention"],
            "strongest_filter_tool": strongest_filter["tool"],
            "strongest_filter_reads": strongest_filter["mean_reads_removed_low_complexity"],
        },
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (docs_root / "lunarc.md").write_text(render_markdown(summary), encoding="utf-8")
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
                "entropy_threshold",
                "polyx_threshold",
                "reads_removed_low_complexity",
                "paired_mode",
                "raw_backend_report_format",
            ],
        )
        writer.writeheader()
        writer.writerows(sample_rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
