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
    SCREEN_TAXONOMY_BENCHMARK_CONTRACT,
    load_corpus_spec,
    load_json,
    localize_results_path,
    preferred_report_run_root,
    resolve_corpus_metadata,
    screen_taxonomy_benchmark_defaults,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Render corpus-01 fastq.screen_taxonomy benchmark summary."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default=str(benchmark_remote_corpus_root()))
    parser.add_argument("--run-root", default="")
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark/fastq.screen_taxonomy/corpus-01",
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


def top_taxa(record: dict) -> list[dict]:
    payload = normalize_metric(record, "top_taxa")
    return payload if isinstance(payload, list) else []


def validate_run_manifest_contract(run_manifest: dict) -> None:
    defaults = screen_taxonomy_benchmark_defaults()
    if run_manifest.get("dry_run"):
        raise SystemExit(
            "screen-taxonomy benchmark report rendering requires an executed run, not --dry-run output"
        )
    if run_manifest.get("sample_limit") is not None:
        raise SystemExit(
            "screen-taxonomy benchmark report rendering requires the full corpus run, not --sample-limit output"
        )
    if run_manifest.get("stage_id") != SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id:
        raise SystemExit(
            "screen-taxonomy benchmark report drift: "
            f"expected stage_id {SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id}, "
            f"found {run_manifest.get('stage_id')}"
        )
    if run_manifest.get("scenario_id") != SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id:
        raise SystemExit(
            "screen-taxonomy benchmark report drift: "
            f"expected scenario_id {SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id}, "
            f"found {run_manifest.get('scenario_id')}"
        )
    if run_manifest.get("tool_kind", "benchmark") != "benchmark":
        raise SystemExit(
            "screen-taxonomy benchmark report drift: "
            f"expected tool_kind benchmark, found {run_manifest.get('tool_kind')}"
        )
    if not run_manifest.get("database_root"):
        raise SystemExit(
            "screen-taxonomy benchmark report drift: run manifest must record database_root"
        )
    if not run_manifest.get("database_digest"):
        raise SystemExit(
            "screen-taxonomy benchmark report drift: run manifest must record database_digest"
        )
    if not run_manifest.get("database_lineage_json"):
        raise SystemExit(
            "screen-taxonomy benchmark report drift: run manifest must record database_lineage_json"
        )
    if not run_manifest.get("database_lineage_digest"):
        raise SystemExit(
            "screen-taxonomy benchmark report drift: run manifest must record database_lineage_digest"
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
            "database_catalog_id",
            "database_artifact_id",
            "database_namespace",
            "database_scope",
        ]:
            if row[key] != run_manifest[key]:
                raise SystemExit(
                    "screen-taxonomy benchmark report drift: "
                    f"expected {key} {run_manifest[key]!r}, found {row[key]!r} "
                    f"for {row['sample_id']}/{row['tool']}"
                )
        if int(row["reads_out"]) > int(row["reads_in"]):
            raise SystemExit(
                "screen-taxonomy benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has reads_out > reads_in"
            )
        if int(row["bases_out"]) > int(row["bases_in"]):
            raise SystemExit(
                "screen-taxonomy benchmark report drift: "
                f"{row['sample_id']}/{row['tool']} has bases_out > bases_in"
            )

    for sample_id, rows in sorted(rows_by_sample.items()):
        observed_tools = [row["tool"] for row in rows]
        if sorted(observed_tools) != sorted(run_manifest["tools"]):
            raise SystemExit(
                "screen-taxonomy benchmark report drift: "
                f"sample {sample_id} expected tools {run_manifest['tools']}, found {observed_tools}"
            )
    missing_samples = sorted(set(expected_sample_ids) - set(rows_by_sample))
    if missing_samples:
        raise SystemExit(
            "screen-taxonomy benchmark report drift: "
            f"missing published rows for samples {missing_samples}"
        )


def render_markdown(summary: dict) -> str:
    lines: list[str] = []
    lines.append("# `fastq.screen_taxonomy` on `corpus-01`")
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
    lines.append(f"- database_root: `{summary['database_root']}`")
    lines.append(f"- database_digest: `{summary['database_digest']}`")
    lines.append(f"- database_lineage_json: `{summary['database_lineage_json']}`")
    lines.append(f"- database_lineage_digest: `{summary['database_lineage_digest']}`")
    lines.append(f"- database_catalog_id: `{summary['database_catalog_id']}`")
    lines.append(f"- database_artifact_id: `{summary['database_artifact_id']}`")
    lines.append(f"- database_namespace: `{summary['database_namespace']}`")
    lines.append(f"- database_scope: `{summary['database_scope']}`")
    lines.append("")
    lines.append("## Executive Summary")
    lines.append("")
    lines.append(
        f"- Fastest median runtime: `{summary['headline']['fastest_tool']}` at `{summary['headline']['fastest_runtime_s']:.3f}` seconds."
    )
    lines.append(
        f"- Lowest mean contamination rate: `{summary['headline']['lowest_contamination_tool']}` at `{summary['headline']['lowest_contamination_rate']:.3f}`."
    )
    lines.append(
        f"- Highest mean classified fraction: `{summary['headline']['highest_classified_tool']}` at `{summary['headline']['highest_classified_fraction']:.3f}`."
    )
    lines.append(
        f"- Sample failures: `{summary['samples_failed']}` sample invocations ended non-zero."
    )
    lines.append("")
    lines.append("## Tool Summary")
    lines.append("")
    lines.append(
        "| Tool | Samples | Pass rate | Median runtime (s) | Mean contamination rate | Mean classified fraction | Mean unclassified fraction | Most common top taxon |"
    )
    lines.append("| --- | ---: | ---: | ---: | ---: | ---: | ---: | --- |")
    for row in summary["tool_summary"]:
        lines.append(
            f"| `{row['tool']}` | {row['records']} | {row['pass_rate']:.1%} | {row['median_runtime_s']:.3f} | {row['mean_contamination_rate']:.3f} | {row['mean_classified_fraction']:.3f} | {row['mean_unclassified_fraction']:.3f} | `{row['most_common_top_taxon']}` |"
        )
    lines.append("")
    lines.append("## Notes")
    lines.append("")
    lines.append(
        "- `corpus-01` is human DNA, so the classifier outputs here are mainly a governed background-screening control, not a discovery cohort."
    )
    lines.append(
        "- The dossier records taxonomy database lineage directly in the run manifest so classifier comparisons remain interpretable when the database changes over time."
    )
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    corpus_root = Path(args.corpus_root).expanduser()
    run_root = (
        Path(args.run_root).expanduser().resolve()
        if args.run_root
        else preferred_report_run_root(corpus_root, SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id)
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
            taxa = top_taxa(record)
            top_taxon = taxa[0] if taxa else {}
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
                "reads_in": int(normalize_metric(record, "reads_in") or 0),
                "reads_out": int(normalize_metric(record, "reads_out") or 0),
                "bases_in": int(normalize_metric(record, "bases_in") or 0),
                "bases_out": int(normalize_metric(record, "bases_out") or 0),
                "pairs_in": int(normalize_metric(record, "pairs_in") or 0),
                "pairs_out": int(normalize_metric(record, "pairs_out") or 0),
                "contamination_rate": float(
                    normalize_metric(record, "contamination_rate") or 0.0
                ),
                "classified_fraction": float(
                    normalize_metric(record, "classified_fraction") or 0.0
                ),
                "unclassified_fraction": float(
                    normalize_metric(record, "unclassified_fraction") or 0.0
                ),
                "classifier": normalize_metric(record, "classifier") or tool,
                "report_format": normalize_metric(record, "report_format"),
                "database_catalog_id": normalize_metric(record, "database_catalog_id")
                or run_manifest["database_catalog_id"],
                "database_artifact_id": normalize_metric(record, "database_artifact_id")
                or run_manifest["database_artifact_id"],
                "database_namespace": run_manifest["database_namespace"],
                "database_scope": run_manifest["database_scope"],
                "minimum_confidence": normalize_metric(record, "minimum_confidence"),
                "emit_unclassified": normalize_metric(record, "emit_unclassified"),
                "top_taxon_label": top_taxon.get("label"),
                "top_taxon_percent": top_taxon.get("percent"),
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
        contamination_rates = [float(row["contamination_rate"]) for row in rows]
        classified = [float(row["classified_fraction"]) for row in rows]
        unclassified = [float(row["unclassified_fraction"]) for row in rows]
        top_taxa_labels = [row["top_taxon_label"] for row in rows if row["top_taxon_label"]]
        pass_count = sum(1 for row in rows if row["exit_code"] == 0)
        most_common_top_taxon = (
            max(set(top_taxa_labels), key=top_taxa_labels.count)
            if top_taxa_labels
            else "unclassified"
        )
        tool_summary.append(
            {
                "tool": tool,
                "records": len(rows),
                "pass_rate": safe_fraction(pass_count, len(rows)),
                "median_runtime_s": safe_median(runtimes),
                "mean_contamination_rate": safe_mean(contamination_rates),
                "mean_classified_fraction": safe_mean(classified),
                "mean_unclassified_fraction": safe_mean(unclassified),
                "most_common_top_taxon": most_common_top_taxon,
            }
        )

    fastest = min(tool_summary, key=lambda row: row["median_runtime_s"])
    lowest_contamination = min(
        tool_summary, key=lambda row: row["mean_contamination_rate"]
    )
    highest_classified = max(
        tool_summary, key=lambda row: row["mean_classified_fraction"]
    )
    summary = {
        "schema_version": "bijux.fastq.screen_taxonomy.corpus_summary.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id,
        "platform": run_manifest["platform"],
        "corpus_root": run_manifest["corpus_root"],
        "run_root": str(run_root),
        "tools": run_manifest["tools"],
        "samples_total": len(expected_sample_ids),
        "samples_failed": int(run_manifest.get("samples_failed", 0)),
        "cohort_counts": dict(sorted(cohort_counts.items())),
        "era_counts": dict(sorted(era_counts.items())),
        "layout_counts": dict(sorted(layout_counts.items())),
        "database_root": run_manifest["database_root"],
        "database_digest": run_manifest["database_digest"],
        "database_size_bytes": run_manifest["database_size_bytes"],
        "database_lineage_json": run_manifest["database_lineage_json"],
        "database_lineage_digest": run_manifest["database_lineage_digest"],
        "database_catalog_id": run_manifest["database_catalog_id"],
        "database_artifact_id": run_manifest["database_artifact_id"],
        "database_namespace": run_manifest["database_namespace"],
        "database_scope": run_manifest["database_scope"],
        "headline": {
            "fastest_tool": fastest["tool"],
            "fastest_runtime_s": fastest["median_runtime_s"],
            "lowest_contamination_tool": lowest_contamination["tool"],
            "lowest_contamination_rate": lowest_contamination[
                "mean_contamination_rate"
            ],
            "highest_classified_tool": highest_classified["tool"],
            "highest_classified_fraction": highest_classified[
                "mean_classified_fraction"
            ],
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
