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
    MERGE_PAIRS_BENCHMARK_CONTRACT,
    benchmark_applicable_runs,
    benchmark_applicable_sample_ids,
    benchmark_manifest_failure_count,
    benchmark_manifest_sample_ids,
    localize_results_path,
    load_corpus_spec,
    load_json,
    merge_pairs_benchmark_defaults,
    preferred_report_run_root,
    resolve_corpus_report_runtime,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    return parse_corpus_report_args(
        description="Render corpus-01 fastq.merge_pairs benchmark summary.",
        docs_root="docs/benchmark/fastq.merge_pairs/corpus-01",
    )


def safe_median(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.median(values))


def safe_mean(values: list[float]) -> float | None:
    if not values:
        return None
    return float(statistics.mean(values))


def safe_fraction(numerator: float, denominator: float) -> float | None:
    if denominator == 0:
        return None
    return numerator / denominator


def resolve_merge_report_path(
    *,
    sample_id: str,
    reported_path: str,
    run_root: Path,
    local_results_root: Path,
) -> Path:
    primary = localize_results_path(reported_path, local_results_root)
    if primary.is_file():
        return primary
    canonical = run_root / "bench" / "merge_pairs" / sample_id / "report.json"
    if canonical.is_file():
        return canonical
    legacy = run_root / "bench" / "merge" / sample_id / "report.json"
    if legacy.is_file():
        return legacy
    return primary


def record_parameter(record: dict, key: str):
    return record.get("context", {}).get("parameters", {}).get(key)


def format_merge_setting(value) -> str:
    if value is None:
        return "governed tool default"
    return str(value)


def validate_merge_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "merge benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "merge benchmark report rendering requires the full paired corpus run, not --sample-limit output"
        )
    stage_id = run_manifest.get("stage_id", MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id)
    scenario_id = run_manifest.get(
        "scenario_id", MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id
    )
    tool_kind = run_manifest.get("tool_kind", "benchmark")
    if stage_id != MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "merge benchmark report drift: "
            f"expected stage_id {MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id}, found {stage_id}"
        )
    if scenario_id != MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "merge benchmark report drift: "
            f"expected scenario_id {MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id}, found {scenario_id}"
        )
    if tool_kind != "benchmark":
        raise SystemExit(
            f"merge benchmark report drift: expected tool_kind benchmark, found {tool_kind}"
        )
    run_manifest.setdefault("sample_scope", MERGE_PAIRS_BENCHMARK_CONTRACT.sample_scope)
    if run_manifest.get("sample_scope") != MERGE_PAIRS_BENCHMARK_CONTRACT.sample_scope:
        raise SystemExit(
            "merge benchmark report drift: "
            f"expected sample_scope {MERGE_PAIRS_BENCHMARK_CONTRACT.sample_scope}, "
            f"found {run_manifest.get('sample_scope')}"
        )


def validate_merge_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["layout"] != "pe":
            raise SystemExit(
                "merge benchmark report drift: "
                f"sample {row['sample_id']} is not paired-end"
            )
        for key in ["merge_overlap", "min_length", "unmerged_read_policy"]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "merge benchmark report drift: "
                    f"expected {key}={run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
        if row["pairs_in"] <= 0:
            raise SystemExit(
                "merge benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has non-positive pairs_in"
            )
        if row["reads_merged"] > row["pairs_in"]:
            raise SystemExit(
                "merge benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_merged > pairs_in"
            )
        if row["reads_unmerged"] > row["pairs_in"] - row["reads_merged"]:
            raise SystemExit(
                "merge benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_unmerged beyond residual pairs"
            )
        expected_rate = safe_fraction(row["reads_merged"], row["pairs_in"])
        if expected_rate is None or abs(row["merge_rate"] - expected_rate) > 1e-6:
            raise SystemExit(
                "merge benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} merge_rate {row['merge_rate']!r} "
                f"does not match reads_merged/pairs_in {expected_rate!r}"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "merge benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "merge benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.merge_pairs` on `corpus-01`")
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
    lines.append(f"- merge_overlap: `{format_merge_setting(summary['merge_overlap'])}`")
    lines.append(f"- min_length: `{format_merge_setting(summary['min_length'])}`")
    lines.append(f"- unmerged_read_policy: `{summary['unmerged_read_policy']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("best_merge_rate_tool"):
        lines.append(
            f"- Highest median merge rate: `{headline['best_merge_rate_tool']}` at `{headline['best_merge_rate']:.3f}`."
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
        "| Tool | Samples | Pass rate | Median runtime (s) | Median merge rate | Median base retention | Mean merged reads |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        merge_rate = (
            "n/a" if row["median_merge_rate"] is None else f"{row['median_merge_rate']:.3f}"
        )
        base_retention = (
            "n/a"
            if row["median_base_retention"] is None
            else f"{row['median_base_retention']:.3f}"
        )
        merged_reads = (
            "n/a" if row["mean_reads_merged"] is None else f"{row['mean_reads_merged']:.1f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {merge_rate} | {base_retention} | {merged_reads} |"
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
        "- This dossier is intentionally paired-end only. Single-end corpus members are excluded because `fastq.merge_pairs` is not defined for them."
    )
    lines.append(
        "- Sample-level detail is written to `sample_results.csv` beside this report for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    runtime = resolve_corpus_report_runtime(
        args,
        stage_id=MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id,
    )
    repo_root = runtime.repo_root
    corpus_root = runtime.corpus_root
    run_root = runtime.run_root
    docs_root = runtime.docs_root
    local_results_root = runtime.local_results_root

    spec = load_corpus_spec(repo_root)
    defaults = merge_pairs_benchmark_defaults()
    run_manifest = runtime.run_manifest
    validate_merge_run_manifest_contract(run_manifest)
    manifest_sample_ids = benchmark_manifest_sample_ids(run_manifest)

    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=manifest_sample_ids,
    )
    expected_sample_ids = benchmark_applicable_sample_ids(
        MERGE_PAIRS_BENCHMARK_CONTRACT,
        run_manifest,
        metadata_by_sample,
    )
    applicable_runs = benchmark_applicable_runs(
        MERGE_PAIRS_BENCHMARK_CONTRACT,
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
        metadata = metadata_by_sample.get(sample_id, {})
        cohort_key = f"{metadata.get('era', 'unknown')}_{metadata.get('layout', run['layout'])}"
        cohort_counts[cohort_key] += 1
        era_counts[metadata.get("era", "unknown")] += 1
        layout_counts[metadata.get("layout", run["layout"])] += 1

        report_path = resolve_merge_report_path(
            sample_id=sample_id,
            reported_path=run["report_json"],
            run_root=run_root,
            local_results_root=local_results_root,
        )
        if not report_path.is_file():
            raise SystemExit(f"missing report.json for {sample_id}: {report_path}")
        report = load_json(report_path)
        records = report.get("records", [])
        if not records:
            raise SystemExit(f"report.json for {sample_id} contains no records")
        for record in records:
            tool = record.get("context", {}).get("tool", "unknown")
            metrics = record.get("metrics", {}).get("metrics", {})
            row = {
                "sample_id": sample_id,
                "accession": metadata.get("accession"),
                "era": metadata.get("era"),
                "layout": metadata.get("layout", run["layout"]),
                "study_accession": metadata.get("study_accession"),
                "size_band": metadata.get("size_band"),
                "tool": tool,
                "runtime_s": float(record.get("execution", {}).get("runtime_s", 0.0)),
                "exit_code": int(record.get("execution", {}).get("exit_code", 0)),
                "reads_in": int(metrics.get("reads_in", 0)),
                "reads_out": int(metrics.get("reads_out", 0)),
                "bases_in": int(metrics.get("bases_in", 0)),
                "bases_out": int(metrics.get("bases_out", 0)),
                "pairs_in": int(metrics.get("pairs_in", 0)),
                "pairs_out": int(metrics.get("pairs_out", 0)),
                "reads_r1": int(metrics.get("reads_r1", 0)),
                "reads_r2": int(metrics.get("reads_r2", 0)),
                "reads_merged": int(metrics.get("reads_merged", 0)),
                "reads_unmerged": int(metrics.get("reads_unmerged", 0)),
                "merge_rate": float(metrics.get("merge_rate", 0.0)),
                "base_retention": safe_fraction(
                    float(metrics.get("bases_out", 0)),
                    float(metrics.get("bases_in", 0)),
                )
                or 0.0,
                "merge_overlap": record_parameter(record, "merge_overlap"),
                "min_length": record_parameter(record, "min_length"),
                "unmerged_read_policy": record_parameter(record, "unmerged_read_policy"),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    run_manifest.setdefault("merge_overlap", defaults["merge_overlap"])
    run_manifest.setdefault("min_length", defaults["min_length"])
    run_manifest.setdefault(
        "unmerged_read_policy", defaults["unmerged_read_policy"]
    )
    validate_merge_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        runtimes = [float(row["runtime_s"]) for row in rows]
        merge_rates = [float(row["merge_rate"]) for row in rows]
        base_retentions = [float(row["base_retention"]) for row in rows]
        merged_reads = [float(row["reads_merged"]) for row in rows]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median(runtimes),
                "median_merge_rate": safe_median(merge_rates),
                "median_base_retention": safe_median(base_retentions),
                "mean_reads_merged": safe_mean(merged_reads),
            }
        )

    fastest = min(
        tool_summary,
        key=lambda row: row["median_runtime_s"]
        if row["median_runtime_s"] is not None
        else float("inf"),
    )
    best_merge_rate = max(
        tool_summary,
        key=lambda row: row["median_merge_rate"]
        if row["median_merge_rate"] is not None
        else float("-inf"),
    )
    best_base_retention = max(
        tool_summary,
        key=lambda row: row["median_base_retention"]
        if row["median_base_retention"] is not None
        else float("-inf"),
    )

    summary = {
        "schema_version": "bijux.fastq.merge_pairs.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "sample_scope": run_manifest["sample_scope"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": benchmark_manifest_failure_count(applicable_runs),
        "tools": run_manifest["tools"],
        "merge_overlap": run_manifest["merge_overlap"],
        "min_length": run_manifest["min_length"],
        "unmerged_read_policy": run_manifest["unmerged_read_policy"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_merge_rate_tool": best_merge_rate["tool"],
            "best_merge_rate": best_merge_rate["median_merge_rate"],
            "best_base_retention_tool": best_base_retention["tool"],
            "best_base_retention": best_base_retention["median_base_retention"],
        },
        "tool_summary": tool_summary,
    }

    sample_results_path = docs_root / "sample_results.csv"
    with sample_results_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "sample_id",
                "accession",
                "study_accession",
                "era",
                "layout",
                "size_band",
                "tool",
                "runtime_s",
                "exit_code",
                "reads_in",
                "reads_out",
                "bases_in",
                "bases_out",
                "pairs_in",
                "pairs_out",
                "reads_r1",
                "reads_r2",
                "reads_merged",
                "reads_unmerged",
                "merge_rate",
                "base_retention",
                "merge_overlap",
                "min_length",
                "unmerged_read_policy",
            ],
        )
        writer.writeheader()
        writer.writerows(sample_rows)

    tool_summary_path = docs_root / "tool_runtime_summary.csv"
    with tool_summary_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "tool",
                "records",
                "pass_rate",
                "median_runtime_s",
                "median_merge_rate",
                "median_base_retention",
                "mean_reads_merged",
            ],
        )
        writer.writeheader()
        writer.writerows(tool_summary)

    cohort_summary_rows = []
    grouped_rows: dict[tuple[str, str, str], list[dict]] = defaultdict(list)
    for row in sample_rows:
        grouped_rows[(row["tool"], "era_layout", f"{row['era']}_{row['layout']}")].append(row)
        grouped_rows[(row["tool"], "size_band", row["size_band"])].append(row)
    for (tool, dimension, cohort), rows in sorted(grouped_rows.items()):
        cohort_summary_rows.append(
            {
                "tool": tool,
                "dimension": dimension,
                "cohort": cohort,
                "samples": len(rows),
                "mean_runtime_s": safe_mean([float(row["runtime_s"]) for row in rows]),
                "median_runtime_s": safe_median([float(row["runtime_s"]) for row in rows]),
                "mean_merge_rate": safe_mean([float(row["merge_rate"]) for row in rows]),
                "median_base_retention": safe_median(
                    [float(row["base_retention"]) for row in rows]
                ),
            }
        )
    cohort_summary_path = docs_root / "cohort_runtime_summary.csv"
    with cohort_summary_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "tool",
                "dimension",
                "cohort",
                "samples",
                "mean_runtime_s",
                "median_runtime_s",
                "mean_merge_rate",
                "median_base_retention",
            ],
        )
        writer.writeheader()
        writer.writerows(cohort_summary_rows)

    outlier_rows = []
    by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        by_sample[row["sample_id"]].append(row)
    for sample_id, rows in sorted(by_sample.items()):
        slowest = max(rows, key=lambda row: float(row["runtime_s"]))
        best_merge = max(rows, key=lambda row: float(row["merge_rate"]))
        outlier_rows.append(
            {
                "sample_id": sample_id,
                "accession": rows[0]["accession"],
                "study_accession": rows[0]["study_accession"],
                "era": rows[0]["era"],
                "layout": rows[0]["layout"],
                "size_band": rows[0]["size_band"],
                "total_runtime_s": sum(float(row["runtime_s"]) for row in rows),
                "slowest_tool": slowest["tool"],
                "slowest_runtime_s": slowest["runtime_s"],
                "best_merge_rate_tool": best_merge["tool"],
                "best_merge_rate": best_merge["merge_rate"],
            }
        )
    outlier_rows.sort(key=lambda row: row["total_runtime_s"], reverse=True)
    outlier_path = docs_root / "sample_runtime_outliers.csv"
    with outlier_path.open("w", encoding="utf-8", newline="") as handle:
        writer = csv.DictWriter(
            handle,
            fieldnames=[
                "sample_id",
                "accession",
                "study_accession",
                "era",
                "layout",
                "size_band",
                "total_runtime_s",
                "slowest_tool",
                "slowest_runtime_s",
                "best_merge_rate_tool",
                "best_merge_rate",
            ],
        )
        writer.writeheader()
        writer.writerows(outlier_rows)

    summary_path = docs_root / "summary.json"
    summary_path.write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")

    markdown_path = docs_root / "benchmark.md"
    markdown_path.write_text(render_markdown(summary), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
