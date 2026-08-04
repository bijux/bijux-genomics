from __future__ import annotations

import hashlib
import importlib.util
import json
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
    def test_repetition_science_metrics_supports_fastq_bundle_artifacts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repetitions = Path(directory) / "outputs/repetitions"
            repetitions.mkdir(parents=True)
            histogram = repetitions / "repetition-1.sample-a.histogram.tsv"
            histogram.write_text(
                "sample_id\tread_length\tcount\n"
                "sample-a\t4\t1\n"
                "sample-a\t6\t1\n",
                encoding="utf-8",
            )
            artifact = repetitions / "repetition-1.read-lengths.json"
            artifact.write_text(
                json.dumps(
                    {
                        "schema_version": "bijux.benchmark.fastq.profile_read_lengths.bundle.v1",
                        "stage_id": "fastq.profile_read_lengths",
                        "tool_id": "seqfu",
                        "results": [
                            {
                                "sample_id": "sample-a",
                                "input_layout": "single_end",
                                "file_count": 1,
                                "read_count": 2,
                                "minimum_read_length": 4,
                                "mean_read_length": 5.0,
                                "median_read_length": 5.0,
                                "maximum_read_length": 6,
                                "distinct_read_lengths": 2,
                                "histogram_artifact": "repetitions/repetition-1.sample-a.histogram.tsv",
                                "histogram_sha256": hashlib.sha256(
                                    histogram.read_bytes()
                                ).hexdigest(),
                            }
                        ],
                    },
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )
            (repetitions / "repetition-1.metrics.json").write_text(
                json.dumps(
                    {
                        "schema_version": "bijux.benchmark.fastq.profile_read_lengths.metrics.v1",
                        "sample_count": 1,
                        "file_count": 1,
                        "read_count": 2,
                        "minimum_read_length": 4,
                        "maximum_read_length": 6,
                        "exact_observation_mismatch_count": 0,
                        "histogram_mismatch_count": 0,
                        "input_bytes": 12,
                        "output_bytes": 34,
                    },
                    indent=2,
                    sort_keys=True,
                )
                + "\n",
                encoding="utf-8",
            )

            metrics = benchmark_evidence.repetition_science_metrics(
                repetitions / "repetition-1",
                {
                    "result_artifact": "read-lengths.json",
                    "scientific_metrics": [
                        "file_count",
                        "read_count",
                        "minimum_read_length",
                        "mean_read_length",
                        "median_read_length",
                        "maximum_read_length",
                        "distinct_read_lengths",
                        "histogram_sha256",
                    ],
                },
            )

        self.assertEqual(metrics["file_count"], 1)
        self.assertEqual(metrics["read_count"], 2)
        self.assertEqual(metrics["mean_read_length"], 5.0)
        self.assertEqual(metrics["median_read_length"], 5.0)
        self.assertEqual(metrics["distinct_read_lengths"], 2)
        self.assertEqual(len(metrics["histogram_sha256"]), 64)

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

    def test_governed_resource_tsv_preserves_wait4_metrics(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "resource.tsv"
            path.write_text(
                "metric\tvalue\tunit\n"
                "wall_seconds\t1.2\tseconds\n"
                "user_seconds\t1.0\tseconds\n"
                "system_seconds\t0.1\tseconds\n"
                "peak_rss_kib\t2048\tkibibytes\n"
                "filesystem_input_operations\t12\toperations\n"
                "filesystem_output_operations\t3\toperations\n",
                encoding="utf-8",
            )

            values = benchmark_evidence.read_resources(path)

        self.assertEqual(values["filesystem_input_operations"], 12.0)
        self.assertEqual(values["filesystem_output_operations"], 3.0)

    def test_governed_resource_tsv_rejects_invented_byte_units(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "resource.tsv"
            path.write_text(
                "metric\tvalue\tunit\nwall_seconds\t1.2\tbytes\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                benchmark_evidence.BenchmarkEvidenceError,
                "invalid resource TSV unit",
            ):
                benchmark_evidence.read_resources(path)

    def test_comparability_rejects_mixed_repository_commits(self) -> None:
        shared = {
            "input_locks": [{"id": "reads", "sha256": "b" * 64}],
            "parameters": {"effective_sha256": "c" * 64},
            "interface": {
                "accepted_input_contracts": ["fastq.single.v1"],
                "output_contract": "profile.v1",
                "normalized_result_contract": "schema.json",
                "downstream_probe": {"stage_id": "consumer"},
            },
        }
        tools = [
            {"tool": "first", "repository_commit": "a" * 40, **shared},
            {"tool": "second", "repository_commit": "d" * 40, **shared},
        ]

        with self.assertRaisesRegex(
            benchmark_evidence.BenchmarkEvidenceError,
            "one repository commit",
        ):
            benchmark_evidence.validate_comparability(tools)

    def test_markdown_does_not_overclaim_pipeline_substitution(self) -> None:
        report = {
            "stage_id": "fastq.profile_reads",
            "run_id": "run-2043",
            "evidence_tier": "comparative",
            "benchmark_profile_id": "fastq-observational-qc",
            "verdict": "passed",
            "interpretation": "Comparison parsing passed.",
            "scientific_question": "Do normalized observations agree?",
            "comparison_scope": "one governed cohort",
            "accuracy_claim": "count_concordance",
            "design": {
                "tool_closure": ["seqkit"],
                "runtime": "apptainer",
                "scheduler": "slurm",
                "threads_by_tool": {"seqkit": 4},
                "cache_policy": "warm",
                "warmup_repetitions": 1,
                "measured_repetitions": 3,
                "corpus_catalog_ids": ["corpus"],
                "corpus_ids": ["corpus"],
                "reference_catalog_ids": [],
                "database_catalog_ids": [],
                "result_artifact": "profile.tsv",
            },
            "comparability": {
                "repository_commit": "a" * 40,
                "input_lock_sha256": "b" * 64,
                "parameter_lock_sha256": "c" * 64,
                "stage_interface": {
                    "accepted_input_contracts": ["fastq.single.v1"],
                    "output_contract": "benchmark.profile.v1",
                    "normalized_result_contract": "profile.schema.json",
                    "downstream_probe": {"stage_id": "benchmark.compare"},
                },
            },
            "tools": [
                {
                    "tool": "seqkit",
                    "repository_commit": "a" * 40,
                    "image": {"sha256": "d" * 64},
                    "slurm": {"job_id": "123"},
                    "performance": {
                        key: {"mean": 1.0, "minimum": 1.0, "maximum": 1.0}
                        for key in (
                            "wall_seconds",
                            "cpu_seconds",
                            "peak_rss_kib",
                            "output_bytes",
                        )
                    },
                    "scientific_observations": {"read_count": [1, 1, 1]},
                    "determinism": {
                        "artifact": "profile.tsv",
                        "sha256": "e" * 64,
                    },
                }
            ],
            "cross_tool_comparison": {"status": "not_applicable", "metrics": []},
            "limitations": ["No transformation output was exercised."],
        }

        markdown = benchmark_evidence.render_markdown(report)

        self.assertIn("normalized benchmark comparison only", markdown)
        self.assertIn(
            "does not claim operational downstream pipeline substitution", markdown
        )

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
            "evidence_tier": "qualification",
            "benchmark_profile_id": "bam-biological-inference",
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
                "corpus_ids": ["ancient-horse-alignment-qualification"],
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

        report["design"]["reference_catalog_ids"] = []
        without_reference = benchmark_evidence.render_markdown(report)
        self.assertIn("- Reference: none\n", without_reference)
        self.assertFalse(
            any(line.endswith(" ") for line in without_reference.splitlines())
        )

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
