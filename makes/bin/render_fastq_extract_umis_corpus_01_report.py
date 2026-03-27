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
    EXTRACT_UMIS_BENCHMARK_CONTRACT,
    extract_umis_benchmark_defaults,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.extract_umis benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="/home/bijan/bijux/corpus_01")
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.extract_umis/corpus-01",
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
    context = record.get("context", {})
    parameters = context.get("parameters", {})
    if not isinstance(parameters, dict):
        return None
    return parameters.get(key)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = extract_umis_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "extract-umis benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "extract-umis benchmark report rendering requires the full paired corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "extract-umis benchmark report drift: "
            f"expected stage_id {EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != EXTRACT_UMIS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "extract-umis benchmark report drift: "
            f"expected scenario_id {EXTRACT_UMIS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "extract-umis benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    if run_manifest.get("sample_scope") != "paired":
        raise SystemExit(
            "extract-umis benchmark report drift: run manifest must record paired sample_scope"
        )
    run_manifest.setdefault("umi_pattern", defaults["umi_pattern"])
    run_manifest.setdefault(
        "allow_missing_umi_headers", defaults["allow_missing_umi_headers"]
    )


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
                "extract-umis benchmark report drift: "
                f"sample {row['sample_id']} is not paired-end"
            )
        if row["paired_mode"] != "paired_end":
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"expected paired_end mode for {row['sample_id']}/{row['tool']}, "
                f"found {row['paired_mode']}"
            )
        if row["umi_pattern"] != run_manifest["umi_pattern"]:
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"expected umi_pattern {run_manifest['umi_pattern']!r}, "
                f"found {row['umi_pattern']!r} for {row['sample_id']}/{row['tool']}"
            )
        if row["raw_backend_report_format"] != "umi_tools_log":
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format 'umi_tools_log', "
                f"found {row['raw_backend_report_format']!r}"
            )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["reads_with_umi"]) > int(row["reads_out"]):
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_with_umi > reads_out"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "extract-umis benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "extract-umis benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.extract_umis` on `corpus-01`")
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
    lines.append(f"- Tools: `{', '.join(summary['tools'])}`")
    lines.append(f"- umi_pattern: `{summary['umi_pattern']}`")
    lines.append(
        f"- allow_missing_umi_headers: `{summary['allow_missing_umi_headers']}`"
    )
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Mean read retention: `{summary['headline']['best_retention_tool']}` at `{summary['headline']['best_retention']:.3f}`."
    )
    lines.append(
        f"- Mean reads with detected UMI: `{summary['headline']['best_umi_detection_tool']}` at `{summary['headline']['best_umi_detection']:.1f}` reads."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Mean reads with UMI | Mean reads with UMI fraction |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_read_retention']:.3f} | {row['mean_reads_with_umi']:.1f} | {row['mean_reads_with_umi_fraction']:.3f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- This paired-only benchmark keeps the governed UMI parsing contract explicit, so later pattern changes cannot silently invalidate comparisons."
    )
    lines.append(
        "- `corpus-01` is not a native UMI cohort, so the dossier records whether missing-header bypass was enabled during execution."
    )
    lines.append(
        "- Published per-sample rows keep read-retention and UMI-detection behavior auditable alongside runtime."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser().resolve()
        if args.run_root
        else preferred_report_run_root(corpus_root, EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id)
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

    for run in run_manifest["runs"]:
        sample_id = run["sample_id"]
        metadata = metadata_by_sample[sample_id]
        era_counts[metadata["era"]] += 1

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
            reads_with_umi = int(normalize_metric(record, "reads_with_umi") or 0)
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
                "paired_mode": normalize_metric(record, "paired_mode")
                or normalize_parameter(record, "paired_mode")
                or ("paired_end" if metadata["layout"] == "pe" else None),
                "umi_pattern": normalize_metric(record, "umi_pattern")
                or normalize_parameter(record, "umi_pattern")
                or run_manifest["umi_pattern"],
                "reads_in": reads_in,
                "reads_out": reads_out,
                "bases_in": int(normalize_metric(record, "bases_in") or 0),
                "bases_out": int(normalize_metric(record, "bases_out") or 0),
                "pairs_in": int(normalize_metric(record, "pairs_in") or 0),
                "pairs_out": int(normalize_metric(record, "pairs_out") or 0),
                "reads_with_umi": reads_with_umi,
                "read_retention": safe_fraction(reads_out, reads_in),
                "reads_with_umi_fraction": safe_fraction(reads_with_umi, reads_in),
                "mean_q_before": float(normalize_metric(record, "mean_q_before") or 0.0),
                "mean_q_after": float(normalize_metric(record, "mean_q_after") or 0.0),
                "raw_backend_report_format": normalize_metric(
                    record, "raw_backend_report_format"
                )
                or normalize_parameter(record, "raw_backend_report_format"),
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
        read_retention = [float(row["read_retention"] or 0.0) for row in rows]
        reads_with_umi = [float(row["reads_with_umi"]) for row in rows]
        reads_with_umi_fraction = [
            float(row["reads_with_umi_fraction"] or 0.0) for row in rows
        ]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_read_retention": safe_median(read_retention),
                "mean_reads_with_umi": safe_mean(reads_with_umi),
                "mean_reads_with_umi_fraction": safe_mean(reads_with_umi_fraction),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    best_retention = max(tool_summary, key=lambda row: row["median_read_retention"])
    best_umi_detection = max(
        tool_summary, key=lambda row: row["mean_reads_with_umi"]
    )
    summary = {
        "schema_version": "bijux.fastq.extract_umis.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": EXTRACT_UMIS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": run_manifest["corpus_root"],
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": int(run_manifest.get("samples_failed", 0)),
        "era_counts": dict(sorted(era_counts.items())),
        "umi_pattern": run_manifest["umi_pattern"],
        "allow_missing_umi_headers": run_manifest["allow_missing_umi_headers"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "best_retention_tool": best_retention["tool"],
            "best_retention": best_retention["median_read_retention"],
            "best_umi_detection_tool": best_umi_detection["tool"],
            "best_umi_detection": best_umi_detection["mean_reads_with_umi"],
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
