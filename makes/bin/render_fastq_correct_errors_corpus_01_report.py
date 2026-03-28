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
    CORRECT_ERRORS_BENCHMARK_CONTRACT,
    benchmark_applicable_runs,
    benchmark_applicable_sample_ids,
    benchmark_manifest_failure_count,
    benchmark_manifest_sample_ids,
    correct_errors_benchmark_defaults,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.correct_errors benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.correct_errors/corpus-01",
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


def correction_effect(record: dict) -> dict:
    payload = normalize_metric(record, "correction_effect")
    return payload if isinstance(payload, dict) else {}


def projected_policy(run_manifest: dict, tool: str) -> dict[str, object]:
    projected = {
        "quality_encoding": run_manifest["quality_encoding"],
        "kmer_size": run_manifest["kmer_size"],
        "musket_kmer_budget": run_manifest["musket_kmer_budget"],
        "genome_size": run_manifest["genome_size"],
        "max_memory_gb": run_manifest["max_memory_gb"],
        "trusted_kmer_artifact": run_manifest["trusted_kmer_artifact"],
        "conservative_mode": run_manifest["conservative_mode"],
    }
    if tool == "musket":
        projected["genome_size"] = None
        projected["max_memory_gb"] = None
        projected["trusted_kmer_artifact"] = None
    elif tool == "lighter":
        projected["musket_kmer_budget"] = None
    elif tool == "bayeshammer":
        projected["kmer_size"] = None
        projected["musket_kmer_budget"] = None
        projected["genome_size"] = None
        projected["trusted_kmer_artifact"] = None
    elif tool == "rcorrector":
        projected["kmer_size"] = None
        projected["musket_kmer_budget"] = None
        projected["genome_size"] = None
        projected["max_memory_gb"] = None
        projected["trusted_kmer_artifact"] = None
    return projected


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = correct_errors_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "correct-errors benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "correct-errors benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != CORRECT_ERRORS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "correct-errors benchmark report drift: "
            f"expected stage_id {CORRECT_ERRORS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != CORRECT_ERRORS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "correct-errors benchmark report drift: "
            f"expected scenario_id {CORRECT_ERRORS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "correct-errors benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    run_manifest.setdefault(
        "sample_scope", CORRECT_ERRORS_BENCHMARK_CONTRACT.sample_scope
    )
    if run_manifest.get("sample_scope") != CORRECT_ERRORS_BENCHMARK_CONTRACT.sample_scope:
        raise SystemExit(
            "correct-errors benchmark report drift: "
            f"expected sample_scope {CORRECT_ERRORS_BENCHMARK_CONTRACT.sample_scope}, "
            f"found {run_manifest.get('sample_scope')}"
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
        expected_policy = projected_policy(run_manifest, row["tool"])
        for key in [
            "quality_encoding",
            "kmer_size",
            "musket_kmer_budget",
            "genome_size",
            "max_memory_gb",
            "trusted_kmer_artifact",
            "conservative_mode",
        ]:
            if row[key] != expected_policy[key]:
                raise SystemExit(
                    "correct-errors benchmark report drift: "
                    f"expected {key} {expected_policy[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
        expected_paired_mode = "paired_end" if row["layout"] == "pe" else "single_end"
        if row["paired_mode"] != expected_paired_mode:
            raise SystemExit(
                "correct-errors benchmark report drift: "
                f"sample {row['sample_id']}/{row['tool']} expected {expected_paired_mode}, "
                f"found {row['paired_mode']}"
            )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "correct-errors benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["bases_out"]) > int(row["bases_in"]):
            raise SystemExit(
                "correct-errors benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has bases_out > bases_in"
            )
        if int(row["corrected_reads"]) > int(row["reads_out"]):
            raise SystemExit(
                "correct-errors benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has corrected_reads > reads_out"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "correct-errors benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "correct-errors benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.correct_errors` on `corpus-01`")
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
    lines.append(f"- quality_encoding: `{summary['quality_encoding']}`")
    lines.append(f"- kmer_size: `{summary['kmer_size']}`")
    lines.append(f"- musket_kmer_budget: `{summary['musket_kmer_budget']}`")
    lines.append(f"- genome_size: `{summary['genome_size']}`")
    lines.append(f"- max_memory_gb: `{summary['max_memory_gb']}`")
    lines.append(f"- trusted_kmer_artifact: `{summary['trusted_kmer_artifact']}`")
    lines.append(
        f"- trusted_kmer_artifact_digest: `{summary['trusted_kmer_artifact_digest']}`"
    )
    lines.append(f"- conservative_mode: `{summary['conservative_mode']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest mean k-mer fix rate: `{summary['headline']['best_kmer_fix_tool']}` at `{summary['headline']['best_kmer_fix_rate']:.3f}`."
    )
    lines.append(
        f"- Highest mean quality uplift: `{summary['headline']['best_quality_uplift_tool']}` at `{summary['headline']['best_quality_uplift']:.3f}`."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Mean corrected reads | Mean k-mer fix rate | Mean quality uplift |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_read_retention']:.3f} | {row['mean_corrected_reads']:.1f} | {row['mean_kmer_fix_rate']:.3f} | {row['mean_quality_uplift']:.3f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This dossier keeps the governed correction policy explicit so runtime and output-change differences remain attributable to tool behavior rather than hidden parameter drift."
    )
    lines.append(
        "- The benchmark records both k-mer-fix rate and quality uplift, because correction stages can leave read counts unchanged while still changing the sequence content materially."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser()
        if args.run_root
        else preferred_report_run_root(corpus_root, CORRECT_ERRORS_BENCHMARK_CONTRACT.stage_id)
    )
    local_results_root = run_root.parents[2]
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)

    spec = load_corpus_spec(repo_root)
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_run_manifest_contract(run_manifest)
    manifest_sample_ids = benchmark_manifest_sample_ids(run_manifest)
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=manifest_sample_ids,
    )
    expected_sample_ids = benchmark_applicable_sample_ids(
        CORRECT_ERRORS_BENCHMARK_CONTRACT,
        run_manifest,
        metadata_by_sample,
    )
    applicable_runs = benchmark_applicable_runs(
        CORRECT_ERRORS_BENCHMARK_CONTRACT,
        run_manifest,
        metadata_by_sample,
    )

    sample_rows: list[dict] = []
    tool_rows: dict[str, list[dict]] = defaultdict(list)
    cohort_counts: dict[str, int] = defaultdict(int)
    era_counts: dict[str, int] = defaultdict(int)
    layout_counts: dict[str, int] = defaultdict(int)

    for run in applicable_runs:
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
            effect = correction_effect(record)
            reads_in = int(normalize_metric(record, "reads_in") or 0)
            reads_out = int(normalize_metric(record, "reads_out") or 0)
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
                "paired_mode": normalize_metric(record, "paired_mode"),
                "correction_engine": normalize_metric(record, "correction_engine") or tool,
                "quality_encoding": normalize_metric(record, "quality_encoding")
                or run_manifest["quality_encoding"],
                "kmer_size": normalize_metric(record, "kmer_size"),
                "musket_kmer_budget": normalize_metric(record, "musket_kmer_budget"),
                "genome_size": normalize_metric(record, "genome_size"),
                "max_memory_gb": normalize_metric(record, "max_memory_gb"),
                "trusted_kmer_artifact": normalize_metric(record, "trusted_kmer_artifact"),
                "conservative_mode": normalize_metric(record, "conservative_mode"),
                "reads_in": reads_in,
                "reads_out": reads_out,
                "bases_in": int(normalize_metric(record, "bases_in") or 0),
                "bases_out": int(normalize_metric(record, "bases_out") or 0),
                "pairs_in": int(normalize_metric(record, "pairs_in") or 0),
                "pairs_out": int(normalize_metric(record, "pairs_out") or 0),
                "corrected_reads": int(normalize_metric(record, "corrected_reads") or 0),
                "mean_q_before": float(normalize_metric(record, "mean_q_before") or 0.0),
                "mean_q_after": float(normalize_metric(record, "mean_q_after") or 0.0),
                "mean_q_delta": float(effect.get("mean_q_delta") or 0.0),
                "outputs_changed": bool(effect.get("outputs_changed")),
                "kmer_fix_rate": float(normalize_metric(record, "kmer_fix_rate") or 0.0),
                "read_retention": safe_fraction(reads_out, reads_in),
                "raw_backend_report_format": normalize_metric(
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
        retentions = [float(row["read_retention"] or 0.0) for row in rows]
        corrected_reads = [float(row["corrected_reads"]) for row in rows]
        kmer_fix_rates = [float(row["kmer_fix_rate"]) for row in rows]
        quality_uplift = [float(row["mean_q_delta"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_read_retention": safe_median(retentions),
                "mean_corrected_reads": safe_mean(corrected_reads),
                "mean_kmer_fix_rate": safe_mean(kmer_fix_rates),
                "mean_quality_uplift": safe_mean(quality_uplift),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    best_kmer_fix = max(tool_summary, key=lambda row: row["mean_kmer_fix_rate"])
    best_quality_uplift = max(
        tool_summary, key=lambda row: row["mean_quality_uplift"]
    )
    summary = {
        "schema_version": "bijux.fastq.correct_errors.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": CORRECT_ERRORS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": CORRECT_ERRORS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "sample_scope": run_manifest["sample_scope"],
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": benchmark_manifest_failure_count(applicable_runs),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "quality_encoding": run_manifest["quality_encoding"],
        "kmer_size": run_manifest["kmer_size"],
        "musket_kmer_budget": run_manifest["musket_kmer_budget"],
        "genome_size": run_manifest["genome_size"],
        "max_memory_gb": run_manifest["max_memory_gb"],
        "trusted_kmer_artifact": run_manifest["trusted_kmer_artifact"],
        "trusted_kmer_artifact_digest": run_manifest["trusted_kmer_artifact_digest"],
        "trusted_kmer_artifact_size_bytes": run_manifest[
            "trusted_kmer_artifact_size_bytes"
        ],
        "conservative_mode": run_manifest["conservative_mode"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_kmer_fix_tool": best_kmer_fix["tool"],
            "best_kmer_fix_rate": best_kmer_fix["mean_kmer_fix_rate"],
            "best_quality_uplift_tool": best_quality_uplift["tool"],
            "best_quality_uplift": best_quality_uplift["mean_quality_uplift"],
        },
        "tool_summary": tool_summary,
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    (docs_root / "benchmark.md").write_text(render_markdown(summary), encoding="utf-8")
    with (docs_root / "sample_results.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(sample_rows[0].keys()))
        writer.writeheader()
        writer.writerows(sample_rows)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
