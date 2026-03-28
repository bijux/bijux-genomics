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
    NORMALIZE_PRIMERS_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    normalize_primers_benchmark_defaults,
    preferred_report_run_root,
    resolve_corpus_metadata,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.normalize_primers benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.normalize_primers/corpus-01",
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


def expected_raw_backend_report_format(tool: str) -> str | None:
    return {"cutadapt": "cutadapt_json"}.get(tool)


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = normalize_primers_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "normalize-primers benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "normalize-primers benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "normalize-primers benchmark report drift: "
            f"expected stage_id {NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "normalize-primers benchmark report drift: "
            f"expected scenario_id {NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "normalize-primers benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
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
        for key in [
            "primer_set_id",
            "orientation_policy",
            "max_mismatch_rate",
            "min_overlap_bp",
            "strict_5p_anchor",
            "allow_iupac_codes",
        ]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "normalize-primers benchmark report drift: "
                    f"expected {key}={run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
        expected_format = expected_raw_backend_report_format(row["tool"])
        if row["raw_backend_report_format"] != expected_format:
            raise SystemExit(
                "normalize-primers benchmark report drift: "
                f"tool {row['tool']} expected raw_backend_report_format "
                f"{expected_format}, found {row['raw_backend_report_format']}"
            )
        if row["reads_out"] > row["reads_in"]:
            raise SystemExit(
                "normalize-primers benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "normalize-primers benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "normalize-primers benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.normalize_primers` on `corpus-01`")
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
    lines.append(f"- Tool roster: `{', '.join(summary['tools'])}`")
    lines.append(f"- primer_set_id: `{summary['primer_set_id']}`")
    lines.append(f"- orientation_policy: `{summary['orientation_policy']}`")
    lines.append(f"- max_mismatch_rate: `{summary['max_mismatch_rate']}`")
    lines.append(f"- min_overlap_bp: `{summary['min_overlap_bp']}`")
    lines.append(f"- strict_5p_anchor: `{summary['strict_5p_anchor']}`")
    lines.append(f"- allow_iupac_codes: `{summary['allow_iupac_codes']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Highest mean primer-trimmed fraction: `{summary['headline']['strongest_trim_tool']}` at `{summary['headline']['strongest_trim_fraction']:.3f}`."
    )
    lines.append(
        f"- Highest median forward-orientation fraction: `{summary['headline']['most_forward_tool']}` at `{summary['headline']['most_forward_fraction']:.3f}`."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Median read retention | Mean primer-trimmed fraction | Median forward-orientation fraction |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['median_read_retention']:.3f} | {row['mean_primer_trimmed_fraction']:.3f} | {row['median_orientation_forward_fraction']:.3f} |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- `corpus-01` is a human DNA cohort, so this run functions as both a throughput benchmark and a governed false-positive control for primer-aware normalization."
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
            corpus_root, NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id
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
            row = {
                "sample_id": sample_id,
                "accession": metadata["accession"],
                "era": metadata["era"],
                "layout": metadata["layout"],
                "study_accession": metadata["study_accession"],
                "size_band": metadata["size_band"],
                "tool": tool,
                "runtime_s": record.get("execution", {}).get("runtime_s"),
                "exit_code": record.get("execution", {}).get("exit_code"),
                "reads_in": reads_in,
                "reads_out": reads_out,
                "read_retention": (reads_out / reads_in) if reads_in else 0.0,
                "primer_trimmed_fraction": float(
                    normalize_metric(record, "primer_trimmed_fraction") or 0.0
                ),
                "orientation_forward_fraction": float(
                    normalize_metric(record, "orientation_forward_fraction") or 0.0
                ),
                "primer_set_id": normalize_parameter(record, "primer_set_id"),
                "marker_id": normalize_parameter(record, "marker_id"),
                "primer_fasta": normalize_parameter(record, "primer_fasta"),
                "orientation_policy": normalize_parameter(record, "orientation_policy"),
                "max_mismatch_rate": normalize_parameter(record, "max_mismatch_rate"),
                "min_overlap_bp": normalize_parameter(record, "min_overlap_bp"),
                "strict_5p_anchor": normalize_parameter(record, "strict_5p_anchor"),
                "allow_iupac_codes": normalize_parameter(record, "allow_iupac_codes"),
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
        primer_trimmed_fractions = [
            float(row["primer_trimmed_fraction"]) for row in rows
        ]
        orientation_forward_fractions = [
            float(row["orientation_forward_fraction"]) for row in rows
        ]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "median_read_retention": safe_median(read_retentions),
                "mean_primer_trimmed_fraction": safe_mean(primer_trimmed_fractions),
                "median_orientation_forward_fraction": safe_median(
                    orientation_forward_fractions
                ),
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    strongest_trim = max(
        tool_summary,
        key=lambda row: row["mean_primer_trimmed_fraction"],
    )
    most_forward = max(
        tool_summary,
        key=lambda row: row["median_orientation_forward_fraction"],
    )
    summary = {
        "schema_version": "bijux.fastq.normalize_primers.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": str(corpus_root),
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": int(run_manifest.get("failures", 0)),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "primer_set_id": run_manifest["primer_set_id"],
        "orientation_policy": run_manifest["orientation_policy"],
        "max_mismatch_rate": run_manifest["max_mismatch_rate"],
        "min_overlap_bp": run_manifest["min_overlap_bp"],
        "strict_5p_anchor": run_manifest["strict_5p_anchor"],
        "allow_iupac_codes": run_manifest["allow_iupac_codes"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "strongest_trim_tool": strongest_trim["tool"],
            "strongest_trim_fraction": strongest_trim["mean_primer_trimmed_fraction"],
            "most_forward_tool": most_forward["tool"],
            "most_forward_fraction": most_forward["median_orientation_forward_fraction"],
        },
        "tool_summary": tool_summary,
    }

    (docs_root / "summary.json").write_text(
        json.dumps(summary, indent=2) + "\n",
        encoding="utf-8",
    )
    with (docs_root / "sample_results.csv").open(
        "w", encoding="utf-8", newline=""
    ) as handle:
        writer = csv.DictWriter(handle, fieldnames=list(sample_rows[0].keys()))
        writer.writeheader()
        writer.writerows(sample_rows)
    (docs_root / "lunarc.md").write_text(
        render_markdown(summary),
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
