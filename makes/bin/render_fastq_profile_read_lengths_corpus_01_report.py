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
    PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.profile_read_lengths benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument(
        "--run-root",
        default="",
        help="Benchmark run root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.profile_read_lengths/lunarc.",
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.profile_read_lengths/corpus-01",
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


def normalize_metric(record: dict, key: str):
    metrics = record.get("metrics", {})
    metrics_payload = metrics.get("metrics", metrics)
    return metrics_payload.get(key)


def validate_read_length_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "read-length benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "read-length benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"expected stage_id {PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.stage_id}, found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"expected scenario_id {PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.scenario_id}, found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind") != "benchmark":
        raise SystemExit(
            "read-length benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    expected_contract = {
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
        "raw_backend_report_format": "seqkit_stats_length_histogram",
    }
    for key, expected in expected_contract.items():
        if run_manifest.get(key) != expected:
            raise SystemExit(
                "read-length benchmark report drift: "
                f"expected {key}={expected!r}, found {run_manifest.get(key)!r}"
            )
    if int(run_manifest.get("histogram_bins", 0)) <= 0:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"histogram_bins must be positive, found {run_manifest.get('histogram_bins')!r}"
        )
    expected_artifacts = [
        "report_json",
        "length_distribution_tsv",
        "length_distribution_json",
    ]
    if run_manifest.get("length_histogram_artifacts") != expected_artifacts:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"expected length_histogram_artifacts {expected_artifacts}, found {run_manifest.get('length_histogram_artifacts')!r}"
        )


def tool_output_dir(run_report_path: Path, tool: str) -> Path:
    return run_report_path.parent / "tools" / tool


def validate_artifact_paths(report_path: Path, tool: str) -> dict[str, str]:
    tool_dir = tool_output_dir(report_path, tool)
    report_json = tool_dir / "profile_read_lengths_report.json"
    histogram_tsv = tool_dir / "length_distribution.tsv"
    histogram_json = tool_dir / "length_distribution.json"
    missing = [
        str(path)
        for path in [report_json, histogram_tsv, histogram_json]
        if not path.is_file()
    ]
    if missing:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"missing governed histogram artifacts for {tool}: {missing}"
        )
    empty = [
        str(path)
        for path in [report_json, histogram_tsv, histogram_json]
        if path.stat().st_size == 0
    ]
    if empty:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"empty governed histogram artifacts for {tool}: {empty}"
        )
    return {
        "report_json_artifact": str(report_json),
        "length_distribution_tsv_artifact": str(histogram_tsv),
        "length_distribution_json_artifact": str(histogram_json),
    }


def validate_read_length_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if int(row["read_count"]) <= 0:
            raise SystemExit(
                "read-length benchmark report drift: "
                f"read_count must be positive for {row['sample_id']}/{row['tool']}"
            )
        if float(row["mean_read_length"]) <= 0.0:
            raise SystemExit(
                "read-length benchmark report drift: "
                f"mean_read_length must be positive for {row['sample_id']}/{row['tool']}"
            )
        if int(row["max_read_length"]) < float(row["mean_read_length"]):
            raise SystemExit(
                "read-length benchmark report drift: "
                f"max_read_length must be >= mean_read_length for {row['sample_id']}/{row['tool']}"
            )
        if int(row["distinct_lengths"]) <= 0:
            raise SystemExit(
                "read-length benchmark report drift: "
                f"distinct_lengths must be positive for {row['sample_id']}/{row['tool']}"
            )
        if int(row["distinct_lengths"]) > int(row["read_count"]):
            raise SystemExit(
                "read-length benchmark report drift: "
                f"distinct_lengths must be <= read_count for {row['sample_id']}/{row['tool']}"
            )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "read-length benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "read-length benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.profile_read_lengths` on `corpus-01`")
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
    lines.append(f"- report_only: `{summary['report_only']}`")
    lines.append(f"- mutates_fastq: `{summary['mutates_fastq']}`")
    lines.append(f"- may_change_read_count: `{summary['may_change_read_count']}`")
    lines.append(f"- raw_backend_report_format: `{summary['raw_backend_report_format']}`")
    lines.append(f"- histogram_bins: `{summary['histogram_bins']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("highest_max_read_length_tool"):
        lines.append(
            f"- Highest median max read length: `{headline['highest_max_read_length_tool']}` at `{headline['highest_max_read_length']:.1f}`."
        )
    if headline.get("widest_length_support_tool"):
        lines.append(
            f"- Widest median distinct-length support: `{headline['widest_length_support_tool']}` at `{headline['widest_length_support']:.1f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read count | Median mean read length | Median max read length | Median distinct lengths |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        read_count = "n/a" if row["median_read_count"] is None else f"{row['median_read_count']:.1f}"
        mean_length = (
            "n/a"
            if row["median_mean_read_length"] is None
            else f"{row['median_mean_read_length']:.3f}"
        )
        max_length = (
            "n/a"
            if row["median_max_read_length"] is None
            else f"{row['median_max_read_length']:.1f}"
        )
        distinct_lengths = (
            "n/a"
            if row["median_distinct_lengths"] is None
            else f"{row['median_distinct_lengths']:.1f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {read_count} | {mean_length} | {max_length} | {distinct_lengths} |"
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
        "- This stage is report-only and non-mutating: governed benchmarking confirms runtime and read-length distribution stability without changing the reads."
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
        Path(args.run_root).expanduser()
        if args.run_root
        else preferred_report_run_root(
            corpus_root,
            PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.stage_id,
        )
    )
    local_results_root = run_root.parents[2]
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)

    spec = load_corpus_spec(repo_root)
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_read_length_run_manifest_contract(run_manifest)
    expected_sample_ids = [run["sample_id"] for run in run_manifest["runs"]]
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=expected_sample_ids,
        fallback_stage_id=PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.stage_id,
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
                "read-length benchmark report drift: "
                f"missing report.json for {sample_id}: {report_path}"
            )
        report = load_json(report_path)
        if not report.get("records"):
            raise SystemExit(
                "read-length benchmark report drift: "
                f"report.json for {sample_id} contains no records"
            )
        for record in report.get("records", []):
            tool = record.get("context", {}).get("tool", "unknown")
            artifact_paths = validate_artifact_paths(report_path, tool)
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
                "read_count": normalize_metric(record, "read_count") or 0,
                "mean_read_length": normalize_metric(record, "mean_read_length") or 0.0,
                "max_read_length": normalize_metric(record, "max_read_length") or 0,
                "distinct_lengths": normalize_metric(record, "distinct_lengths") or 0,
                "report_json_artifact": artifact_paths["report_json_artifact"],
                "length_distribution_tsv_artifact": artifact_paths[
                    "length_distribution_tsv_artifact"
                ],
                "length_distribution_json_artifact": artifact_paths[
                    "length_distribution_json_artifact"
                ],
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_read_length_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median([float(row["runtime_s"]) for row in rows]),
                "median_read_count": safe_median([float(row["read_count"]) for row in rows]),
                "median_mean_read_length": safe_median(
                    [float(row["mean_read_length"]) for row in rows]
                ),
                "median_max_read_length": safe_median(
                    [float(row["max_read_length"]) for row in rows]
                ),
                "median_distinct_lengths": safe_median(
                    [float(row["distinct_lengths"]) for row in rows]
                ),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        highest_max_read_length = max(
            (row for row in tool_summary if row["median_max_read_length"] is not None),
            key=lambda row: row["median_max_read_length"],
        )
        widest_length_support = max(
            (row for row in tool_summary if row["median_distinct_lengths"] is not None),
            key=lambda row: row["median_distinct_lengths"],
        )
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "highest_max_read_length_tool": highest_max_read_length["tool"],
            "highest_max_read_length": highest_max_read_length[
                "median_max_read_length"
            ],
            "widest_length_support_tool": widest_length_support["tool"],
            "widest_length_support": widest_length_support["median_distinct_lengths"],
        }

    summary = {
        "schema_version": "bijux.fastq.profile_read_lengths.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.stage_id,
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
        "report_only": run_manifest["report_only"],
        "mutates_fastq": run_manifest["mutates_fastq"],
        "may_change_read_count": run_manifest["may_change_read_count"],
        "raw_backend_report_format": run_manifest["raw_backend_report_format"],
        "histogram_bins": run_manifest["histogram_bins"],
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
                "read_count",
                "mean_read_length",
                "max_read_length",
                "distinct_lengths",
                "report_json_artifact",
                "length_distribution_tsv_artifact",
                "length_distribution_json_artifact",
            ],
        )
        writer.writeheader()
        for row in sample_rows:
            writer.writerow(row)

    (docs_root / "lunarc.md").write_text(
        render_markdown(summary),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
