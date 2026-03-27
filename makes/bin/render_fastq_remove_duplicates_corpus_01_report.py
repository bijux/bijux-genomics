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
    REMOVE_DUPLICATES_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    remove_duplicates_benchmark_defaults,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.remove_duplicates benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="/home/bijan/bijux/corpus_01")
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.remove_duplicates/corpus-01",
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


def record_value(record: dict, key: str):
    metric_value = normalize_metric(record, key)
    if metric_value is not None:
        return metric_value
    return record.get("context", {}).get("parameters", {}).get(key)


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {
        "clumpify": "clumpify_log",
        "fastuniq": "fastuniq_log",
    }.get(tool)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = remove_duplicates_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "remove-duplicates benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "remove-duplicates benchmark report rendering requires the full paired corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != REMOVE_DUPLICATES_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "remove-duplicates benchmark report drift: "
            f"expected stage_id {REMOVE_DUPLICATES_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != REMOVE_DUPLICATES_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "remove-duplicates benchmark report drift: "
            f"expected scenario_id {REMOVE_DUPLICATES_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "remove-duplicates benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    run_manifest.setdefault("dedup_mode", defaults["dedup_mode"])
    run_manifest.setdefault("keep_order", defaults["keep_order"])


def validate_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["layout"] != "pe":
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"sample {row['sample_id']} is not paired-end"
            )
        if row["dedup_mode"] != run_manifest["dedup_mode"]:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"expected dedup_mode {run_manifest['dedup_mode']}, "
                f"found {row['dedup_mode']} for {row['sample_id']}/{row['tool']}"
            )
        if row["keep_order"] != run_manifest["keep_order"]:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"expected keep_order {run_manifest['keep_order']}, "
                f"found {row['keep_order']} for {row['sample_id']}/{row['tool']}"
            )
        if row["paired_mode"] != "paired_end":
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"expected paired_end mode for {row['sample_id']}/{row['tool']}, found {row['paired_mode']}"
            )
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )
        if row["reads_out"] > row["reads_in"]:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if row["duplicate_reads"] != row["reads_in"] - row["reads_out"]:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} duplicate_reads does not match reads_in - reads_out"
            )
        if row["pair_count_match"] is False:
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} reports pair_count_match=false"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "remove-duplicates benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "remove-duplicates benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.remove_duplicates` on `corpus-01`")
    lines.append("")
    lines.append("## Run Contract")
    lines.append("")
    lines.append(f"- Generated: {summary['generated_at_utc']}")
    lines.append(f"- Platform: `{summary['platform']}`")
    lines.append(f"- Corpus root: `{summary['corpus_root']}`")
    lines.append(f"- Run root: `{summary['run_root']}`")
    lines.append(f"- Scenario: `{summary['scenario_id']}`")
    lines.append(f"- Samples benchmarked: `{summary['samples_total']}` paired-end inputs")
    lines.append(
        f"- Era balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern"
    )
    lines.append(f"- Tool roster: `{', '.join(summary['tools'])}`")
    lines.append(f"- dedup_mode: `{summary['dedup_mode']}`")
    lines.append(f"- keep_order: `{summary['keep_order']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest median deduplication rate: `{summary['headline']['best_dedup_tool']}` at `{summary['headline']['best_dedup_rate']:.3f}`."
    )
    lines.append(
        f"- Highest mean duplicate removal: `{summary['headline']['strongest_tool']}` at `{summary['headline']['strongest_duplicate_reads']:.1f}` reads."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median dedup rate | Mean duplicate reads |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_dedup_rate']:.3f} | {row['mean_duplicate_reads']:.1f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This paired-only benchmark holds one stable deduplication contract across the full cohort so rate differences remain attributable to backend behavior rather than policy drift."
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
            corpus_root, REMOVE_DUPLICATES_BENCHMARK_CONTRACT.stage_id
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
    era_counts: dict[str, int] = defaultdict(int)
    cohort_counts: dict[str, int] = defaultdict(int)

    for run in run_manifest["runs"]:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample[sample_id]
        era_counts[metadata["era"]] += 1
        cohort_counts[f"{metadata['era']}_{metadata['layout']}"] += 1
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
            duplicate_reads = normalize_metric(record, "duplicate_reads")
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
                "duplicate_reads": int(
                    duplicate_reads if duplicate_reads is not None else reads_in - reads_out
                ),
                "dedup_rate": float(normalize_metric(record, "dedup_rate") or 0.0),
                "paired_mode": record_value(record, "paired_mode"),
                "dedup_mode": record_value(record, "dedup_mode"),
                "keep_order": record_value(record, "keep_order"),
                "pair_count_match": record_value(record, "pair_count_match"),
                "raw_backend_report_format": record_value(
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
        dedup_rates = [float(row["dedup_rate"]) for row in rows]
        duplicate_reads = [float(row["duplicate_reads"]) for row in rows]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_dedup_rate": safe_median(dedup_rates),
                "mean_duplicate_reads": safe_mean(duplicate_reads),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    best_dedup = max(tool_summary, key=lambda row: row["median_dedup_rate"])
    strongest = max(tool_summary, key=lambda row: row["mean_duplicate_reads"])
    summary = {
        "schema_version": "bijux.fastq.remove_duplicates.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "platform": run_manifest["platform"],
        "stage_id": run_manifest["stage_id"],
        "scenario_id": run_manifest["scenario_id"],
        "corpus_root": run_manifest["corpus_root"],
        "run_root": str(run_root),
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "tools": run_manifest["tools"],
        "dedup_mode": run_manifest["dedup_mode"],
        "keep_order": run_manifest["keep_order"],
        "era_counts": dict(sorted(era_counts.items())),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_dedup_tool": best_dedup["tool"],
            "best_dedup_rate": best_dedup["median_dedup_rate"],
            "strongest_tool": strongest["tool"],
            "strongest_duplicate_reads": strongest["mean_duplicate_reads"],
        },
        "tool_summary": tool_summary,
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2) + "\n",
        encoding="utf-8",
    )
    if sample_rows:
        with (docs_root / "sample_results.csv").open("w", encoding="utf-8", newline="") as handle:
            writer = csv.DictWriter(handle, fieldnames=list(sample_rows[0].keys()))
            writer.writeheader()
            writer.writerows(sample_rows)
    (docs_root / "lunarc.md").write_text(render_markdown(summary), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
