#!/usr/bin/env python3
from __future__ import annotations

import argparse
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from benchmark_fastq_corpus.support import (
    parse_corpus_report_args,
    CorpusReportArtifacts,
    CorpusReportContext,
    FILTER_READS_BENCHMARK_CONTRACT,
    filter_reads_benchmark_defaults,
    load_json,
    localize_results_path,
    run_corpus_report,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.filter_reads benchmark summary.",
        docs_root="docs/benchmark/fastq.filter_reads/corpus-01",
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


def normalize_parameter(record: dict, key: str):
    parameters = record.get("context", {}).get("parameters", {})
    return parameters.get(key)


def delta_metric(record: dict, key: str) -> float:
    delta_metrics = normalize_metric(record, "delta_metrics")
    if not isinstance(delta_metrics, dict):
        return 0.0
    return float(delta_metrics.get(key, 0.0))


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {
        "bbduk": "bbduk_stats",
        "fastp": "fastp_json",
        "prinseq": None,
        "seqkit": None,
    }.get(tool)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = filter_reads_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "filter-reads benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "filter-reads benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != FILTER_READS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "filter-reads benchmark report drift: "
            f"expected stage_id {FILTER_READS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != FILTER_READS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "filter-reads benchmark report drift: "
            f"expected scenario_id {FILTER_READS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "filter-reads benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    for key, value in defaults.items():
        run_manifest.setdefault(key, value)


def validate_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        for key in [
            "max_n",
            "max_n_fraction",
            "max_n_count",
            "low_complexity_threshold",
            "entropy_threshold",
            "kmer_ref",
            "polyx_policy",
        ]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "filter-reads benchmark report drift: "
                    f"expected {key} {run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "filter-reads benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format!r}, found {row['raw_backend_report_format']!r}"
            )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "filter-reads benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["bases_out"]) > int(row["bases_in"]):
            raise SystemExit(
                "filter-reads benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has bases_out > bases_in"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "filter-reads benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "filter-reads benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.filter_reads` on `corpus-01`")
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
    lines.append(f"- max_n: `{summary['max_n']}`")
    lines.append(f"- max_n_fraction: `{summary['max_n_fraction']}`")
    lines.append(f"- max_n_count: `{summary['max_n_count']}`")
    lines.append(
        f"- low_complexity_threshold: `{summary['low_complexity_threshold']}`"
    )
    lines.append(f"- entropy_threshold: `{summary['entropy_threshold']}`")
    lines.append(f"- kmer_ref: `{summary['kmer_ref']}`")
    lines.append(f"- polyx_policy: `{summary['polyx_policy']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest median base retention: `{summary['headline']['best_base_retention_tool']}` at `{summary['headline']['best_base_retention']:.3f}`."
    )
    lines.append(
        f"- Highest mean reads dropped: `{summary['headline']['strongest_filter_tool']}` at `{summary['headline']['strongest_filter_reads']:.1f}` reads."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Median read retention | Mean reads dropped | Mean low-complexity removals | Mean N removals |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_base_retention']:.3f} | {row['median_read_retention']:.3f} | {row['mean_reads_dropped']:.1f} | {row['mean_reads_removed_low_complexity']:.1f} | {row['mean_reads_removed_by_n']:.1f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This benchmark keeps one governed filter contract across the full human DNA cohort so any retention drift stays attributable to backend behavior rather than threshold changes."
    )
    lines.append(
        "- Raw backend report formats remain explicit in `sample_results.csv` so future audits can distinguish native evidence from the governed summary layer."
    )
    return "\n".join(lines) + "\n"


def build_artifacts(context: CorpusReportContext) -> CorpusReportArtifacts:
    corpus_root = context.runtime.corpus_root
    run_root = context.runtime.run_root
    local_results_root = context.runtime.local_results_root

    run_manifest = context.run_manifest
    expected_sample_ids = context.expected_sample_ids
    metadata_by_sample = context.metadata_by_sample

    sample_rows: list[dict] = []
    tool_rows: dict[str, list[dict]] = defaultdict(list)
    cohort_counts: dict[str, int] = defaultdict(int)
    era_counts: dict[str, int] = defaultdict(int)
    layout_counts: dict[str, int] = defaultdict(int)

    for run in context.applicable_runs:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample[sample_id]
        cohort_counts[f"{metadata['era']}_{metadata['layout']}"] += 1
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
            reads_in = int(normalize_metric(record, "reads_in") or 0)
            reads_out = int(normalize_metric(record, "reads_out") or 0)
            bases_in = int(normalize_metric(record, "bases_in") or 0)
            bases_out = int(normalize_metric(record, "bases_out") or 0)
            row = {
                "sample_id": sample_id,
                "accession": metadata["accession"],
                "era": metadata["era"],
                "layout": metadata["layout"],
                "study_accession": metadata["study_accession"],
                "size_band": metadata["size_band"],
                "tool": tool,
                "runtime_s": float(record.get("execution", {}).get("runtime_s") or 0.0),
                "exit_code": int(record.get("execution", {}).get("exit_code") or 0),
                "reads_in": reads_in,
                "reads_out": reads_out,
                "reads_dropped": int(normalize_metric(record, "reads_dropped") or 0),
                "reads_removed_by_n": int(
                    normalize_metric(record, "reads_removed_by_n") or 0
                ),
                "reads_removed_by_entropy": int(
                    normalize_metric(record, "reads_removed_by_entropy") or 0
                ),
                "reads_removed_low_complexity": int(
                    normalize_metric(record, "reads_removed_low_complexity") or 0
                ),
                "reads_removed_by_kmer": int(
                    normalize_metric(record, "reads_removed_by_kmer") or 0
                ),
                "reads_removed_contaminant_kmer": int(
                    normalize_metric(record, "reads_removed_contaminant_kmer") or 0
                ),
                "reads_removed_by_length": int(
                    normalize_metric(record, "reads_removed_by_length") or 0
                ),
                "bases_in": bases_in,
                "bases_out": bases_out,
                "read_retention": (reads_out / reads_in) if reads_in else 0.0,
                "base_retention": (bases_out / bases_in) if bases_in else 0.0,
                "mean_q_before": float(normalize_metric(record, "mean_q_before") or 0.0),
                "mean_q_after": float(normalize_metric(record, "mean_q_after") or 0.0),
                "mean_q_delta": delta_metric(record, "mean_q"),
                "max_n": normalize_parameter(record, "max_n"),
                "max_n_fraction": normalize_parameter(record, "max_n_fraction"),
                "max_n_count": normalize_parameter(record, "max_n_count"),
                "low_complexity_threshold": normalize_parameter(
                    record, "low_complexity_threshold"
                ),
                "entropy_threshold": normalize_parameter(record, "entropy_threshold"),
                "kmer_ref": normalize_parameter(record, "kmer_ref"),
                "polyx_policy": normalize_parameter(record, "polyx_policy"),
                "raw_backend_report_format": normalize_parameter(
                    record, "raw_backend_report_format"
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary: list[dict] = []
    for tool, rows in sorted(tool_rows.items()):
        runtimes = [float(row["runtime_s"]) for row in rows]
        read_retentions = [float(row["read_retention"]) for row in rows]
        base_retentions = [float(row["base_retention"]) for row in rows]
        reads_dropped = [float(row["reads_dropped"]) for row in rows]
        removed_low_complexity = [
            float(row["reads_removed_low_complexity"]) for row in rows
        ]
        removed_by_n = [float(row["reads_removed_by_n"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_read_retention": safe_median(read_retentions),
                "median_base_retention": safe_median(base_retentions),
                "mean_reads_dropped": safe_mean(reads_dropped),
                "mean_reads_removed_low_complexity": safe_mean(
                    removed_low_complexity
                ),
                "mean_reads_removed_by_n": safe_mean(removed_by_n),
            }
        )

    headline_fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    headline_retention = max(
        tool_summary, key=lambda row: row["median_base_retention"]
    )
    headline_filter = max(tool_summary, key=lambda row: row["mean_reads_dropped"])

    summary = {
        "schema_version": "bijux.fastq.filter_reads.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": FILTER_READS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": FILTER_READS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "samples_total": len(expected_sample_ids),
        "samples_failed": run_manifest["samples_failed"],
        "tools": run_manifest["tools"],
        "cohort_counts": dict(cohort_counts),
        "era_counts": dict(era_counts),
        "layout_counts": dict(layout_counts),
        "max_n": run_manifest["max_n"],
        "max_n_fraction": run_manifest["max_n_fraction"],
        "max_n_count": run_manifest["max_n_count"],
        "low_complexity_threshold": run_manifest["low_complexity_threshold"],
        "entropy_threshold": run_manifest["entropy_threshold"],
        "kmer_ref": run_manifest["kmer_ref"],
        "polyx_policy": run_manifest["polyx_policy"],
        "headline": {
            "fastest_tool": headline_fastest["tool"],
            "fastest_runtime_s": headline_fastest["median_runtime_s"],
            "best_base_retention_tool": headline_retention["tool"],
            "best_base_retention": headline_retention["median_base_retention"],
            "strongest_filter_tool": headline_filter["tool"],
            "strongest_filter_reads": headline_filter["mean_reads_dropped"],
        },
        "tool_summary": tool_summary,
    }

    return CorpusReportArtifacts(
        summary=summary,
        markdown=render_markdown(summary),
        sample_rows=sample_rows,
        sample_fieldnames=list(sample_rows[0].keys()),
    )


def main() -> int:
    return run_corpus_report(
        parse_args(),
        contract=FILTER_READS_BENCHMARK_CONTRACT,
        validate_run_manifest=validate_run_manifest_contract,
        build_artifacts=build_artifacts,
    )


if __name__ == "__main__":
    raise SystemExit(main())
