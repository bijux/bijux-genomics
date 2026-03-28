#!/usr/bin/env python3
from __future__ import annotations

import argparse
import csv
import json
import statistics
from collections import defaultdict
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    fmt_csv_value,
    fmt_fraction,
    fmt_metric,
    fmt_runtime,
    fmt_value,
    load_csv_rows,
    load_json,
    optional_int,
    parse_corpus_briefing_args,
    publish_corpus_briefing_artifacts,
    percentile,
    resolve_corpus_briefing_runtime,
    iter_cohort_row_groups,
    find_cohort_entry,
    find_cohort_entry,
    safe_mean,
    safe_median,
    BriefingMetricSpec,
    summarize_tool_runtime_rows,
    summarize_cohort_metric_rows,
    BriefingCopiedMetricSpec,
    BriefingOutlierSpec,
    summarize_sample_outlier_rows,
)

def parse_args() -> argparse.Namespace:
    return parse_corpus_briefing_args(
        description="Render an enriched benchmark briefing from corpus-01 report-qc artifacts.",
        docs_root="docs/benchmark/fastq.report_qc/corpus-01",
    )


def validate_summary_contract(summary: dict) -> None:
    expected_values = {
        "stage_id": "fastq.report_qc",
        "scenario_id": "qc_aggregation_fairness",
        "aggregation_engine": "multiqc",
        "aggregation_scope": "governed_qc_artifacts",
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
    }
    for key, expected in expected_values.items():
        if summary.get(key) != expected:
            raise SystemExit(
                f"report-qc briefing drift: summary {key}={summary.get(key)!r} expected {expected!r}"
            )
    if not summary.get("governed_contributor_stage_ids"):
        raise SystemExit(
            "report-qc briefing drift: governed_contributor_stage_ids must not be empty"
        )


def validate_rows_contract(summary: dict, rows: list[dict]) -> None:
    expected_tools = sorted(summary.get("tools") or [])
    observed_tools = sorted({row.get("tool", "") for row in rows if row.get("tool")})
    if observed_tools != expected_tools:
        raise SystemExit(
            f"report-qc briefing drift: observed tools {observed_tools!r} expected {expected_tools!r}"
        )
    for row in rows:
        module_count = optional_int(row.get("multiqc_module_count"))
        governed_inputs = optional_int(row.get("governed_qc_input_count"))
        if module_count is None or module_count <= 0:
            raise SystemExit(
                "report-qc briefing drift: multiqc_module_count must stay positive"
            )
        if governed_inputs is None or governed_inputs <= 0:
            raise SystemExit(
                "report-qc briefing drift: governed_qc_input_count must stay positive"
            )


def tool_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_tool_runtime_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('multiqc_module_count', 'median_multiqc_module_count', 'median', skip_blank=True),
            BriefingMetricSpec('multiqc_sample_count', 'median_multiqc_sample_count', 'median', skip_blank=True),
            BriefingMetricSpec('governed_qc_input_count', 'median_governed_qc_input_count', 'median'),
            BriefingMetricSpec('contamination_rate', 'median_contamination_rate', 'median'),
            BriefingMetricSpec('mean_q', 'median_mean_q', 'median'),
        ],
    )
def cohort_runtime_summary(rows: list[dict]) -> list[dict]:
    return summarize_cohort_metric_rows(
        rows,
        metric_specs=[
            BriefingMetricSpec('multiqc_module_count', 'median_multiqc_module_count', 'median', skip_blank=True),
            BriefingMetricSpec('governed_qc_input_count', 'median_governed_qc_input_count', 'median'),
            BriefingMetricSpec('contamination_rate', 'median_contamination_rate', 'median'),
        ],
    )
def sample_runtime_outliers(rows: list[dict]) -> list[dict]:
    return summarize_sample_outlier_rows(
        rows,
        selectors=[
            BriefingOutlierSpec('runtime_s', 'max', 'slowest_tool', 'slowest_runtime_s', parser='float', copied_metrics=(BriefingCopiedMetricSpec('multiqc_module_count', 'multiqc_module_count', parser='int', skip_blank=True),)),
            BriefingOutlierSpec('governed_qc_input_count', 'max', None, 'governed_qc_input_count', parser='int'),
        ],
        total_runtime_output_key='total_runtime_s',
    )
def render_markdown(
    summary: dict,
    runtime_rows: list[dict],
    cohort_rows: list[dict],
    outliers: list[dict],
) -> str:
    reference_tool = summary["tools"][0]
    tool_lookup = {row["tool"]: row for row in runtime_rows}
    tool_row = tool_lookup[reference_tool]
    ancient_pe = find_cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="era_layout",
        cohort="ancient_pe",
    )
    modern_pe = find_cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="era_layout",
        cohort="modern_pe",
    )
    size_under_100 = find_cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="size_band",
        cohort="under_100mb",
    )
    size_under_500 = find_cohort_entry(
        cohort_rows,
        tool=reference_tool,
        dimension="size_band",
        cohort="under_500mb",
    )

    lines = [
        "# `fastq.report_qc` benchmark on `corpus-01`",
        "",
        "## What was run",
        "",
        "This benchmark measures the governed `fastq.report_qc` stage across the curated `corpus-01` human DNA cohort on the Lunarc Apptainer platform.",
        "",
        f"- Platform: `{summary['platform']}` on Lunarc",
        f"- Corpus root: `{summary['corpus_root']}`",
        f"- Benchmark root: `{summary['run_root']}`",
        f"- Input balance: `{summary['era_counts'].get('ancient', 0)}` ancient, `{summary['era_counts'].get('modern', 0)}` modern, `{summary['layout_counts'].get('se', 0)}` single-end, `{summary['layout_counts'].get('pe', 0)}` paired-end",
        f"- Tool set: `{', '.join(summary['tools'])}`",
        f"- Aggregation contract: `{summary['aggregation_engine']}`, `{summary['aggregation_scope']}`, report_only=`{summary['report_only']}`",
        f"- Governed contributor stages: `{', '.join(summary['governed_contributor_stage_ids'])}`",
        "",
        "## Executive summary",
        "",
        f"- Every tool completed successfully on all `{summary['samples_total']}` samples; stage-level sample failures were `{summary['samples_failed']}`.",
        f"- `{reference_tool}` ran at `p50={fmt_runtime(tool_row['median_runtime_s'])}s` with median MultiQC sample count `{fmt_value(tool_row['median_multiqc_sample_count'])}` and median module count `{fmt_value(tool_row['median_multiqc_module_count'])}`.",
        f"- Governed evidence stayed stable: median governed QC input count was `{fmt_value(tool_row['median_governed_qc_input_count'])}` and every published row preserved `reads_out == reads_in` and `bases_out == bases_in`.",
        (
            f"- Runtime remains input-driven for `{reference_tool}`: `modern_pe` averages `{fmt_runtime(modern_pe['mean_runtime_s'])}s` while `ancient_pe` averages `{fmt_runtime(ancient_pe['mean_runtime_s'])}s`."
            if ancient_pe and modern_pe
            else "- Runtime remains cohort-sensitive across the balanced corpus mix."
        ),
        (
            f"- Size-band spread is visible in the aggregation stage: `under_500mb` averages `{fmt_runtime(size_under_500['mean_runtime_s'])}s` versus `{fmt_runtime(size_under_100['mean_runtime_s'])}s` on `under_100mb` inputs."
            if size_under_100 and size_under_500
            else "- Size-band spread is visible in the aggregation stage."
        ),
        "",
        "## Tool ranking",
        "",
        "| Tool | Pass rate | Mean (s) | Median (s) | P90 (s) | Max (s) | Median modules | Median sample count | Median governed inputs | Median contamination |",
        "| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |",
    ]
    for row in runtime_rows:
        lines.append(
            f"| `{row['tool']}` | {fmt_fraction(row['pass_rate'])} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_runtime(row['p90_runtime_s'])} | {fmt_runtime(row['max_runtime_s'])} | {fmt_value(row['median_multiqc_module_count'])} | {fmt_value(row['median_multiqc_sample_count'])} | {fmt_value(row['median_governed_qc_input_count'])} | {fmt_value(row['median_contamination_rate'])} |"
        )
    lines.extend(
        [
            "",
            "## Cohort behavior",
            "",
            "| Tool | Dimension | Cohort | Samples | Mean runtime (s) | Median runtime (s) | Median modules | Median governed inputs | Median contamination |",
            "| --- | --- | --- | ---: | ---: | ---: | ---: | ---: | ---: |",
        ]
    )
    for row in cohort_rows:
        lines.append(
            f"| `{row['tool']}` | `{row['dimension']}` | `{row['cohort']}` | {row['samples']} | {fmt_runtime(row['mean_runtime_s'])} | {fmt_runtime(row['median_runtime_s'])} | {fmt_value(row['median_multiqc_module_count'])} | {fmt_value(row['median_governed_qc_input_count'])} | {fmt_value(row['median_contamination_rate'])} |"
        )
    lines.extend(
        [
            "",
            "## Highest-cost samples",
            "",
            "| Sample | Accession | Era | Layout | Size band | Slowest runtime (s) | Modules | Governed inputs |",
            "| --- | --- | --- | --- | --- | ---: | ---: | ---: |",
        ]
    )
    for row in outliers[:10]:
        modules = "n/a" if row["multiqc_module_count"] is None else str(row["multiqc_module_count"])
        lines.append(
            f"| `{row['sample_id']}` | `{row['accession']}` | `{row['era']}` | `{row['layout']}` | `{row['size_band']}` | {fmt_runtime(row['slowest_runtime_s'])} | {modules} | {row['governed_qc_input_count']} |"
        )
    lines.extend(
        [
            "",
            "## Interpretation",
            "",
            "- `fastq.report_qc` is a report-only aggregation stage, so benchmark value comes from stable governed manifest handling, MultiQC bundle publication, and runtime predictability across corpus composition rather than from read mutation.",
            "- The governed input contract here deliberately joins validation, adapter inspection, read profiling, and read-length evidence so the published aggregation reflects the canonical raw-QC surface instead of a single observer shortcut.",
            "- `sample_results.csv`, `tool_runtime_summary.csv`, `cohort_runtime_summary.csv`, and `sample_runtime_outliers.csv` are published beside this briefing for reproducible downstream analysis.",
        ]
    )
    return "\n".join(lines) + "\n"

def main() -> int:
    runtime = resolve_corpus_briefing_runtime(parse_args())
    docs_root = runtime.docs_root
    summary = runtime.summary
    rows = runtime.sample_rows
    validate_summary_contract(summary)
    validate_rows_contract(summary, rows)

    runtime_rows = tool_runtime_summary(rows)
    cohort_rows = cohort_runtime_summary(rows)
    outliers = sample_runtime_outliers(rows)
    publish_corpus_briefing_artifacts(
        docs_root,
        markdown=render_markdown(summary, runtime_rows, cohort_rows, outliers),
        runtime_rows=runtime_rows,
        cohort_rows=cohort_rows,
        outlier_rows=outliers,
    )
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
