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
    benchmark_remote_corpus_root,
    DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT,
    deplete_reference_contaminants_benchmark_defaults,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.deplete_reference_contaminants benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.deplete_reference_contaminants/corpus-01",
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


def normalize_parameter(record: dict, key: str):
    return record.get("context", {}).get("parameters", {}).get(key)


def depletion_summary(record: dict) -> dict:
    payload = normalize_metric(record, "depletion_summary")
    return payload if isinstance(payload, dict) else {}


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {"bowtie2": "bowtie2_met_file"}.get(tool)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = deplete_reference_contaminants_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "deplete-reference-contaminants benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "deplete-reference-contaminants benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: "
            f"expected stage_id {DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: "
            f"expected scenario_id {DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    if not run_manifest.get("reference_index"):
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: run manifest must record reference_index"
        )
    if not run_manifest.get("reference_index_digest"):
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: run manifest must record reference_index_digest"
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
        if row["reference_catalog_id"] != run_manifest["reference_catalog_id"]:
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"expected reference_catalog_id {run_manifest['reference_catalog_id']!r}, "
                f"found {row['reference_catalog_id']!r} for {row['sample_id']}/{row['tool']}"
            )
        if row["reference_index_backend"] != run_manifest["reference_index_backend"]:
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"expected reference_index_backend {run_manifest['reference_index_backend']!r}, "
                f"found {row['reference_index_backend']!r} for {row['sample_id']}/{row['tool']}"
            )
        if row["decoy_mode"] != run_manifest["decoy_mode"]:
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"expected decoy_mode {run_manifest['decoy_mode']!r}, "
                f"found {row['decoy_mode']!r} for {row['sample_id']}/{row['tool']}"
            )
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format!r}, found {row['raw_backend_report_format']!r}"
            )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["bases_out"]) > int(row["bases_in"]):
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has bases_out > bases_in"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "deplete-reference-contaminants benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "deplete-reference-contaminants benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.deplete_reference_contaminants` on `corpus-01`")
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
    lines.append(f"- reference_index: `{summary['reference_index']}`")
    lines.append(f"- reference_index_digest: `{summary['reference_index_digest']}`")
    lines.append(f"- reference_catalog_id: `{summary['reference_catalog_id']}`")
    lines.append(f"- reference_index_backend: `{summary['reference_index_backend']}`")
    lines.append(f"- decoy_mode: `{summary['decoy_mode']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest mean contaminant fraction removed: `{summary['headline']['strongest_depletion_tool']}` at `{summary['headline']['strongest_depletion_fraction']:.3f}`."
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
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Median base retention | Mean contaminant fraction removed | Mean reads removed |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_read_retention']:.3f} | {row['median_base_retention']:.3f} | {row['mean_contaminant_fraction_removed']:.3f} | {row['mean_reads_removed']:.1f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- `corpus-01` is a human DNA cohort, so contaminant depletion here functions as a false-positive control and reference-lineage throughput benchmark rather than a high-yield cleanup stage."
    )
    lines.append(
        "- The dossier records index lineage and governed decoy policy directly so later reruns can separate reference drift from real backend differences."
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
            corpus_root, DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id
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
                "contaminant_fraction_removed": float(
                    normalize_metric(record, "contaminant_fraction_removed") or 0.0
                ),
                "pairs_in": int(normalize_metric(record, "pairs_in") or 0),
                "pairs_out": int(normalize_metric(record, "pairs_out") or 0),
                "reference_catalog_id": normalize_parameter(
                    record, "reference_catalog_id"
                )
                or run_manifest["reference_catalog_id"],
                "reference_index_backend": normalize_parameter(
                    record, "reference_index_backend"
                )
                or run_manifest["reference_index_backend"],
                "decoy_mode": normalize_parameter(record, "decoy_mode")
                or run_manifest["decoy_mode"],
                "raw_backend_report": normalize_parameter(record, "raw_backend_report"),
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
        contamination_removed = [
            float(row["contaminant_fraction_removed"]) for row in rows
        ]
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
                "mean_contaminant_fraction_removed": safe_mean(contamination_removed),
                "mean_reads_removed": safe_mean(reads_removed),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    strongest_depletion = max(
        tool_summary,
        key=lambda row: row["mean_contaminant_fraction_removed"],
    )
    best_retention = max(tool_summary, key=lambda row: row["median_read_retention"])
    summary = {
        "schema_version": "bijux.fastq.deplete_reference_contaminants.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": int(run_manifest.get("samples_failed", 0)),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "reference_index": run_manifest["reference_index"],
        "reference_index_digest": run_manifest["reference_index_digest"],
        "reference_index_size_bytes": run_manifest["reference_index_size_bytes"],
        "reference_catalog_id": run_manifest["reference_catalog_id"],
        "reference_index_backend": run_manifest["reference_index_backend"],
        "decoy_mode": run_manifest["decoy_mode"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "strongest_depletion_tool": strongest_depletion["tool"],
            "strongest_depletion_fraction": strongest_depletion[
                "mean_contaminant_fraction_removed"
            ],
            "best_read_retention_tool": best_retention["tool"],
            "best_read_retention": best_retention["median_read_retention"],
        },
        "tool_summary": tool_summary,
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (docs_root / "lunarc.md").write_text(render_markdown(summary), encoding="utf-8")
    with (docs_root / "sample_results.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(sample_rows[0].keys()))
        writer.writeheader()
        writer.writerows(sample_rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
