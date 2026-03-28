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
    TRIM_READS_BENCHMARK_CONTRACT,
    discover_normalized_samples,
    load_corpus_spec,
    load_json,
    localize_results_path,
    load_published_sample_metadata,
    preferred_report_run_root,
    trim_reads_benchmark_defaults,
    validate_corpus_contract,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.trim_reads benchmark summary.",
        docs_root="docs/benchmark/fastq.trim_reads/corpus-01",
    )


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def metric_or_run_default(record: dict, key: str, run_manifest: dict):
    value = normalize_metric(record, key)
    if value is not None:
        return value
    return run_manifest.get(key)


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {
        "fastp": "fastp_json",
        "cutadapt": "cutadapt_json",
        "bbduk": "bbduk_stats",
    }.get(tool)


def validate_trim_row_contract(*, run_manifest: dict, sample_rows: list[dict]) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "trim benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )
        for key in [
            "min_length",
            "quality_cutoff",
            "n_policy",
            "adapter_policy",
            "polyx_policy",
            "contaminant_policy",
            "adapter_bank_preset",
            "polyx_preset",
            "contaminant_preset",
        ]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "trim benchmark report drift: "
                    f"expected {key}={run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "trim benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )


def validate_trim_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit("trim benchmark report rendering requires an executed run, not --dry-run output")
    stage_id = run_manifest.get("stage_id", TRIM_READS_BENCHMARK_CONTRACT.stage_id)
    scenario_id = run_manifest.get("scenario_id", TRIM_READS_BENCHMARK_CONTRACT.scenario_id)
    tool_kind = run_manifest.get("tool_kind", "benchmark")
    if stage_id != TRIM_READS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "trim benchmark report drift: "
            f"expected stage_id {TRIM_READS_BENCHMARK_CONTRACT.stage_id}, found {stage_id}"
        )
    if scenario_id != TRIM_READS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "trim benchmark report drift: "
            f"expected scenario_id {TRIM_READS_BENCHMARK_CONTRACT.scenario_id}, found {scenario_id}"
        )
    if tool_kind != "benchmark":
        raise SystemExit(
            f"trim benchmark report drift: expected tool_kind benchmark, found {tool_kind}"
        )


def render_markdown(summary: dict) -> str:
    def format_trim_setting(value):
        return "governed tool default" if value is None else value

    lines: list[str] = []
    lines.append("# `fastq.trim_reads` on `corpus-01`")
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
    lines.append(f"- min_length: `{format_trim_setting(summary['min_length'])}`")
    lines.append(
        f"- quality_cutoff: `{format_trim_setting(summary['quality_cutoff'])}`"
    )
    lines.append(f"- n_policy: `{summary['n_policy']}`")
    lines.append(f"- adapter_policy: `{summary['adapter_policy']}`")
    lines.append(f"- polyx_policy: `{summary['polyx_policy']}`")
    lines.append(f"- contaminant_policy: `{summary['contaminant_policy']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("best_base_retention_tool"):
        lines.append(
            f"- Highest median base retention: `{headline['best_base_retention_tool']}` at `{headline['best_base_retention']:.3f}`."
        )
    if headline.get("best_read_retention_tool"):
        lines.append(
            f"- Highest median read retention: `{headline['best_read_retention_tool']}` at `{headline['best_read_retention']:.3f}`."
        )
    if headline.get("best_q_gain_tool"):
        lines.append(
            f"- Highest mean Q delta: `{headline['best_q_gain_tool']}` at `{headline['best_q_gain']:.3f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean Q delta |"
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
        read_retention = (
            "n/a"
            if row["median_read_retention"] is None
            else f"{row['median_read_retention']:.3f}"
        )
        mean_q_delta = (
            "n/a" if row["mean_q_delta"] is None else f"{row['mean_q_delta']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {base_retention} | {read_retention} | {mean_q_delta} |"
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
        "- This benchmark intentionally pins bank-free trim policies so the full governed trim fairness cohort can execute under one comparable contract."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def load_sample_metadata(repo_root: Path, corpus_root: Path, spec: dict) -> dict[str, dict]:
    if (corpus_root / "normalized").is_dir():
        return validate_corpus_contract(
            corpus_root,
            spec,
            discover_normalized_samples(corpus_root),
        )
    return load_published_sample_metadata(repo_root, spec)


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser()
        if args.run_root
        else preferred_report_run_root(corpus_root, TRIM_READS_BENCHMARK_CONTRACT.stage_id)
    )
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)
    local_results_root = run_root.parents[2]

    spec = load_corpus_spec(repo_root)
    defaults = trim_reads_benchmark_defaults()
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_trim_run_manifest_contract(run_manifest)
    run_manifest.setdefault("quality_cutoff", defaults["quality_cutoff"])
    run_manifest.setdefault("adapter_bank_preset", defaults["adapter_bank_preset"])
    run_manifest.setdefault("polyx_preset", defaults["polyx_preset"])
    run_manifest.setdefault("contaminant_preset", defaults["contaminant_preset"])
    metadata_by_sample = load_sample_metadata(repo_root, corpus_root, spec)

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
            raise SystemExit(f"missing report.json for {sample_id}: {report_path}")
        report = load_json(report_path)
        records = report.get("records", [])
        if not records:
            raise SystemExit(f"report.json for {sample_id} contains no records")
        for record in records:
            tool = record.get("context", {}).get("tool", "unknown")
            delta_metrics = normalize_metric(record, "delta_metrics")
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
                "base_retention": delta_metrics.get("base_retention", 0.0)
                if isinstance(delta_metrics, dict)
                else 0.0,
                "read_retention": delta_metrics.get("read_retention", 0.0)
                if isinstance(delta_metrics, dict)
                else 0.0,
                "mean_q_delta": delta_metrics.get("mean_q_delta", 0.0)
                if isinstance(delta_metrics, dict)
                else 0.0,
                "min_length": metric_or_run_default(record, "min_length", run_manifest),
                "quality_cutoff": metric_or_run_default(
                    record, "quality_cutoff", run_manifest
                ),
                "n_policy": metric_or_run_default(record, "n_policy", run_manifest),
                "adapter_policy": metric_or_run_default(
                    record, "adapter_policy", run_manifest
                ),
                "polyx_policy": metric_or_run_default(
                    record, "polyx_policy", run_manifest
                ),
                "contaminant_policy": metric_or_run_default(
                    record, "contaminant_policy", run_manifest
                ),
                "adapter_bank_preset": (
                    normalize_metric(record, "adapter_preset")
                    or run_manifest.get("adapter_bank_preset")
                ),
                "polyx_preset": (
                    normalize_metric(record, "polyx_preset")
                    or run_manifest.get("polyx_preset")
                ),
                "contaminant_preset": (
                    normalize_metric(record, "contaminant_preset")
                    or run_manifest.get("contaminant_preset")
                ),
                "raw_backend_report_format": (
                    normalize_metric(record, "raw_backend_report_format")
                    or expected_raw_backend_report_format(tool)
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_trim_row_contract(run_manifest=run_manifest, sample_rows=sample_rows)

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        runtimes = [float(row["runtime_s"]) for row in rows]
        base_retentions = [float(row["base_retention"]) for row in rows]
        read_retentions = [float(row["read_retention"]) for row in rows]
        mean_q_deltas = [float(row["mean_q_delta"]) for row in rows]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median(runtimes),
                "median_base_retention": safe_median(base_retentions),
                "median_read_retention": safe_median(read_retentions),
                "mean_q_delta": safe_mean(mean_q_deltas),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        best_base_retention = max(
            (row for row in tool_summary if row["median_base_retention"] is not None),
            key=lambda row: row["median_base_retention"],
        )
        best_read_retention = max(
            (row for row in tool_summary if row["median_read_retention"] is not None),
            key=lambda row: row["median_read_retention"],
        )
        best_q_gain = max(
            (row for row in tool_summary if row["mean_q_delta"] is not None),
            key=lambda row: row["mean_q_delta"],
        )
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_base_retention_tool": best_base_retention["tool"],
            "best_base_retention": best_base_retention["median_base_retention"],
            "best_read_retention_tool": best_read_retention["tool"],
            "best_read_retention": best_read_retention["median_read_retention"],
            "best_q_gain_tool": best_q_gain["tool"],
            "best_q_gain": best_q_gain["mean_q_delta"],
        }

    summary = {
        "schema_version": "bijux.fastq.trim_reads.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": TRIM_READS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": run_manifest["scenario_id"],
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "min_length": run_manifest["min_length"],
        "quality_cutoff": run_manifest["quality_cutoff"],
        "n_policy": run_manifest["n_policy"],
        "adapter_policy": run_manifest["adapter_policy"],
        "polyx_policy": run_manifest["polyx_policy"],
        "contaminant_policy": run_manifest["contaminant_policy"],
        "adapter_bank_preset": run_manifest["adapter_bank_preset"],
        "polyx_preset": run_manifest["polyx_preset"],
        "contaminant_preset": run_manifest["contaminant_preset"],
        "headline": headline,
        "tool_summary": tool_summary,
    }

    summary_path = docs_root / "summary.json"
    summary_path.write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    sample_results_path = docs_root / "sample_results.csv"
    with sample_results_path.open("w", encoding="utf-8", newline="") as handle:
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
                "base_retention",
                "read_retention",
                "mean_q_delta",
                "min_length",
                "quality_cutoff",
                "n_policy",
                "adapter_policy",
                "polyx_policy",
                "contaminant_policy",
                "adapter_bank_preset",
                "polyx_preset",
                "contaminant_preset",
                "raw_backend_report_format",
            ],
        )
        writer.writeheader()
        for row in sample_rows:
            writer.writerow(row)

    (docs_root / "benchmark.md").write_text(
        render_markdown(summary),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
