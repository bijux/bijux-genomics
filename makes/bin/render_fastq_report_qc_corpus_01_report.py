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
    REPORT_QC_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.report_qc benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="/home/bijan/bijux/corpus_01")
    parser.add_argument(
        "--run-root",
        default="",
        help="Benchmark run root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.report_qc/lunarc.",
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.report_qc/corpus-01",
        help="Directory where summary artifacts should be written.",
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


def enrich_multiqc_artifacts(row: dict) -> dict:
    data_dir = Path(row["multiqc_data"]) if row.get("multiqc_data") else None
    if data_dir and not row.get("multiqc_report"):
        report_path = data_dir / "multiqc_report.html"
        if report_path.is_file():
            row["multiqc_report"] = str(report_path)
    data_json = None
    if data_dir:
        candidate = data_dir / "multiqc_report_data" / "multiqc_data.json"
        if candidate.is_file():
            data_json = load_json(candidate)
    if data_json is not None:
        if row.get("multiqc_sample_count") is None:
            general_stats = data_json.get("report_general_stats_data")
            if isinstance(general_stats, list) and general_stats:
                first = general_stats[0]
                if isinstance(first, dict):
                    row["multiqc_sample_count"] = len(first)
        if row.get("multiqc_module_count") is None:
            plot_data = data_json.get("report_plot_data")
            if isinstance(plot_data, dict):
                row["multiqc_module_count"] = len(plot_data)
    return row


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "report-qc benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "report-qc benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != REPORT_QC_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "report-qc benchmark report drift: "
            f"expected stage_id {REPORT_QC_BENCHMARK_CONTRACT.stage_id}, found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != REPORT_QC_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "report-qc benchmark report drift: "
            f"expected scenario_id {REPORT_QC_BENCHMARK_CONTRACT.scenario_id}, found {run_manifest.get('scenario_id')}"
        )
    expected_contract = {
        "tool_kind": "benchmark",
        "aggregation_engine": "multiqc",
        "aggregation_scope": "governed_qc_artifacts",
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
    }
    for key, expected in expected_contract.items():
        if run_manifest.get(key) != expected:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"expected {key}={expected!r}, found {run_manifest.get(key)!r}"
            )


def validate_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["reads_out"] != row["reads_in"]:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"reads_out must equal reads_in for {row['sample_id']}/{row['tool']}"
            )
        if row["bases_out"] != row["bases_in"]:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"bases_out must equal bases_in for {row['sample_id']}/{row['tool']}"
            )
        if row["governed_qc_input_count"] <= 0:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"governed_qc_input_count must be positive for {row['sample_id']}/{row['tool']}"
            )
        if row["governed_qc_input_count"] != row["expected_governed_qc_input_count"]:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"governed_qc_input_count mismatch for {row['sample_id']}/{row['tool']}"
            )
        if not 0.0 <= row["mean_q"] <= 45.0:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"mean_q must be within [0, 45] for {row['sample_id']}/{row['tool']}"
            )
        if not 0.0 <= row["contamination_rate"] <= 1.0:
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"contamination_rate must be within [0, 1] for {row['sample_id']}/{row['tool']}"
            )
        if row["exit_code"] == 0:
            for key in [
                "governed_qc_manifest_artifact",
                "raw_fastqc_dir",
                "multiqc_report",
                "multiqc_data",
                "report_json_artifact",
            ]:
                if not row[key]:
                    raise SystemExit(
                        "report-qc benchmark report drift: "
                        f"{key} must be populated for {row['sample_id']}/{row['tool']}"
                    )
                if not Path(row[key]).exists():
                    raise SystemExit(
                        "report-qc benchmark report drift: "
                        f"{key} path does not exist for {row['sample_id']}/{row['tool']}: {row[key]}"
                    )
            if row["multiqc_sample_count"] is None or row["multiqc_sample_count"] <= 0:
                raise SystemExit(
                    "report-qc benchmark report drift: "
                    f"multiqc_sample_count must be positive for {row['sample_id']}/{row['tool']}"
                )
            if row["multiqc_module_count"] is None or row["multiqc_module_count"] <= 0:
                raise SystemExit(
                    "report-qc benchmark report drift: "
                    f"multiqc_module_count must be positive for {row['sample_id']}/{row['tool']}"
                )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "report-qc benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.report_qc` on `corpus-01`")
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
    lines.append(f"- aggregation_engine: `{summary['aggregation_engine']}`")
    lines.append(f"- aggregation_scope: `{summary['aggregation_scope']}`")
    lines.append(
        f"- Governed contributor stages: `{', '.join(summary['governed_contributor_stage_ids'])}`"
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("largest_multiqc_module_tool"):
        lines.append(
            f"- Highest median MultiQC module count: `{headline['largest_multiqc_module_tool']}` at `{headline['largest_multiqc_module_count']:.1f}`."
        )
    if headline.get("highest_governed_input_tool"):
        lines.append(
            f"- Highest median governed QC input count: `{headline['highest_governed_input_tool']}` at `{headline['highest_governed_input_count']:.1f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median modules | Median sample count | Median governed inputs | Median contamination rate | Median mean Q |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        modules = (
            "n/a"
            if row["median_multiqc_module_count"] is None
            else f"{row['median_multiqc_module_count']:.1f}"
        )
        sample_count = (
            "n/a"
            if row["median_multiqc_sample_count"] is None
            else f"{row['median_multiqc_sample_count']:.1f}"
        )
        governed_inputs = (
            "n/a"
            if row["median_governed_qc_input_count"] is None
            else f"{row['median_governed_qc_input_count']:.1f}"
        )
        contamination = (
            "n/a"
            if row["median_contamination_rate"] is None
            else f"{row['median_contamination_rate']:.4f}"
        )
        mean_q = "n/a" if row["median_mean_q"] is None else f"{row['median_mean_q']:.3f}"
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {modules} | {sample_count} | {governed_inputs} | {contamination} | {mean_q} |"
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
        "- This stage is report-only and non-mutating: governed benchmarking confirms MultiQC aggregation behavior without changing the reads."
    )
    lines.append(
        "- `sample_results.csv` beside this report keeps the per-sample MultiQC counts, contamination carry-through, and governed manifest lineage for deeper inspection."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).resolve()
    run_root = (
        Path(args.run_root).resolve()
        if args.run_root
        else preferred_report_run_root(
            corpus_root, REPORT_QC_BENCHMARK_CONTRACT.stage_id
        ).resolve()
    )
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)
    local_results_root = run_root.parents[2]

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
        metadata = metadata_by_sample.get(sample_id, {})
        cohort_key = f"{metadata.get('era', 'unknown')}_{metadata.get('layout', run['layout'])}"
        cohort_counts[cohort_key] += 1
        era_counts[metadata.get("era", "unknown")] += 1
        layout_counts[metadata.get("layout", run["layout"])] += 1

        report_path = localize_results_path(run["report_json"], local_results_root)
        if not report_path.is_file():
            raise SystemExit(
                "report-qc benchmark report drift: "
                f"missing report.json for {sample_id}: {report_path}"
            )
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
                "pairs_in": normalize_metric(record, "pairs_in"),
                "pairs_out": normalize_metric(record, "pairs_out"),
                "mean_q": normalize_metric(record, "mean_q") or 0.0,
                "contamination_rate": normalize_metric(record, "contamination_rate") or 0.0,
                "multiqc_sample_count": normalize_metric(record, "multiqc_sample_count"),
                "multiqc_module_count": normalize_metric(record, "multiqc_module_count"),
                "governed_qc_input_count": normalize_metric(
                    record, "governed_qc_input_count"
                )
                or 0,
                "expected_governed_qc_input_count": run["governed_qc_input_count"],
                "governed_qc_lineage_hash": normalize_metric(
                    record, "governed_qc_lineage_hash"
                ),
                "raw_fastqc_dir": str(
                    localize_results_path(
                        normalize_metric(record, "raw_fastqc_dir"),
                        local_results_root,
                    )
                )
                if normalize_metric(record, "raw_fastqc_dir")
                else "",
                "multiqc_report": str(
                    localize_results_path(
                        normalize_metric(record, "multiqc_report"),
                        local_results_root,
                    )
                )
                if normalize_metric(record, "multiqc_report")
                else "",
                "multiqc_data": str(
                    localize_results_path(
                        normalize_metric(record, "multiqc_data"),
                        local_results_root,
                    )
                )
                if normalize_metric(record, "multiqc_data")
                else "",
                "report_json_artifact": str(
                    localize_results_path(run["report_json"], local_results_root)
                    .parent
                    .joinpath("tools")
                    .joinpath(tool)
                    .joinpath("report_qc_report.json")
                ),
                "governed_qc_manifest_artifact": str(
                    localize_results_path(run["governed_qc_manifest"], local_results_root)
                ),
            }
            row = enrich_multiqc_artifacts(row)
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        runtimes = [float(row["runtime_s"]) for row in rows]
        modules = [
            float(row["multiqc_module_count"])
            for row in rows
            if row["multiqc_module_count"] is not None
        ]
        sample_counts = [
            float(row["multiqc_sample_count"])
            for row in rows
            if row["multiqc_sample_count"] is not None
        ]
        governed_inputs = [float(row["governed_qc_input_count"]) for row in rows]
        contamination = [float(row["contamination_rate"]) for row in rows]
        mean_q_values = [float(row["mean_q"]) for row in rows]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median(runtimes),
                "median_multiqc_module_count": safe_median(modules),
                "median_multiqc_sample_count": safe_median(sample_counts),
                "median_governed_qc_input_count": safe_median(governed_inputs),
                "median_contamination_rate": safe_median(contamination),
                "median_mean_q": safe_median(mean_q_values),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        largest_modules = max(
            (row for row in tool_summary if row["median_multiqc_module_count"] is not None),
            key=lambda row: row["median_multiqc_module_count"],
        )
        largest_inputs = max(
            (
                row
                for row in tool_summary
                if row["median_governed_qc_input_count"] is not None
            ),
            key=lambda row: row["median_governed_qc_input_count"],
        )
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "largest_multiqc_module_tool": largest_modules["tool"],
            "largest_multiqc_module_count": largest_modules["median_multiqc_module_count"],
            "highest_governed_input_tool": largest_inputs["tool"],
            "highest_governed_input_count": largest_inputs[
                "median_governed_qc_input_count"
            ],
        }

    summary = {
        "schema_version": "bijux.fastq.report_qc.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": REPORT_QC_BENCHMARK_CONTRACT.stage_id,
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
        "aggregation_engine": run_manifest["aggregation_engine"],
        "aggregation_scope": run_manifest["aggregation_scope"],
        "report_only": run_manifest["report_only"],
        "mutates_fastq": run_manifest["mutates_fastq"],
        "may_change_read_count": run_manifest["may_change_read_count"],
        "governed_contributor_stage_ids": run_manifest["governed_contributor_stage_ids"],
        "governed_contributor_tool_ids": run_manifest["governed_contributor_tool_ids"],
        "headline": headline,
        "tool_summary": tool_summary,
    }
    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2, sort_keys=True) + "\n",
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
                "pairs_in",
                "pairs_out",
                "mean_q",
                "contamination_rate",
                "multiqc_sample_count",
                "multiqc_module_count",
                "governed_qc_input_count",
                "expected_governed_qc_input_count",
                "governed_qc_lineage_hash",
                "raw_fastqc_dir",
                "multiqc_report",
                "multiqc_data",
                "report_json_artifact",
                "governed_qc_manifest_artifact",
            ],
        )
        writer.writeheader()
        writer.writerows(sample_rows)

    (docs_root / "lunarc.md").write_text(render_markdown(summary), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
