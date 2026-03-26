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
    TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    load_json,
    load_published_sample_metadata,
    validate_corpus_contract,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.trim_terminal_damage benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="/home/bijan/bijux/corpus_01")
    parser.add_argument(
        "--run-root",
        default="",
        help="Benchmark run root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.trim_terminal_damage/lunarc.",
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.trim_terminal_damage/corpus-01",
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


def localize_results_path(path_str: str, local_results_root: Path) -> Path:
    path = Path(path_str)
    if path.exists():
        return path
    marker = "/results/"
    if marker not in path_str:
        return path
    return local_results_root / path_str.split(marker, 1)[1]


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
    return {"cutadapt": "cutadapt_json"}.get(tool)


def validate_terminal_damage_run_manifest_contract(run_manifest: dict) -> None:
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "terminal-damage benchmark report rendering requires an executed run, not --dry-run output"
        )
    stage_id = run_manifest.get("stage_id", TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.stage_id)
    scenario_id = run_manifest.get(
        "scenario_id", TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.scenario_id
    )
    tool_kind = run_manifest.get("tool_kind", "benchmark")
    if stage_id != TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "terminal-damage benchmark report drift: "
            f"expected stage_id {TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.stage_id}, found {stage_id}"
        )
    if scenario_id != TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "terminal-damage benchmark report drift: "
            f"expected scenario_id {TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.scenario_id}, found {scenario_id}"
        )
    if tool_kind != "benchmark":
        raise SystemExit(
            "terminal-damage benchmark report drift: "
            f"expected tool_kind benchmark, found {tool_kind}"
        )


def validate_terminal_damage_row_contract(*, run_manifest: dict, sample_rows: list[dict]) -> None:
    expected_tools = run_manifest["tools"]
    rows_by_sample: dict[str, list[dict]] = defaultdict(list)
    for row in sample_rows:
        rows_by_sample[row["sample_id"]].append(row)
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "terminal-damage benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )
        for key in [
            "damage_mode",
            "execution_policy",
            "trim_5p_bases",
            "trim_3p_bases",
            "requested_trim_5p_bases",
            "requested_trim_3p_bases",
        ]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "terminal-damage benchmark report drift: "
                    f"expected {key}={run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(expected_tools):
            raise SystemExit(
                "terminal-damage benchmark report drift: "
                f"sample {sample_id} expected tools {expected_tools}, found {observed_tools}"
            )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.trim_terminal_damage` on `corpus-01`")
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
    lines.append(f"- damage_mode: `{summary['damage_mode']}`")
    lines.append(f"- execution_policy: `{summary['execution_policy']}`")
    lines.append(f"- trim_5p_bases: `{summary['trim_5p_bases']}`")
    lines.append(f"- trim_3p_bases: `{summary['trim_3p_bases']}`")
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
    if headline.get("largest_asymmetry_reduction_tool"):
        lines.append(
            f"- Largest mean terminal asymmetry reduction: `{headline['largest_asymmetry_reduction_tool']}` at `{headline['largest_asymmetry_reduction']:.3f}`."
        )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median base retention | Mean asymmetry reduction | Mean Q delta |"
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
        asymmetry_reduction = (
            "n/a"
            if row["mean_asymmetry_reduction"] is None
            else f"{row['mean_asymmetry_reduction']:.3f}"
        )
        mean_q_delta = (
            "n/a" if row["mean_q_delta"] is None else f"{row['mean_q_delta']:.3f}"
        )
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {pass_rate} | {runtime} | {base_retention} | {asymmetry_reduction} | {mean_q_delta} |"
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
        "- This benchmark pins the governed ancient-DNA terminal-trim policy across the full corpus so modern samples act as negative-control inputs for damage-aware trimming behavior."
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
    corpus_root = Path(args.corpus_root).resolve()
    run_root = (
        Path(args.run_root).resolve()
        if args.run_root
        else default_results_stage_root(
            corpus_root, TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.stage_id
        )
    )
    docs_root = (repo_root / args.docs_root).resolve()
    docs_root.mkdir(parents=True, exist_ok=True)
    local_results_root = run_root.parents[2]

    spec = load_corpus_spec(repo_root)
    run_manifest = load_json(run_root / "run_manifest.json")
    validate_terminal_damage_run_manifest_contract(run_manifest)
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
            pre = normalize_metric(record, "ct_ga_asymmetry_pre")
            post = normalize_metric(record, "ct_ga_asymmetry_post")
            asymmetry_reduction = None
            if pre is not None and post is not None:
                asymmetry_reduction = float(pre) - float(post)
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
                "damage_mode": metric_or_run_default(record, "damage_mode", run_manifest),
                "execution_policy": metric_or_run_default(
                    record, "execution_policy", run_manifest
                ),
                "trim_5p_bases": metric_or_run_default(
                    record, "trim_5p_bases", run_manifest
                ),
                "trim_3p_bases": metric_or_run_default(
                    record, "trim_3p_bases", run_manifest
                ),
                "requested_trim_5p_bases": metric_or_run_default(
                    record, "requested_trim_5p_bases", run_manifest
                ),
                "requested_trim_3p_bases": metric_or_run_default(
                    record, "requested_trim_3p_bases", run_manifest
                ),
                "udg_classification": normalize_metric(record, "udg_classification"),
                "ct_ga_asymmetry_pre": pre,
                "ct_ga_asymmetry_post": post,
                "asymmetry_reduction": asymmetry_reduction,
                "raw_backend_report_format": (
                    normalize_metric(record, "raw_backend_report_format")
                    or expected_raw_backend_report_format(tool)
                ),
            }
            sample_rows.append(row)
            tool_rows[tool].append(row)

    validate_terminal_damage_row_contract(
        run_manifest=run_manifest,
        sample_rows=sample_rows,
    )

    tool_summary = []
    for tool in sorted(tool_rows):
        rows = tool_rows[tool]
        runtimes = [float(row["runtime_s"]) for row in rows]
        base_retentions = [float(row["base_retention"]) for row in rows]
        mean_q_deltas = [float(row["mean_q_delta"]) for row in rows]
        asymmetry_reductions = [
            float(row["asymmetry_reduction"])
            for row in rows
            if row["asymmetry_reduction"] is not None
        ]
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": sum(1 for row in rows if row["exit_code"] == 0) / len(rows)
                if rows
                else None,
                "median_runtime_s": safe_median(runtimes),
                "median_base_retention": safe_median(base_retentions),
                "mean_asymmetry_reduction": safe_mean(asymmetry_reductions),
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
        headline = {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_base_retention_tool": best_base_retention["tool"],
            "best_base_retention": best_base_retention["median_base_retention"],
        }
        asymmetry_rows = [
            row for row in tool_summary if row["mean_asymmetry_reduction"] is not None
        ]
        if asymmetry_rows:
            largest_asymmetry_reduction = max(
                asymmetry_rows,
                key=lambda row: row["mean_asymmetry_reduction"],
            )
            headline["largest_asymmetry_reduction_tool"] = largest_asymmetry_reduction["tool"]
            headline["largest_asymmetry_reduction"] = largest_asymmetry_reduction[
                "mean_asymmetry_reduction"
            ]

    summary = {
        "schema_version": "bijux.fastq.trim_terminal_damage.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": TRIM_TERMINAL_DAMAGE_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": run_manifest["scenario_id"],
        "platform": run_manifest["platform"],
        "corpus_root": run_manifest.get("corpus_root", str(corpus_root)),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": run_manifest["samples_total"],
        "samples_failed": run_manifest["samples_failed"],
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "damage_mode": run_manifest["damage_mode"],
        "execution_policy": run_manifest["execution_policy"],
        "trim_5p_bases": run_manifest["trim_5p_bases"],
        "trim_3p_bases": run_manifest["trim_3p_bases"],
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
                "base_retention",
                "read_retention",
                "mean_q_delta",
                "damage_mode",
                "execution_policy",
                "trim_5p_bases",
                "trim_3p_bases",
                "requested_trim_5p_bases",
                "requested_trim_3p_bases",
                "udg_classification",
                "ct_ga_asymmetry_pre",
                "ct_ga_asymmetry_post",
                "asymmetry_reduction",
                "raw_backend_report_format",
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
