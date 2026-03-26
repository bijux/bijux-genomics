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


if __name__ == "__main__":
    unittest.main()
