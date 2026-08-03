from __future__ import annotations

import hashlib
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
sys.path.insert(0, str(BENCHMARKS))
SPEC = importlib.util.spec_from_file_location(
    "benchmark_evidence", BENCHMARKS / "benchmark_evidence.py"
)
assert SPEC is not None and SPEC.loader is not None
benchmark_evidence = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(benchmark_evidence)


class BenchmarkEvidenceTests(unittest.TestCase):
    def test_resource_tsv_requires_real_columns(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "resource.tsv"
            path.write_text(
                "wall_seconds\t1.2\n"
                "user_seconds\t1.0\n"
                "system_seconds\t0.1\n"
                "peak_rss_kib\t2048\n",
                encoding="utf-8",
            )

            values = benchmark_evidence.read_resources(path)

        self.assertEqual(values["wall_seconds"], 1.2)
        self.assertEqual(values["peak_rss_kib"], 2048.0)

    def test_literal_separator_resource_line_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "resource.tsv"
            path.write_text(
                "wall_secondst1.2tuser_secondst1.0tsystem_secondst0.1tpeak_rss_kibt2048\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                benchmark_evidence.BenchmarkEvidenceError,
                "invalid resource TSV row",
            ):
                benchmark_evidence.read_resources(path)

    def test_output_inventory_rejects_unrecorded_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            output = root / "outputs/recorded.txt"
            output.parent.mkdir()
            output.write_text("recorded\n", encoding="utf-8")
            (root / "outputs/unrecorded.txt").write_text(
                "unrecorded\n", encoding="utf-8"
            )
            evidence = {
                "outputs": [
                    {
                        "path": "outputs/recorded.txt",
                        "size_bytes": output.stat().st_size,
                        "sha256": hashlib.sha256(output.read_bytes()).hexdigest(),
                    }
                ]
            }

            with self.assertRaisesRegex(
                benchmark_evidence.BenchmarkEvidenceError,
                "output inventory is incomplete",
            ):
                benchmark_evidence.verify_recorded_outputs(root, evidence)

    def test_markdown_states_accuracy_boundary(self) -> None:
        report = {
            "stage_id": "bam.genotyping",
            "run_id": "run-2001",
            "verdict": "passed",
            "interpretation": "Candidate-site recovery is not genotype accuracy.",
            "scientific_question": "Can the governed execution recover candidate sites?",
            "comparison_scope": "one interval",
            "accuracy_claim": "candidate_site_recovery_not_genotype_truth",
            "design": {
                "tool_closure": ["bcftools"],
                "runtime": "apptainer",
                "scheduler": "slurm",
                "threads_by_tool": {"bcftools": 4},
                "cache_policy": "warm",
                "warmup_repetitions": 1,
                "measured_repetitions": 3,
                "corpus_catalog_ids": ["corpus"],
                "reference_catalog_ids": ["reference"],
                "database_catalog_ids": ["database"],
                "result_artifact": "genotypes.tsv",
            },
            "tools": [
                {
                    "tool": "bcftools",
                    "repository_commit": "a" * 40,
                    "image": {"sha256": "b" * 64},
                    "slurm": {"job_id": "123"},
                    "performance": {
                        key: {
                            "mean": 1.0,
                            "minimum": 1.0,
                            "maximum": 1.0,
                        }
                        for key in (
                            "wall_seconds",
                            "cpu_seconds",
                            "peak_rss_kib",
                            "output_bytes",
                        )
                    },
                    "scientific_observations": {"covered_sites": [21, 21, 21]},
                    "determinism": {
                        "artifact": "genotypes.tsv",
                        "sha256": "c" * 64,
                    },
                }
            ],
            "cross_tool_comparison": {"status": "not_applicable", "metrics": []},
            "limitations": ["No sample-level truth genotype is available."],
        }

        markdown = benchmark_evidence.render_markdown(report)

        self.assertIn("not genotype accuracy", markdown)
        self.assertIn("candidate_site_recovery_not_genotype_truth", markdown)
        self.assertIn("No sample-level truth genotype", markdown)

    def test_cross_tool_comparison_preserves_disagreement(self) -> None:
        campaign = {
            "comparison_metrics": [
                {
                    "id": "mean_depth",
                    "absolute_tolerance": 0.0,
                    "relative_tolerance": 0.05,
                }
            ]
        }
        tools = [
            {
                "tool": "first",
                "scientific_observations": {"mean_depth": [1.0, 1.0, 1.0]},
            },
            {
                "tool": "second",
                "scientific_observations": {"mean_depth": [1.2, 1.2, 1.2]},
            },
        ]

        comparison = benchmark_evidence.compare_tools(campaign, tools)

        self.assertEqual(comparison["status"], "outside_tolerance")
        metric = comparison["metrics"][0]
        self.assertAlmostEqual(metric["absolute_range"], 0.2)
        self.assertAlmostEqual(metric["allowed_range"], 0.06)
        self.assertFalse(metric["within_tolerance"])
        self.assertEqual(benchmark_evidence.comparison_verdict(comparison), "failed")

    def test_within_tolerance_comparison_is_admissible(self) -> None:
        self.assertEqual(
            benchmark_evidence.comparison_verdict(
                {"status": "within_tolerance", "metrics": []}
            ),
            "passed",
        )


if __name__ == "__main__":
    unittest.main()
