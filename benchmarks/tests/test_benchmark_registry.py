from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "benchmark_registry", BENCHMARKS / "benchmark_registry.py"
)
assert SPEC is not None and SPEC.loader is not None
benchmark_registry = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = benchmark_registry
SPEC.loader.exec_module(benchmark_registry)

ROOT = Path(__file__).parents[2]


class BenchmarkRegistryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = benchmark_registry.load_toml(
            ROOT / "containers/contracts/catalog.toml"
        )
        self.registry = benchmark_registry.load_toml(
            ROOT / "benchmarks/campaigns.toml"
        )

    def test_campaigns_cover_every_retained_tool_for_the_stage(self) -> None:
        campaigns = benchmark_registry.validate_registry(self.registry, self.catalog)

        self.assertEqual(campaigns[0]["stage_id"], "bam.genotyping")
        self.assertEqual(campaigns[0]["evidence_tier"], "qualification")
        self.assertEqual(campaigns[0]["tool_ids"], ["bcftools"])
        self.assertEqual(campaigns[1]["stage_id"], "bam.coverage")
        self.assertEqual(campaigns[1]["tool_ids"], ["bedtools", "mosdepth", "samtools"])
        self.assertEqual(
            benchmark_registry.threads_by_tool(campaigns[1]),
            {"bedtools": 1, "mosdepth": 4, "samtools": 1},
        )
        self.assertEqual(campaigns[2]["stage_id"], "bam.endogenous_content")
        self.assertEqual(campaigns[2]["tool_ids"], ["samtools"])
        self.assertEqual(campaigns[3]["stage_id"], "bam.mapping_summary")
        self.assertEqual(campaigns[3]["tool_ids"], ["picard", "samtools"])
        self.assertEqual(
            benchmark_registry.threads_by_tool(campaigns[3]),
            {"picard": 1, "samtools": 1},
        )

    def test_per_tool_threads_must_cover_the_tool_closure(self) -> None:
        self.registry["campaigns"][1]["threads_by_tool"].pop("samtools")

        with self.assertRaisesRegex(
            benchmark_registry.RegistryError,
            "threads_by_tool must cover the exact tool closure",
        ):
            benchmark_registry.validate_registry(self.registry, self.catalog)

    def test_reference_free_summary_campaign_is_valid(self) -> None:
        campaigns = benchmark_registry.validate_registry(self.registry, self.catalog)

        self.assertEqual(campaigns[3]["reference_catalog_ids"], [])

    def test_omitting_a_retained_tool_is_rejected(self) -> None:
        self.registry["campaigns"][0]["tool_ids"] = []

        with self.assertRaisesRegex(
            benchmark_registry.RegistryError,
            "tool_ids must be a non-empty text list",
        ):
            benchmark_registry.validate_registry(self.registry, self.catalog)

    def test_result_artifact_cannot_escape_a_repetition_directory(self) -> None:
        self.registry["campaigns"][0]["result_artifact"] = "../result.tsv"

        with self.assertRaisesRegex(
            benchmark_registry.RegistryError,
            "result_artifact must be a safe filename",
        ):
            benchmark_registry.validate_registry(self.registry, self.catalog)

    def test_multi_tool_campaign_requires_comparison_metrics(self) -> None:
        self.registry["campaigns"][1]["comparison_metrics"] = []

        with self.assertRaisesRegex(
            benchmark_registry.RegistryError,
            "multi-tool benchmarks require predeclared comparison metrics",
        ):
            benchmark_registry.validate_registry(self.registry, self.catalog)

    def test_coverage_does_not_claim_missing_campaigns_are_defined(self) -> None:
        campaigns = benchmark_registry.validate_registry(self.registry, self.catalog)
        report = benchmark_registry.coverage_report(campaigns, self.catalog)

        self.assertEqual(report["summary"]["defined_stage_count"], 4)
        self.assertEqual(report["summary"]["qualification_stage_count"], 4)
        self.assertEqual(report["summary"]["comparative_stage_count"], 0)
        self.assertEqual(report["summary"]["admitted_stage_count"], 0)
        self.assertGreater(report["summary"]["missing_stage_count"], 1)

    def test_coverage_admits_matching_complete_tool_evidence(self) -> None:
        campaigns = benchmark_registry.validate_registry(self.registry, self.catalog)
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "run-2001-bam-genotyping.json").write_text(
                json.dumps(
                    {
                        "schema_version": "bijux.container.stage_benchmark_report.v1",
                        "verdict": "passed",
                        "run_id": "run-2001",
                        "stage_id": "bam.genotyping",
                        "tools": [{"tool": "bcftools"}],
                    }
                ),
                encoding="utf-8",
            )

            report = benchmark_registry.coverage_report(campaigns, self.catalog, root)

        row = next(
            row for row in report["stages"] if row["stage_id"] == "bam.genotyping"
        )
        self.assertEqual(report["summary"]["admitted_stage_count"], 1)
        self.assertEqual(row["execution_status"], "admitted")


if __name__ == "__main__":
    unittest.main()
