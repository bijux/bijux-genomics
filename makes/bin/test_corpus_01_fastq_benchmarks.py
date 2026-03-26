from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
BIN_DIR = ROOT / "makes" / "bin"
if str(BIN_DIR) not in sys.path:
    sys.path.insert(0, str(BIN_DIR))

import corpus_01_fastq_benchmark_support as support
import audit_corpus_01_fastq_benchmark_docs as benchmark_docs_audit
import render_fastq_detect_adapters_corpus_01_briefing as detect_adapters_briefing
import render_fastq_detect_adapters_corpus_01_report as detect_adapters_report
import render_fastq_merge_pairs_corpus_01_briefing as merge_briefing
import render_fastq_merge_pairs_corpus_01_report as merge_report
import render_fastq_profile_overrepresented_sequences_corpus_01_briefing as overrepresented_briefing
import render_fastq_profile_overrepresented_sequences_corpus_01_report as overrepresented_report
import render_fastq_report_qc_corpus_01_briefing as report_qc_briefing
import render_fastq_report_qc_corpus_01_report as report_qc_report
import render_fastq_profile_read_lengths_corpus_01_briefing as profile_read_lengths_briefing
import render_fastq_profile_read_lengths_corpus_01_report as profile_read_lengths_report
import render_fastq_profile_reads_corpus_01_briefing as profile_reads_briefing
import render_fastq_profile_reads_corpus_01_report as profile_reads_report
import render_fastq_trim_reads_corpus_01_briefing as trim_reads_briefing
import render_fastq_trim_reads_corpus_01_report as trim_reads_report
import render_fastq_trim_terminal_damage_corpus_01_briefing as terminal_damage_briefing
import render_fastq_trim_terminal_damage_corpus_01_report as terminal_damage_report
import render_fastq_trim_polyg_tails_corpus_01_briefing as trim_polyg_briefing
import render_fastq_trim_polyg_tails_corpus_01_report as trim_polyg_report


class CorpusBenchmarkSupportTests(unittest.TestCase):
    def test_validate_corpus_contract_accepts_balanced_manifest(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            corpus_root = Path(tmpdir)
            normalized = corpus_root / "normalized"
            normalized.mkdir(parents=True)

            spec = {
                "target_ancient_se": 1,
                "target_ancient_pe": 1,
                "target_modern_se": 1,
                "target_modern_pe": 1,
                "samples": [
                    {
                        "accession": "ACC_ANCIENT_SE",
                        "era": "ancient",
                        "layout": "se",
                        "study_accession": "PRJ1",
                        "size_band": "under_100mb",
                    },
                    {
                        "accession": "ACC_ANCIENT_PE",
                        "era": "ancient",
                        "layout": "pe",
                        "study_accession": "PRJ2",
                        "size_band": "under_100mb",
                    },
                    {
                        "accession": "ACC_MODERN_SE",
                        "era": "modern",
                        "layout": "se",
                        "study_accession": "PRJ3",
                        "size_band": "under_500mb",
                    },
                    {
                        "accession": "ACC_MODERN_PE",
                        "era": "modern",
                        "layout": "pe",
                        "study_accession": "PRJ4",
                        "size_band": "under_500mb",
                    },
                ],
            }

            files = {
                "raw/ACC_ANCIENT_SE/reads.fastq.gz": "digest-ancient-se",
                "normalized/sample_0001_R1.fastq.gz": "digest-ancient-se",
                "raw/ACC_ANCIENT_PE/R1.fastq.gz": "digest-ancient-pe-r1",
                "raw/ACC_ANCIENT_PE/R2.fastq.gz": "digest-ancient-pe-r2",
                "normalized/sample_0002_R1.fastq.gz": "digest-ancient-pe-r1",
                "normalized/sample_0002_R2.fastq.gz": "digest-ancient-pe-r2",
                "raw/ACC_MODERN_SE/reads.fastq.gz": "digest-modern-se",
                "normalized/sample_0003_R1.fastq.gz": "digest-modern-se",
                "raw/ACC_MODERN_PE/R1.fastq.gz": "digest-modern-pe-r1",
                "raw/ACC_MODERN_PE/R2.fastq.gz": "digest-modern-pe-r2",
                "normalized/sample_0004_R1.fastq.gz": "digest-modern-pe-r1",
                "normalized/sample_0004_R2.fastq.gz": "digest-modern-pe-r2",
            }
            (corpus_root / "MANIFEST.json").write_text(
                json.dumps({"files": files}, indent=2) + "\n",
                encoding="utf-8",
            )

            for relative_path in [
                "normalized/sample_0001_R1.fastq.gz",
                "normalized/sample_0002_R1.fastq.gz",
                "normalized/sample_0002_R2.fastq.gz",
                "normalized/sample_0003_R1.fastq.gz",
                "normalized/sample_0004_R1.fastq.gz",
                "normalized/sample_0004_R2.fastq.gz",
            ]:
                (corpus_root / relative_path).write_bytes(b"test\n")

            samples = support.discover_normalized_samples(corpus_root, expected_total=4)
            metadata = support.validate_corpus_contract(corpus_root, spec, samples)

            self.assertEqual(metadata["sample_0001"]["accession"], "ACC_ANCIENT_SE")
            self.assertEqual(metadata["sample_0004"]["layout"], "pe")

    def test_require_exact_tool_roster_rejects_missing_tool(self) -> None:
        with self.assertRaises(SystemExit):
            support.require_exact_tool_roster(
                "fastq.trim_polyg_tails",
                ["fastp"],
                ["fastp", "bbduk"],
            )

    def test_select_paired_samples_rejects_unbalanced_subset(self) -> None:
        spec = {
            "target_ancient_pe": 1,
            "target_modern_pe": 1,
        }
        samples = [
            {"sample_id": "sample_0001"},
            {"sample_id": "sample_0002"},
        ]
        metadata_by_sample = {
            "sample_0001": {"era": "ancient", "layout": "pe"},
            "sample_0002": {"era": "ancient", "layout": "pe"},
        }

        with self.assertRaises(SystemExit):
            support.select_paired_samples(spec, samples, metadata_by_sample)

    def test_resolve_corpus_metadata_falls_back_to_published_docs(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            repo_root = Path(tmpdir)
            docs_root = repo_root / "docs" / "benchmark" / "fastq.validate_reads" / "corpus-01"
            docs_root.mkdir(parents=True)
            (docs_root / "sample_results.csv").write_text(
                "\n".join(
                    [
                        "sample_id,accession,era,layout,study_accession,size_band,tool",
                        "sample_0001,ACC_ANCIENT_SE,ancient,se,PRJ1,under_100mb,fastqvalidator",
                        "sample_0002,ACC_ANCIENT_PE,ancient,pe,PRJ2,under_100mb,fastqvalidator",
                        "sample_0003,ACC_MODERN_SE,modern,se,PRJ3,under_500mb,fastqvalidator",
                        "sample_0004,ACC_MODERN_PE,modern,pe,PRJ4,under_500mb,fastqvalidator",
                    ]
                )
                + "\n",
                encoding="utf-8",
            )
            corpus_root = repo_root / "missing-corpus"
            spec = {
                "target_ancient_se": 1,
                "target_ancient_pe": 1,
                "target_modern_se": 1,
                "target_modern_pe": 1,
            }

            metadata = support.resolve_corpus_metadata(
                repo_root,
                corpus_root,
                spec,
                expected_sample_ids=[
                    "sample_0001",
                    "sample_0002",
                    "sample_0003",
                    "sample_0004",
                ],
            )

            self.assertEqual(metadata["sample_0001"]["accession"], "ACC_ANCIENT_SE")
            self.assertEqual(metadata["sample_0004"]["layout"], "pe")


class CorpusBenchmarkDocsAuditTests(unittest.TestCase):
    def test_audit_docs_reports_missing_stage_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            docs_root = Path(tmpdir) / "docs" / "benchmark"
            stage_root = docs_root / "fastq.validate_reads"
            corpus_root = stage_root / "corpus-01"
            corpus_root.mkdir(parents=True)
            (stage_root / "corpus-01-method.md").write_text("# method\n", encoding="utf-8")
            (corpus_root / "summary.json").write_text(
                json.dumps(
                    {
                        "stage_id": "fastq.validate_reads",
                        "scenario_id": "validation_fairness",
                    }
                )
                + "\n",
                encoding="utf-8",
            )
            (corpus_root / "sample_results.csv").write_text("sample_id,tool\n", encoding="utf-8")
            report = benchmark_docs_audit.audit_docs(docs_root)
            validate_report = next(
                stage for stage in report["stages"] if stage["stage_id"] == "fastq.validate_reads"
            )

            self.assertEqual(validate_report["status"], "incomplete")
            self.assertGreaterEqual(validate_report["issue_count"], 4)
            self.assertTrue(
                any(issue["issue_id"] == "missing-lunarc-md" for issue in validate_report["issues"])
            )

    def test_render_markdown_summarizes_completion_and_issue_count(self) -> None:
        report = {
            "stage_count": 2,
            "completed_stage_count": 1,
            "issue_count": 3,
            "stages": [
                {
                    "stage_id": "fastq.validate_reads",
                    "status": "complete",
                    "issue_count": 0,
                    "issues": [],
                },
                {
                    "stage_id": "fastq.trim_reads",
                    "status": "incomplete",
                    "issue_count": 3,
                    "issues": [
                        {
                            "issue_id": "missing-corpus-dir",
                            "detail": "missing docs/benchmark/fastq.trim_reads/corpus-01",
                        }
                    ],
                },
            ],
        }

        markdown = benchmark_docs_audit.render_markdown(report)

        self.assertIn("Completed stage dossiers: `1`", markdown)
        self.assertIn("Publication issues: `3`", markdown)
        self.assertIn("`fastq.trim_reads`: `incomplete` (`3` issues)", markdown)

    def test_merge_stage_is_tracked_in_publication_audit(self) -> None:
        stage_ids = [contract.stage_id for contract in benchmark_docs_audit.STAGE_CONTRACTS]

        self.assertIn("fastq.merge_pairs", stage_ids)

    def test_report_qc_stage_is_tracked_in_publication_audit(self) -> None:
        stage_ids = [contract.stage_id for contract in benchmark_docs_audit.STAGE_CONTRACTS]

        self.assertIn("fastq.report_qc", stage_ids)


class TrimPolygReportingTests(unittest.TestCase):
    def test_trim_polyg_summary_tracks_runtime_and_retention(self) -> None:
        records = [
            {
                "tool": "fastp",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.97",
                "bases_trimmed_polyg": "24",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "fastp",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "bases_trimmed_polyg": "28",
                "mean_q_delta": "0.40",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.96",
                "bases_trimmed_polyg": "21",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.94",
                "bases_trimmed_polyg": "25",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = trim_polyg_briefing.tool_runtime_summary(records)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["fastp"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["fastp"]["median_base_retention"], 0.96)
        self.assertAlmostEqual(by_tool["fastp"]["mean_bases_trimmed_polyg"], 26.0)
        self.assertGreater(
            by_tool["bbduk"]["slowdown_vs_fastest_median"],
            by_tool["fastp"]["slowdown_vs_fastest_median"],
        )

    def test_trim_polyg_outliers_capture_slowest_and_strongest_trim_tools(self) -> None:
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "fastp",
                "runtime_s": "5.0",
                "bases_trimmed_polyg": "40",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "bbduk",
                "runtime_s": "6.5",
                "bases_trimmed_polyg": "22",
            },
        ]

        outliers = trim_polyg_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbduk")
        self.assertEqual(outliers[0]["most_trimming_tool"], "fastp")
        self.assertAlmostEqual(outliers[0]["most_trimmed_bases"], 40.0)

    def test_trim_polyg_markdown_mentions_polyx_preset(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_polyg_tails/lunarc",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastp",
                "fastest_runtime_s": 0.9,
                "largest_polyg_trim_tool": "fastp",
                "largest_polyg_trim_bases": 26.0,
                "best_base_retention_tool": "fastp",
                "best_base_retention": 0.96,
            },
            "tool_summary": [
                {
                    "tool": "fastp",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.96,
                    "mean_bases_trimmed_polyg": 26.0,
                    "mean_q_delta": 0.35,
                }
            ],
        }

        markdown = trim_polyg_report.render_markdown(summary)

        self.assertIn("PolyX preset: `illumina_twocolor`", markdown)
        self.assertIn("Mean bases trimmed", markdown)

    def test_trim_polyg_report_contract_rejects_mixed_preset_rows(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "trim_polyg": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "polyx_preset": "illumina_twocolor",
                "min_polyg_run": 10,
                "trim_polyg": True,
            },
            {
                "sample_id": "sample_0001",
                "tool": "bbduk",
                "raw_backend_report_format": "bbduk_stats",
                "polyx_preset": "wrong_preset",
                "min_polyg_run": 10,
                "trim_polyg": True,
            },
        ]

        with self.assertRaises(SystemExit):
            trim_polyg_report.validate_trim_polyg_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )


class ReportQcReportingTests(unittest.TestCase):
    def test_report_qc_tool_summary_tracks_multiqc_and_governed_inputs(self) -> None:
        rows = [
            {
                "tool": "multiqc",
                "runtime_s": "4.2",
                "exit_code": "0",
                "multiqc_module_count": "8",
                "multiqc_sample_count": "1",
                "governed_qc_input_count": "6",
                "contamination_rate": "0.012",
                "mean_q": "34.1",
            },
            {
                "tool": "multiqc",
                "runtime_s": "4.8",
                "exit_code": "0",
                "multiqc_module_count": "9",
                "multiqc_sample_count": "1",
                "governed_qc_input_count": "6",
                "contamination_rate": "0.010",
                "mean_q": "34.6",
            },
        ]

        summary_rows = report_qc_briefing.tool_runtime_summary(rows)

        self.assertEqual(len(summary_rows), 1)
        self.assertAlmostEqual(summary_rows[0]["median_runtime_s"], 4.5)
        self.assertAlmostEqual(summary_rows[0]["median_multiqc_module_count"], 8.5)
        self.assertAlmostEqual(summary_rows[0]["median_governed_qc_input_count"], 6.0)
        self.assertAlmostEqual(summary_rows[0]["median_mean_q"], 34.35)

    def test_report_qc_markdown_mentions_aggregation_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/results/corpus_01/fastq.report_qc/lunarc",
            "scenario_id": "qc_aggregation_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["multiqc"],
            "aggregation_engine": "multiqc",
            "aggregation_scope": "governed_qc_artifacts",
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "governed_contributor_stage_ids": [
                "fastq.validate_reads",
                "fastq.detect_adapters",
                "fastq.profile_reads",
                "fastq.profile_read_lengths",
            ],
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "multiqc",
                "fastest_runtime_s": 4.5,
                "largest_multiqc_module_tool": "multiqc",
                "largest_multiqc_module_count": 8.5,
                "highest_governed_input_tool": "multiqc",
                "highest_governed_input_count": 6.0,
            },
            "tool_summary": [
                {
                    "tool": "multiqc",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 4.5,
                    "median_multiqc_module_count": 8.5,
                    "median_multiqc_sample_count": 1.0,
                    "median_governed_qc_input_count": 6.0,
                    "median_contamination_rate": 0.011,
                    "median_mean_q": 34.35,
                }
            ],
        }

        markdown = report_qc_report.render_markdown(summary)

        self.assertIn("aggregation_engine: `multiqc`", markdown)
        self.assertIn("Governed contributor stages", markdown)
        self.assertIn("Median governed inputs", markdown)

    def test_report_qc_report_contract_rejects_mismatched_governed_input_count(self) -> None:
        run_manifest = {
            "tools": ["multiqc"],
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "multiqc",
                "reads_in": 100,
                "reads_out": 100,
                "bases_in": 1000,
                "bases_out": 1000,
                "governed_qc_input_count": 5,
                "expected_governed_qc_input_count": 6,
                "mean_q": 34.0,
                "contamination_rate": 0.01,
                "exit_code": 1,
            }
        ]

        with self.assertRaises(SystemExit):
            report_qc_report.validate_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_report_qc_enriches_missing_multiqc_artifacts_from_bundle(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            data_dir = Path(tmpdir) / "multiqc_data"
            report_data_dir = data_dir / "multiqc_report_data"
            report_data_dir.mkdir(parents=True)
            (data_dir / "multiqc_report.html").write_text("<html></html>\n", encoding="utf-8")
            (report_data_dir / "multiqc_data.json").write_text(
                json.dumps(
                    {
                        "report_general_stats_data": [
                            {"sample_a": {"total_sequences": 10}, "sample_b": {"total_sequences": 12}}
                        ],
                        "report_plot_data": {
                            "general_stats_table": {},
                            "fastqc_sequence_counts_plot": {},
                            "fastqc_adapter_content_plot": {},
                        },
                    }
                )
                + "\n",
                encoding="utf-8",
            )

            row = report_qc_report.enrich_multiqc_artifacts(
                {
                    "multiqc_data": str(data_dir),
                    "multiqc_report": "",
                    "multiqc_sample_count": None,
                    "multiqc_module_count": None,
                }
            )

            self.assertEqual(row["multiqc_sample_count"], 2)
            self.assertEqual(row["multiqc_module_count"], 3)
            self.assertEqual(row["multiqc_report"], str(data_dir / "multiqc_report.html"))


class TrimReadsReportingTests(unittest.TestCase):
    def test_trim_reads_summary_tracks_runtime_and_retention(self) -> None:
        rows = [
            {
                "tool": "fastp",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.97",
                "read_retention": "0.96",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "fastp",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.95",
                "read_retention": "0.94",
                "mean_q_delta": "0.40",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.96",
                "read_retention": "0.93",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "bbduk",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.94",
                "read_retention": "0.91",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = trim_reads_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["fastp"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["fastp"]["median_base_retention"], 0.96)
        self.assertAlmostEqual(by_tool["fastp"]["median_read_retention"], 0.95)
        self.assertGreater(
            by_tool["bbduk"]["slowdown_vs_fastest_median"],
            by_tool["fastp"]["slowdown_vs_fastest_median"],
        )

    def test_trim_reads_outliers_capture_slowest_and_lowest_retention_tools(self) -> None:
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "fastp",
                "runtime_s": "5.0",
                "base_retention": "0.98",
            },
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "modern",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ1",
                "tool": "bbduk",
                "runtime_s": "6.5",
                "base_retention": "0.91",
            },
        ]

        outliers = trim_reads_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbduk")
        self.assertEqual(outliers[0]["lowest_retention_tool"], "bbduk")
        self.assertAlmostEqual(outliers[0]["lowest_base_retention"], 0.91)

    def test_trim_reads_markdown_mentions_trim_policy_bundle(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_reads/lunarc",
            "scenario_id": "trim_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastp",
                "fastest_runtime_s": 0.9,
                "best_base_retention_tool": "fastp",
                "best_base_retention": 0.96,
                "best_read_retention_tool": "fastp",
                "best_read_retention": 0.95,
                "best_q_gain_tool": "fastp",
                "best_q_gain": 0.35,
            },
            "tool_summary": [
                {
                    "tool": "fastp",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.96,
                    "median_read_retention": 0.95,
                    "mean_q_delta": 0.35,
                }
            ],
        }

        markdown = trim_reads_report.render_markdown(summary)

        self.assertIn("adapter_policy: `none`", markdown)
        self.assertIn("Median read retention", markdown)

    def test_trim_reads_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
            "adapter_bank_preset": None,
            "polyx_preset": None,
            "contaminant_preset": None,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "min_length": 30,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_bank_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            },
            {
                "sample_id": "sample_0001",
                "tool": "bbduk",
                "raw_backend_report_format": "bbduk_stats",
                "min_length": 20,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_bank_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            },
        ]

        with self.assertRaises(SystemExit):
            trim_reads_report.validate_trim_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )


class MergeReportingTests(unittest.TestCase):
    def test_merge_runtime_summary_tracks_runtime_and_merge_rate(self) -> None:
        rows = [
            {
                "tool": "pear",
                "runtime_s": "1.0",
                "exit_code": "0",
                "merge_rate": "0.80",
                "base_retention": "0.70",
                "reads_merged": "800",
            },
            {
                "tool": "pear",
                "runtime_s": "1.2",
                "exit_code": "0",
                "merge_rate": "0.84",
                "base_retention": "0.74",
                "reads_merged": "840",
            },
            {
                "tool": "bbmerge",
                "runtime_s": "2.0",
                "exit_code": "0",
                "merge_rate": "0.81",
                "base_retention": "0.72",
                "reads_merged": "810",
            },
            {
                "tool": "bbmerge",
                "runtime_s": "2.2",
                "exit_code": "0",
                "merge_rate": "0.82",
                "base_retention": "0.71",
                "reads_merged": "820",
            },
        ]

        summary_rows = merge_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["pear"]["median_runtime_s"], 1.1)
        self.assertAlmostEqual(by_tool["pear"]["median_merge_rate"], 0.82)
        self.assertAlmostEqual(by_tool["pear"]["mean_reads_merged"], 820.0)
        self.assertGreater(
            by_tool["bbmerge"]["slowdown_vs_fastest_median"],
            by_tool["pear"]["slowdown_vs_fastest_median"],
        )

    def test_merge_outliers_capture_slowest_and_best_merge_tool(self) -> None:
        rows = [
            {
                "sample_id": "sample_0008",
                "accession": "ACC8",
                "era": "ancient",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ8",
                "tool": "pear",
                "runtime_s": "3.0",
                "merge_rate": "0.83",
            },
            {
                "sample_id": "sample_0008",
                "accession": "ACC8",
                "era": "ancient",
                "layout": "pe",
                "size_band": "under_500mb",
                "study_accession": "PRJ8",
                "tool": "bbmerge",
                "runtime_s": "4.5",
                "merge_rate": "0.79",
            },
        ]

        outliers = merge_briefing.sample_runtime_outliers(rows)

        self.assertEqual(outliers[0]["slowest_tool"], "bbmerge")
        self.assertEqual(outliers[0]["best_merge_rate_tool"], "pear")
        self.assertAlmostEqual(outliers[0]["best_merge_rate"], 0.83)

    def test_merge_markdown_mentions_paired_only_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.merge_pairs/lunarc",
            "scenario_id": "merge_fairness",
            "samples_total": 10,
            "samples_failed": 0,
            "tools": ["adapterremoval", "pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
            "era_counts": {"ancient": 5, "modern": 5},
            "cohort_counts": {"ancient_pe": 5, "modern_pe": 5},
            "headline": {
                "fastest_tool": "pear",
                "fastest_runtime_s": 1.1,
                "best_merge_rate_tool": "pear",
                "best_merge_rate": 0.82,
                "best_base_retention_tool": "pear",
                "best_base_retention": 0.72,
            },
            "tool_summary": [
                {
                    "tool": "pear",
                    "records": 10,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.1,
                    "median_merge_rate": 0.82,
                    "median_base_retention": 0.72,
                    "mean_reads_merged": 820.0,
                }
            ],
        }

        markdown = merge_report.render_markdown(summary)

        self.assertIn("Samples benchmarked: `10` paired-end inputs", markdown)
        self.assertIn("merge_overlap: `governed tool default`", markdown)
        self.assertIn("min_length: `governed tool default`", markdown)
        self.assertIn("unmerged_read_policy: `emit_unmerged_pairs`", markdown)

    def test_merge_report_contract_rejects_rate_drift(self) -> None:
        run_manifest = {
            "tools": ["pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
        }
        sample_rows = [
            {
                "sample_id": "sample_0008",
                "tool": "pear",
                "layout": "pe",
                "merge_overlap": None,
                "min_length": None,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "pairs_in": 100,
                "reads_merged": 80,
                "reads_unmerged": 20,
                "merge_rate": 0.70,
            }
        ]

        with self.assertRaises(SystemExit):
            merge_report.validate_merge_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0008"],
            )

    def test_merge_report_contract_rejects_missing_sample_rows(self) -> None:
        run_manifest = {
            "tools": ["pear"],
            "merge_overlap": None,
            "min_length": None,
            "unmerged_read_policy": "emit_unmerged_pairs",
        }
        sample_rows = [
            {
                "sample_id": "sample_0008",
                "tool": "pear",
                "layout": "pe",
                "merge_overlap": None,
                "min_length": None,
                "unmerged_read_policy": "emit_unmerged_pairs",
                "pairs_in": 100,
                "reads_merged": 80,
                "reads_unmerged": 20,
                "merge_rate": 0.8,
            }
        ]

        with self.assertRaises(SystemExit):
            merge_report.validate_merge_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0008", "sample_0009"],
            )

    def test_merge_report_localizes_remote_results_path(self) -> None:
        local_results_root = Path("/tmp/local-results")

        localized = merge_report.localize_results_path(
            "/home/bijan/bijux/results/corpus_01/fastq.merge_pairs/lunarc/bench/merge_pairs/sample_0008/report.json",
            local_results_root,
        )

        self.assertEqual(
            localized,
            local_results_root
            / "corpus_01/fastq.merge_pairs/lunarc/bench/merge_pairs/sample_0008/report.json",
        )

    def test_trim_reads_report_contract_rejects_missing_tool_rows(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "min_length": 30,
            "quality_cutoff": None,
            "n_policy": "retain",
            "adapter_policy": "none",
            "polyx_policy": "none",
            "contaminant_policy": "none",
            "adapter_bank_preset": None,
            "polyx_preset": None,
            "contaminant_preset": None,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "min_length": 30,
                "quality_cutoff": None,
                "n_policy": "retain",
                "adapter_policy": "none",
                "polyx_policy": "none",
                "contaminant_policy": "none",
                "adapter_bank_preset": None,
                "polyx_preset": None,
                "contaminant_preset": None,
            }
        ]

        with self.assertRaises(SystemExit):
            trim_reads_report.validate_trim_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_trim_reads_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            trim_reads_report.validate_trim_run_manifest_contract(
                {
                    "stage_id": "fastq.trim_reads",
                    "scenario_id": "trim_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )


class DetectAdaptersReportingTests(unittest.TestCase):
    def test_detect_adapters_summary_tracks_runtime_and_signal(self) -> None:
        rows = [
            {
                "tool": "fastqc",
                "runtime_s": "1.2",
                "exit_code": "0",
                "candidate_adapter_count": "2",
                "adapter_trimmed_fraction": "",
                "mean_q": "31.5",
            },
            {
                "tool": "fastqc",
                "runtime_s": "1.4",
                "exit_code": "0",
                "candidate_adapter_count": "4",
                "adapter_trimmed_fraction": "",
                "mean_q": "32.5",
            },
        ]

        summary_rows = detect_adapters_briefing.tool_runtime_summary(rows)

        self.assertEqual(len(summary_rows), 1)
        self.assertAlmostEqual(summary_rows[0]["median_runtime_s"], 1.3)
        self.assertAlmostEqual(summary_rows[0]["mean_candidate_adapter_count"], 3.0)
        self.assertAlmostEqual(summary_rows[0]["median_mean_q"], 32.0)

    def test_detect_adapters_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.detect_adapters",
            "scenario_id": "detect_adapters_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.detect_adapters/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["adapter_observer"],
            "inspection_mode": "evidence_only",
            "report_only": True,
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "adapter_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "candidate_adapter_count": "2",
                "adapter_trimmed_fraction": "",
                "mean_q": "31.0",
            }
        ]
        runtime_rows = detect_adapters_briefing.tool_runtime_summary(rows)
        cohort_rows = detect_adapters_briefing.cohort_runtime_summary(rows)
        outliers = detect_adapters_briefing.sample_runtime_outliers(rows)

        markdown = detect_adapters_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`adapter_observer` ran at", markdown)
        self.assertNotIn("`fastqc` ran at", markdown)

    def test_detect_adapters_briefing_rejects_tool_drift(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_briefing.validate_rows_contract(
                {"tools": ["fastqc"]},
                rows=[
                    {
                        "tool": "other_tool",
                    }
                ],
            )

    def test_detect_adapters_markdown_mentions_observer_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.detect_adapters/lunarc",
            "scenario_id": "detect_adapters_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastqc"],
            "inspection_mode": "evidence_only",
            "report_only": True,
            "evidence_scope": "full_input",
            "evidence_format": "fastqc_summary",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "fastqc",
                "fastest_runtime_s": 1.3,
                "largest_adapter_signal_tool": "fastqc",
                "largest_adapter_signal": 3.0,
                "highest_trimmed_fraction_tool": None,
                "highest_trimmed_fraction": None,
            },
            "tool_summary": [
                {
                    "tool": "fastqc",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.3,
                    "mean_candidate_adapter_count": 3.0,
                    "mean_adapter_trimmed_fraction": None,
                    "median_mean_q": 32.0,
                }
            ],
        }

        markdown = detect_adapters_report.render_markdown(summary)

        self.assertIn("inspection_mode: `evidence_only`", markdown)
        self.assertIn("report_only: `True`", markdown)

    def test_detect_adapters_report_contract_rejects_mutating_rows(self) -> None:
        run_manifest = {"tools": ["fastqc"]}
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastqc",
                "reads_in": 100,
                "reads_out": 99,
                "bases_in": 1000,
                "bases_out": 1000,
                "adapter_trimmed_fraction": None,
            }
        ]

        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
                expected_sample_ids=["sample_0001"],
            )

    def test_detect_adapters_report_contract_rejects_missing_sample_rows(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_row_contract(
                run_manifest={"tools": ["fastqc"]},
                sample_rows=[],
                expected_sample_ids=["sample_0001"],
            )

    def test_detect_adapters_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_run_manifest_contract(
                {
                    "stage_id": "fastq.detect_adapters",
                    "scenario_id": "detect_adapters_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                    "inspection_mode": "evidence_only",
                    "report_only": True,
                    "evidence_scope": "full_input",
                    "evidence_format": "fastqc_summary",
                }
            )

    def test_detect_adapters_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            detect_adapters_report.validate_detect_run_manifest_contract(
                {
                    "stage_id": "fastq.detect_adapters",
                    "scenario_id": "detect_adapters_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "inspection_mode": "evidence_only",
                    "report_only": True,
                    "evidence_scope": "full_input",
                    "evidence_format": "fastqc_summary",
                }
            )


class OverrepresentedReportingTests(unittest.TestCase):
    def test_overrepresented_markdown_mentions_top_k_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_overrepresented_sequences/lunarc",
            "scenario_id": "overrepresented_sequence_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["fastqc", "fastq_scan", "seqkit"],
            "top_k": 50,
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit",
                "fastest_runtime_s": 0.8,
                "largest_sequence_count_tool": "fastqc",
                "largest_sequence_count": 12.0,
                "highest_top_fraction_tool": "fastq_scan",
                "highest_top_fraction": 0.12,
            },
            "tool_summary": [
                {
                    "tool": "seqkit",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.8,
                    "median_sequence_count": 10.0,
                    "median_flagged_sequences": 2.0,
                    "median_top_fraction": 0.09,
                }
            ],
        }

        markdown = overrepresented_report.render_markdown(summary)

        self.assertIn("top_k: `50`", markdown)
        self.assertIn("Median flagged sequences", markdown)

    def test_overrepresented_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_overrepresented_sequences",
                    "scenario_id": "overrepresented_sequence_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )

    def test_overrepresented_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_overrepresented_sequences",
                    "scenario_id": "overrepresented_sequence_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 4,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "top_k": 50,
                    "overrepresented_artifacts": [
                        "overrepresented_sequences_tsv",
                        "overrepresented_sequences_json",
                        "report_json",
                    ],
                }
            )

    def test_overrepresented_report_contract_rejects_missing_sample_tool_rows(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_row_contract(
                run_manifest={"tools": ["fastqc", "seqkit"], "top_k": 50},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": 5,
                        "flagged_sequences": 1,
                        "top_fraction": 0.1,
                        "top_k": 50,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_overrepresented_report_contract_rejects_excess_ranked_sequences(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_report.validate_overrepresented_row_contract(
                run_manifest={"tools": ["fastqc"], "top_k": 5},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": 6,
                        "flagged_sequences": 1,
                        "top_fraction": 0.1,
                        "top_k": 5,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_overrepresented_report_validates_artifact_publication(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = (
                Path(tmpdir)
                / "bench"
                / "profile_overrepresented_sequences"
                / "sample_0001"
                / "report.json"
            )
            tool_dir = report_path.parent / "tools" / "fastqc"
            tool_dir.mkdir(parents=True)
            (tool_dir / "overrepresented_sequences.tsv").write_text(
                "sequence\tcount\nACGT\t4\n",
                encoding="utf-8",
            )
            (tool_dir / "overrepresented_sequences.json").write_text(
                json.dumps({"sequence_count": 1}) + "\n",
                encoding="utf-8",
            )
            (tool_dir / "overrepresented_report.json").write_text(
                json.dumps({"top_fraction": 0.2}) + "\n",
                encoding="utf-8",
            )

            artifacts = overrepresented_report.validate_artifact_paths(report_path, "fastqc")

        self.assertTrue(
            artifacts["overrepresented_sequences_tsv_artifact"].endswith(
                "overrepresented_sequences.tsv"
            )
        )
        self.assertTrue(
            artifacts["overrepresented_sequences_json_artifact"].endswith(
                "overrepresented_sequences.json"
            )
        )

    def test_overrepresented_report_rejects_empty_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = (
                Path(tmpdir)
                / "bench"
                / "profile_overrepresented_sequences"
                / "sample_0001"
                / "report.json"
            )
            tool_dir = report_path.parent / "tools" / "fastqc"
            tool_dir.mkdir(parents=True)
            (tool_dir / "overrepresented_sequences.tsv").write_text("", encoding="utf-8")
            (tool_dir / "overrepresented_sequences.json").write_text("{}", encoding="utf-8")
            (tool_dir / "overrepresented_report.json").write_text("{}", encoding="utf-8")

            with self.assertRaises(SystemExit):
                overrepresented_report.validate_artifact_paths(report_path, "fastqc")

    def test_overrepresented_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_overrepresented_sequences",
            "scenario_id": "overrepresented_sequence_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_overrepresented_sequences/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["observer_a"],
            "top_k": 50,
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "observer_a",
                "runtime_s": "1.0",
                "exit_code": "0",
                "sequence_count": "5",
                "flagged_sequences": "1",
                "top_fraction": "0.1",
                "top_k": "50",
                "overrepresented_sequences_tsv_artifact": "/tmp/overrepresented_sequences.tsv",
                "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                "report_json_artifact": "/tmp/overrepresented_report.json",
            }
        ]

        runtime_rows = overrepresented_briefing.tool_runtime_summary(rows)
        cohort_rows = overrepresented_briefing.cohort_runtime_summary(rows)
        outliers = overrepresented_briefing.sample_runtime_outliers(rows)

        markdown = overrepresented_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`observer_a` ran at", markdown)
        self.assertNotIn("`fastqc` ran at", markdown)

    def test_overrepresented_briefing_rejects_sequence_count_drift(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_briefing.validate_rows_contract(
                {"tools": ["fastqc"], "top_k": 5},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": "6",
                        "flagged_sequences": "1",
                        "top_fraction": "0.1",
                        "top_k": "5",
                        "overrepresented_sequences_tsv_artifact": "/tmp/overrepresented_sequences.tsv",
                        "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                        "report_json_artifact": "/tmp/overrepresented_report.json",
                    }
                ],
            )

    def test_overrepresented_briefing_rejects_artifact_suffix_drift(self) -> None:
        with self.assertRaises(SystemExit):
            overrepresented_briefing.validate_rows_contract(
                {"tools": ["fastqc"], "top_k": 50},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "fastqc",
                        "sequence_count": "5",
                        "flagged_sequences": "1",
                        "top_fraction": "0.1",
                        "top_k": "50",
                        "overrepresented_sequences_tsv_artifact": "/tmp/wrong.tsv",
                        "overrepresented_sequences_json_artifact": "/tmp/overrepresented_sequences.json",
                        "report_json_artifact": "/tmp/overrepresented_report.json",
                    }
                ],
            )


class ProfileReadsReportingTests(unittest.TestCase):
    def test_profile_reads_summary_tracks_runtime_and_profile_metrics(self) -> None:
        histogram = [{"length": 50, "count": 2}, {"length": 75, "count": 2}]
        derived = profile_reads_report.derived_histogram_metrics(histogram)

        self.assertEqual(derived["histogram_bin_count"], 2)
        self.assertEqual(derived["max_observed_length"], 75)
        self.assertAlmostEqual(derived["mean_read_length"], 62.5)

    def test_profile_reads_markdown_mentions_profile_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_reads/lunarc",
            "scenario_id": "profile_reads_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["seqkit_stats"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_tsv",
            "length_histogram_source": "seqkit_fx2tab",
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit_stats",
                "fastest_runtime_s": 1.1,
                "highest_mean_q_tool": "seqkit_stats",
                "highest_mean_q": 33.2,
                "widest_histogram_tool": "seqkit_stats",
                "widest_histogram_bins": 42,
            },
            "tool_summary": [
                {
                    "tool": "seqkit_stats",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.1,
                    "median_reads_total": 1000.0,
                    "median_bases_total": 75000.0,
                    "median_mean_q": 33.2,
                    "median_gc_percent": 45.0,
                    "median_read_length": 75.0,
                    "median_histogram_bin_count": 42.0,
                }
            ],
        }

        markdown = profile_reads_report.render_markdown(summary)

        self.assertIn("raw_backend_report_format: `seqkit_stats_tsv`", markdown)
        self.assertIn("length_histogram_source: `seqkit_fx2tab`", markdown)

    def test_profile_reads_report_contract_rejects_empty_histograms(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "reads_total": 100,
                        "bases_total": 1000,
                        "mean_q": 31.0,
                        "gc_percent": 45.0,
                        "histogram_bin_count": 0,
                        "max_observed_length": 75,
                        "mean_read_length": 10.0,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_profile_reads_report_contract_rejects_histogram_length_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "reads_total": 100,
                        "bases_total": 1000,
                        "mean_q": 31.0,
                        "gc_percent": 45.0,
                        "histogram_bin_count": 10,
                        "max_observed_length": 5,
                        "mean_read_length": 10.0,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_profile_reads_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_tsv",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_report.validate_profile_reads_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_tsv",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_reads",
            "scenario_id": "profile_reads_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_reads/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["profile_observer"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_tsv",
            "length_histogram_source": "seqkit_fx2tab",
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "profile_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "reads_total": "100",
                "bases_total": "5000",
                "mean_q": "31.0",
                "gc_percent": "44.0",
                "histogram_bin_count": "2",
                "max_observed_length": "75",
                "mean_read_length": "50.0",
            }
        ]
        runtime_rows = profile_reads_briefing.tool_runtime_summary(rows)
        cohort_rows = profile_reads_briefing.cohort_runtime_summary(rows)
        outliers = profile_reads_briefing.sample_runtime_outliers(rows)

        markdown = profile_reads_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`profile_observer` ran at", markdown)
        self.assertNotIn("`seqkit_stats` ran at", markdown)

    def test_profile_reads_briefing_rejects_contract_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_briefing.validate_summary_contract(
                {
                    "stage_id": "fastq.profile_reads",
                    "scenario_id": "profile_reads_fairness",
                    "tools": ["seqkit_stats"],
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "wrong",
                    "length_histogram_source": "seqkit_fx2tab",
                }
            )

    def test_profile_reads_briefing_rejects_histogram_row_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_reads_briefing.validate_rows_contract(
                {"tools": ["seqkit_stats"]},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "histogram_bin_count": "0",
                    }
                ],
            )


class ProfileReadLengthsReportingTests(unittest.TestCase):
    def test_read_length_markdown_mentions_histogram_contract(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_read_lengths/lunarc",
            "scenario_id": "read_length_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["seqkit_stats"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_length_histogram",
            "histogram_bins": 100,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "seqkit_stats",
                "fastest_runtime_s": 1.2,
                "highest_max_read_length_tool": "seqkit_stats",
                "highest_max_read_length": 151.0,
                "widest_length_support_tool": "seqkit_stats",
                "widest_length_support": 48.0,
            },
            "tool_summary": [
                {
                    "tool": "seqkit_stats",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 1.2,
                    "median_read_count": 1000.0,
                    "median_mean_read_length": 74.2,
                    "median_max_read_length": 151.0,
                    "median_distinct_lengths": 48.0,
                }
            ],
        }

        markdown = profile_read_lengths_report.render_markdown(summary)

        self.assertIn("raw_backend_report_format: `seqkit_stats_length_histogram`", markdown)
        self.assertIn("histogram_bins: `100`", markdown)

    def test_read_length_report_contract_rejects_invalid_distinct_lengths(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "read_count": 100,
                        "mean_read_length": 50.0,
                        "max_read_length": 75,
                        "distinct_lengths": 101,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_read_length_report_rejects_sample_limited_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "sample_limit": 2,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_length_histogram",
                    "histogram_bins": 100,
                    "length_histogram_artifacts": [
                        "report_json",
                        "length_distribution_tsv",
                        "length_distribution_json",
                    ],
                }
            )

    def test_read_length_report_rejects_nonpositive_histogram_bins(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_run_manifest_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": False,
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "seqkit_stats_length_histogram",
                    "histogram_bins": 0,
                    "length_histogram_artifacts": [
                        "report_json",
                        "length_distribution_tsv",
                        "length_distribution_json",
                    ],
                }
            )

    def test_read_length_report_rejects_nonpositive_mean_length(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_report.validate_read_length_row_contract(
                run_manifest={"tools": ["seqkit_stats"]},
                sample_rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "read_count": 100,
                        "mean_read_length": 0.0,
                        "max_read_length": 75,
                        "distinct_lengths": 10,
                    }
                ],
                expected_sample_ids=["sample_0001"],
            )

    def test_read_length_artifact_check_rejects_empty_files(self) -> None:
        with tempfile.TemporaryDirectory() as tmpdir:
            report_path = Path(tmpdir) / "bench" / "profile_read_lengths" / "sample_0001" / "report.json"
            tool_dir = report_path.parent / "tools" / "seqkit_stats"
            tool_dir.mkdir(parents=True)
            report_path.write_text("{}", encoding="utf-8")
            (tool_dir / "profile_read_lengths_report.json").write_text(
                "{}",
                encoding="utf-8",
            )
            (tool_dir / "length_distribution.tsv").write_text("", encoding="utf-8")
            (tool_dir / "length_distribution.json").write_text(
                "{\"histogram\": []}",
                encoding="utf-8",
            )

            with self.assertRaises(SystemExit):
                profile_read_lengths_report.validate_artifact_paths(
                    report_path, "seqkit_stats"
                )

    def test_read_length_briefing_avoids_hardcoded_tool_name(self) -> None:
        summary = {
            "stage_id": "fastq.profile_read_lengths",
            "scenario_id": "read_length_fairness",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.profile_read_lengths/lunarc",
            "samples_total": 1,
            "samples_failed": 0,
            "tools": ["length_observer"],
            "report_only": True,
            "mutates_fastq": False,
            "may_change_read_count": False,
            "raw_backend_report_format": "seqkit_stats_length_histogram",
            "histogram_bins": 100,
            "era_counts": {"ancient": 1, "modern": 0},
            "layout_counts": {"se": 1, "pe": 0},
        }
        rows = [
            {
                "sample_id": "sample_0001",
                "accession": "ACC1",
                "era": "ancient",
                "layout": "se",
                "size_band": "under_100mb",
                "study_accession": "PRJ1",
                "tool": "length_observer",
                "runtime_s": "1.0",
                "exit_code": "0",
                "read_count": "100",
                "mean_read_length": "50.0",
                "max_read_length": "75",
                "distinct_lengths": "12",
                "report_json_artifact": "/tmp/profile_read_lengths_report.json",
                "length_distribution_tsv_artifact": "/tmp/length_distribution.tsv",
                "length_distribution_json_artifact": "/tmp/length_distribution.json",
            }
        ]
        runtime_rows = profile_read_lengths_briefing.tool_runtime_summary(rows)
        cohort_rows = profile_read_lengths_briefing.cohort_runtime_summary(rows)
        outliers = profile_read_lengths_briefing.sample_runtime_outliers(rows)

        markdown = profile_read_lengths_briefing.render_markdown(
            summary, rows, runtime_rows, cohort_rows, outliers
        )

        self.assertIn("`length_observer` ran at", markdown)
        self.assertNotIn("`seqkit_stats` ran at", markdown)
        self.assertIn("Governed artifacts per sample/tool", markdown)

    def test_read_length_briefing_rejects_contract_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_briefing.validate_summary_contract(
                {
                    "stage_id": "fastq.profile_read_lengths",
                    "scenario_id": "read_length_fairness",
                    "tools": ["seqkit_stats"],
                    "report_only": True,
                    "mutates_fastq": False,
                    "may_change_read_count": False,
                    "raw_backend_report_format": "wrong",
                    "histogram_bins": 100,
                }
            )

    def test_read_length_briefing_rejects_artifact_row_drift(self) -> None:
        with self.assertRaises(SystemExit):
            profile_read_lengths_briefing.validate_rows_contract(
                {"tools": ["seqkit_stats"]},
                rows=[
                    {
                        "sample_id": "sample_0001",
                        "tool": "seqkit_stats",
                        "distinct_lengths": "10",
                        "report_json_artifact": "/tmp/not-right.json",
                        "length_distribution_tsv_artifact": "/tmp/length_distribution.tsv",
                        "length_distribution_json_artifact": "/tmp/length_distribution.json",
                    }
                ],
            )


class TerminalDamageReportingTests(unittest.TestCase):
    def test_terminal_damage_summary_tracks_runtime_and_asymmetry(self) -> None:
        rows = [
            {
                "tool": "cutadapt",
                "runtime_s": "0.8",
                "exit_code": "0",
                "base_retention": "0.95",
                "asymmetry_reduction": "0.25",
                "mean_q_delta": "0.30",
            },
            {
                "tool": "cutadapt",
                "runtime_s": "1.0",
                "exit_code": "0",
                "base_retention": "0.93",
                "asymmetry_reduction": "0.20",
                "mean_q_delta": "0.20",
            },
            {
                "tool": "seqkit",
                "runtime_s": "1.6",
                "exit_code": "0",
                "base_retention": "0.97",
                "asymmetry_reduction": "0.05",
                "mean_q_delta": "0.10",
            },
            {
                "tool": "seqkit",
                "runtime_s": "1.8",
                "exit_code": "0",
                "base_retention": "0.96",
                "asymmetry_reduction": "0.04",
                "mean_q_delta": "0.10",
            },
        ]

        summary_rows = terminal_damage_briefing.tool_runtime_summary(rows)
        by_tool = {row["tool"]: row for row in summary_rows}

        self.assertAlmostEqual(by_tool["cutadapt"]["median_runtime_s"], 0.9)
        self.assertAlmostEqual(by_tool["cutadapt"]["mean_asymmetry_reduction"], 0.225)
        self.assertGreater(
            by_tool["seqkit"]["median_base_retention"],
            by_tool["cutadapt"]["median_base_retention"],
        )

    def test_terminal_damage_markdown_mentions_damage_policy(self) -> None:
        summary = {
            "generated_at_utc": "2026-03-26T00:00:00+00:00",
            "platform": "lunarc-apptainer",
            "corpus_root": "/home/bijan/bijux/corpus_01",
            "run_root": "/home/bijan/bijux/corpus_01/benchmarks/fastq.trim_terminal_damage/lunarc",
            "scenario_id": "terminal_damage_fairness",
            "samples_total": 20,
            "samples_failed": 0,
            "tools": ["adapterremoval", "cutadapt", "seqkit"],
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
            "era_counts": {"ancient": 10, "modern": 10},
            "layout_counts": {"se": 10, "pe": 10},
            "cohort_counts": {"ancient_pe": 5, "ancient_se": 5, "modern_pe": 5, "modern_se": 5},
            "headline": {
                "fastest_tool": "cutadapt",
                "fastest_runtime_s": 0.9,
                "best_base_retention_tool": "seqkit",
                "best_base_retention": 0.965,
                "largest_asymmetry_reduction_tool": "cutadapt",
                "largest_asymmetry_reduction": 0.225,
            },
            "tool_summary": [
                {
                    "tool": "cutadapt",
                    "records": 20,
                    "pass_rate": 1.0,
                    "median_runtime_s": 0.9,
                    "median_base_retention": 0.94,
                    "mean_asymmetry_reduction": 0.225,
                    "mean_q_delta": 0.25,
                }
            ],
        }

        markdown = terminal_damage_report.render_markdown(summary)

        self.assertIn("execution_policy: `explicit_terminal_trim`", markdown)
        self.assertIn("Mean asymmetry reduction", markdown)

    def test_terminal_damage_report_contract_rejects_policy_drift(self) -> None:
        run_manifest = {
            "tools": ["adapterremoval", "cutadapt", "seqkit"],
            "damage_mode": "ancient",
            "execution_policy": "explicit_terminal_trim",
            "trim_5p_bases": 2,
            "trim_3p_bases": 2,
            "requested_trim_5p_bases": 2,
            "requested_trim_3p_bases": 2,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "adapterremoval",
                "raw_backend_report_format": None,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
            {
                "sample_id": "sample_0001",
                "tool": "cutadapt",
                "raw_backend_report_format": "cutadapt_json",
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 1,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
            {
                "sample_id": "sample_0001",
                "tool": "seqkit",
                "raw_backend_report_format": None,
                "damage_mode": "ancient",
                "execution_policy": "explicit_terminal_trim",
                "trim_5p_bases": 2,
                "trim_3p_bases": 2,
                "requested_trim_5p_bases": 2,
                "requested_trim_3p_bases": 2,
            },
        ]

        with self.assertRaises(SystemExit):
            terminal_damage_report.validate_terminal_damage_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )

    def test_terminal_damage_report_rejects_dry_run_manifest(self) -> None:
        with self.assertRaises(SystemExit):
            terminal_damage_report.validate_terminal_damage_run_manifest_contract(
                {
                    "stage_id": "fastq.trim_terminal_damage",
                    "scenario_id": "terminal_damage_fairness",
                    "tool_kind": "benchmark",
                    "dry_run": True,
                }
            )

    def test_trim_polyg_report_contract_rejects_missing_tool_rows(self) -> None:
        run_manifest = {
            "tools": ["fastp", "bbduk"],
            "polyx_preset": "illumina_twocolor",
            "min_polyg_run": 10,
            "trim_polyg": True,
        }
        sample_rows = [
            {
                "sample_id": "sample_0001",
                "tool": "fastp",
                "raw_backend_report_format": "fastp_json",
                "polyx_preset": "illumina_twocolor",
                "min_polyg_run": 10,
                "trim_polyg": True,
            }
        ]

        with self.assertRaises(SystemExit):
            trim_polyg_report.validate_trim_polyg_row_contract(
                run_manifest=run_manifest,
                sample_rows=sample_rows,
            )


if __name__ == "__main__":
    unittest.main()
