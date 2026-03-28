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
    DETECT_ADAPTERS_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.detect_adapters benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument(
        "--run-root",
        default="",
        help="Benchmark run root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.detect_adapters/lunarc.",
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.detect_adapters/corpus-01",
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


def validate_detect_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "detect-adapters benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "detect-adapters benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != DETECT_ADAPTERS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "detect-adapters benchmark report drift: "
            f"expected stage_id {DETECT_ADAPTERS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != DETECT_ADAPTERS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "detect-adapters benchmark report drift: "
            f"expected scenario_id {DETECT_ADAPTERS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind") != "benchmark":
        raise SystemExit(
            "detect-adapters benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    expected_contract = {
        "inspection_mode": "evidence_only",
        "report_only": True,
        "evidence_scope": "full_input",
        "evidence_format": "fastqc_summary",
    }
    for key, expected in expected_contract.items():
        if run_manifest.get(key) != expected:
            raise SystemExit(
                "detect-adapters benchmark report drift: "
                f"expected {key}={expected!r}, found {run_manifest.get(key)!r}"
            )


def validate_detect_row_contract(
    *,
    run_manifest: dict,
    sample_rows: list[dict],
    expected_sample_ids: list[str],
) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        if row["reads_in"] != row["reads_out"]:
            raise SystemExit(
                "detect-adapters benchmark report drift: "
                f"reads_out must equal reads_in for {row['sample_id']}/{row['tool']}"
            )
        if row["bases_in"] != row["bases_out"]:
            raise SystemExit(
                "detect-adapters benchmark report drift: "
                f"bases_out must equal bases_in for {row['sample_id']}/{row['tool']}"
            )
        fraction = row["adapter_trimmed_fraction"]
        if fraction is not None and not 0.0 <= fraction <= 1.0:
            raise SystemExit(
                "detect-adapters benchmark report drift: "
                f"adapter_trimmed_fraction must be within [0, 1] for {row['sample_id']}/{row['tool']}"
            )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "detect-adapters benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "detect-adapters benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.detect_adapters` on `corpus-01`")
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
    lines.append(f"- inspection_mode: `{summary['inspection_mode']}`")
    lines.append(f"- evidence_scope: `{summary['evidence_scope']}`")
    lines.append(f"- evidence_format: `{summary['evidence_format']}`")
    lines.append(f"- report_only: `{summary['report_only']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    headline = summary.get("headline", {})
    if headline.get("fastest_tool"):
        lines.append(
            f"- Fastest median runtime: `{headline['fastest_tool']}` at `{headline['fastest_runtime_s']:.3f}` seconds."
        )
    if headline.get("largest_adapter_signal_tool"):
        lines.append(
            f"- Highest mean candidate-adapter count: `{headline['largest_adapter_signal_tool']}` at `{headline['largest_adapter_signal']:.3f}` candidates per sample."
        )
    if headline.get("highest_trimmed_fraction_tool"):
        lines.append(
            f"- Highest mean adapter-trimmed fraction: `{headline['highest_trimmed_fraction_tool']}` at `{headline['highest_trimmed_fraction']:.3f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Mean candidates | Mean adapter-trimmed fraction | Median mean Q |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        pass_rate = "n/a" if row["pass_rate"] is None else f"{row['pass_rate']:.1%}"
        runtime = "n/a" if row["median_runtime_s"] is None else f"{row['median_runtime_s']:.3f}"
        mean_candidates = (
            "n/a"
            if row["mean_candidate_adapter_count"] is None
            else f"{row['mean_candidate_adapter_count']:.3f}"
        )
        trimmed_fraction = (
            "n/a"
            if row["mean_adapter_trimmed_fraction"] is None
            else f"{row['mean_adapter_trimmed_fraction']:.3f}"
        )
        mean_q = (
            "n/a" if row["median_mean_q"] is None else f"{row['median_mean_q']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {mean_candidates} | {trimmed_fraction} | {mean_q} |"
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
        "- This stage is evidence-only: governed benchmarking confirms adapter-inspection throughput and signal stability without mutating reads."
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
            DETECT_ADAPTERS_BENCHMARK_CONTRACT.stage_id,
        )
    )
    local_results_root = run_root.parents[2]
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)

    spec = load_corpus_spec(repo_root)
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_detect_run_manifest_contract(run_manifest)
    expected_sample_ids = [run["sample_id"] for run in run_manifest["runs"]]
    metadata_by_sample = resolve_corpus_metadata(
        repo_root,
        corpus_root,
        spec,
        expected_sample_ids=expected_sample_ids,
        fallback_stage_id=DETECT_ADAPTERS_BENCHMARK_CONTRACT.stage_id,
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
                "detect-adapters benchmark report drift: "
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
                "mean_q": normalize_metric(record, "mean_q") or 0.0,
                "candidate_adapter_count": normalize_metric(record, "candidate_adapter_count")
                or 0,
                "adapter_trimmed_fraction": normalize_metric(
                    record, "adapter_trimmed_fraction"
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_detect_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
        expected_sample_ids=expected_sample_ids,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        runtimes = [float(row["runtime_s"]) for row in rows]
        mean_q_values = [float(row["mean_q"]) for row in rows]
        candidate_counts = [float(row["candidate_adapter_count"]) for row in rows]
        trimmed_fractions = [
            float(row["adapter_trimmed_fraction"])
            for row in rows
            if row["adapter_trimmed_fraction"] is not None
        ]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median(runtimes),
                "median_mean_q": safe_median(mean_q_values),
                "mean_candidate_adapter_count": safe_mean(candidate_counts),
                "mean_adapter_trimmed_fraction": safe_mean(trimmed_fractions),
            }
        )

    headline = {}
    if tool_summary:
        fastest = min(
            (row for row in tool_summary if row["median_runtime_s"] is not None),
            key=lambda row: row["median_runtime_s"],
        )
        largest_adapter_signal = max(
            (
                row
                for row in tool_summary
                if row["mean_candidate_adapter_count"] is not None
            ),
            key=lambda row: row["mean_candidate_adapter_count"],
        )
        trimmed_fraction_rows = [
            row
            for row in tool_summary
            if row["mean_adapter_trimmed_fraction"] is not None
        ]
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "largest_adapter_signal_tool": largest_adapter_signal["tool"],
            "largest_adapter_signal": largest_adapter_signal[
                "mean_candidate_adapter_count"
            ],
            "highest_trimmed_fraction_tool": (
                max(
                    trimmed_fraction_rows,
                    key=lambda row: row["mean_adapter_trimmed_fraction"],
                )["tool"]
                if trimmed_fraction_rows
                else None
            ),
            "highest_trimmed_fraction": (
                max(
                    trimmed_fraction_rows,
                    key=lambda row: row["mean_adapter_trimmed_fraction"],
                )["mean_adapter_trimmed_fraction"]
                if trimmed_fraction_rows
                else None
            ),
        }

    summary = {
        "schema_version": "bijux.fastq.detect_adapters.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": DETECT_ADAPTERS_BENCHMARK_CONTRACT.stage_id,
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
        "inspection_mode": run_manifest["inspection_mode"],
        "report_only": run_manifest["report_only"],
        "evidence_scope": run_manifest["evidence_scope"],
        "evidence_format": run_manifest["evidence_format"],
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
                "mean_q",
                "candidate_adapter_count",
                "adapter_trimmed_fraction",
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
