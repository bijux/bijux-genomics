#!/usr/bin/env python3
from __future__ import annotations

import argparse
import statistics
from collections import defaultdict
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    parse_corpus_report_args,
    DEPLETE_RRNA_BENCHMARK_CONTRACT,
    deplete_rrna_benchmark_defaults,
    load_corpus_spec,
    load_json,
    localize_results_path,
    publish_corpus_report_artifacts,
    preferred_report_run_root,
    resolve_corpus_report_runtime,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.deplete_rrna benchmark summary.",
        docs_root="docs/benchmark/fastq.deplete_rrna/corpus-01",
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


def depletion_summary(record: dict) -> dict:
    payload = normalize_metric(record, "depletion_summary")
    return payload if isinstance(payload, dict) else {}


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = deplete_rrna_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "deplete-rrna benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "deplete-rrna benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != DEPLETE_RRNA_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "deplete-rrna benchmark report drift: "
            f"expected stage_id {DEPLETE_RRNA_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != DEPLETE_RRNA_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "deplete-rrna benchmark report drift: "
            f"expected scenario_id {DEPLETE_RRNA_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "deplete-rrna benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    if not run_manifest.get("rrna_db"):
        raise SystemExit(
            "deplete-rrna benchmark report drift: run manifest must record rrna_db"
        )
    if not run_manifest.get("rrna_bundle_digest"):
        raise SystemExit(
            "deplete-rrna benchmark report drift: run manifest must record rrna_bundle_digest"
        )
    run_manifest.setdefault("rrna_bundle_id", defaults["rrna_bundle_id"])
    run_manifest.setdefault("min_identity", defaults["min_identity"])


def validate_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["database_artifact_id"] != run_manifest["rrna_bundle_id"]:
            raise SystemExit(
                "deplete-rrna benchmark report drift: "
                f"expected database_artifact_id {run_manifest['rrna_bundle_id']!r}, "
                f"found {row['database_artifact_id']!r} for {row['sample_id']}/{row['tool']}"
            )
        if float(row["min_identity"]) != float(run_manifest["min_identity"]):
            raise SystemExit(
                "deplete-rrna benchmark report drift: "
                f"expected min_identity {run_manifest['min_identity']}, "
                f"found {row['min_identity']} for {row['sample_id']}/{row['tool']}"
            )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "deplete-rrna benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["bases_out"]) > int(row["bases_in"]):
            raise SystemExit(
                "deplete-rrna benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has bases_out > bases_in"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "deplete-rrna benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "deplete-rrna benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.deplete_rrna` on `corpus-01`")
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
    lines.append(f"- rrna_bundle_id: `{summary['rrna_bundle_id']}`")
    lines.append(f"- rrna_bundle_digest: `{summary['rrna_bundle_digest']}`")
    lines.append(f"- rrna_db: `{summary['rrna_db']}`")
    lines.append(f"- min_identity: `{summary['min_identity']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest mean rRNA fraction removed: `{summary['headline']['strongest_depletion_tool']}` at `{summary['headline']['strongest_depletion_fraction']:.3f}`."
    )
    lines.append(
        f"- Highest median read retention: `{summary['headline']['best_read_retention_tool']}` at `{summary['headline']['best_read_retention']:.3f}`."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Median base retention | Mean rRNA fraction removed | Mean reads removed |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_read_retention']:.3f} | {row['median_base_retention']:.3f} | {row['mean_rrna_fraction_removed']:.3f} | {row['mean_reads_removed']:.1f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- `corpus-01` is a human DNA cohort, so rRNA depletion behaves primarily as a governed false-positive control and throughput benchmark rather than a high-yield cleanup stage."
    )
    lines.append(
        "- The run manifest pins the concrete SortMeRNA reference path and bundle digest so future reruns can detect reference drift immediately."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    runtime = resolve_corpus_report_runtime(
        args,
        stage_id=DEPLETE_RRNA_BENCHMARK_CONTRACT.stage_id,
    )
    repo_root = runtime.repo_root
    corpus_root = runtime.corpus_root
    run_root = runtime.run_root
    docs_root = runtime.docs_root
    local_results_root = runtime.local_results_root

    spec = load_corpus_spec(repo_root)
    run_manifest = runtime.run_manifest
    validate_run_manifest_contract(run_manifest)
    expected_sample_ids = [run["sample_id"] for run in run_manifest["runs"]]
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=expected_sample_ids,
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
            reads_in = int(normalize_metric(record, "reads_in") or 0)
            reads_out = int(normalize_metric(record, "reads_out") or 0)
            bases_in = int(normalize_metric(record, "bases_in") or 0)
            bases_out = int(normalize_metric(record, "bases_out") or 0)
            summary_payload = depletion_summary(record)
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
                "bases_in": bases_in,
                "bases_out": bases_out,
                "reads_removed": int(summary_payload.get("reads_removed", 0) or 0),
                "bases_removed": int(summary_payload.get("bases_removed", 0) or 0),
                "read_retention": (reads_out / reads_in) if reads_in else 0.0,
                "base_retention": (bases_out / bases_in) if bases_in else 0.0,
                "rrna_fraction_removed": float(
                    normalize_metric(record, "rrna_fraction_removed") or 0.0
                ),
                "pairs_in": int(normalize_metric(record, "pairs_in") or 0),
                "pairs_out": int(normalize_metric(record, "pairs_out") or 0),
                "database_artifact_id": summary_payload.get("database_artifact_id"),
                "screening_engine": summary_payload.get("screening_engine"),
                "report_tsv": summary_payload.get("report_tsv"),
                "report_json": summary_payload.get("report_json"),
                "min_identity": run_manifest["min_identity"],
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
        rrna_removed = [float(row["rrna_fraction_removed"]) for row in rows]
        reads_removed = [float(row["reads_removed"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_read_retention": safe_median(read_retentions),
                "median_base_retention": safe_median(base_retentions),
                "mean_rrna_fraction_removed": safe_mean(rrna_removed),
                "mean_reads_removed": safe_mean(reads_removed),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    strongest_depletion = max(
        tool_summary,
        key=lambda row: row["mean_rrna_fraction_removed"],
    )
    best_retention = max(tool_summary, key=lambda row: row["median_read_retention"])
    summary = {
        "schema_version": "bijux.fastq.deplete_rrna.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": DEPLETE_RRNA_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": DEPLETE_RRNA_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": int(run_manifest.get("samples_failed", 0)),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "rrna_db": run_manifest["rrna_db"],
        "rrna_bundle_id": run_manifest["rrna_bundle_id"],
        "rrna_bundle_digest": run_manifest["rrna_bundle_digest"],
        "rrna_bundle_size_bytes": run_manifest["rrna_bundle_size_bytes"],
        "min_identity": run_manifest["min_identity"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "strongest_depletion_tool": strongest_depletion["tool"],
            "strongest_depletion_fraction": strongest_depletion[
                "mean_rrna_fraction_removed"
            ],
            "best_read_retention_tool": best_retention["tool"],
            "best_read_retention": best_retention["median_read_retention"],
        },
        "tool_summary": tool_summary,
    }

    publish_corpus_report_artifacts(
        docs_root,
        summary=summary,
        markdown=render_markdown(summary),
        sample_rows=sample_rows,
        sample_fieldnames=list(sample_rows[0].keys()),
        summary_sort_keys=True,
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
