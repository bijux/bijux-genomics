from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import unittest

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "benchmark_registry", BENCHMARKS / "benchmark_registry.py"
)
assert SPEC is not None and SPEC.loader is not None
benchmark_registry = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = benchmark_registry
SPEC.loader.exec_module(benchmark_registry)

ROOT = Path(__file__).parents[3]


class BenchmarkRegistryTests(unittest.TestCase):
    def setUp(self) -> None:
        self.catalog = benchmark_registry.load_toml(
            ROOT / "containers/contracts/catalog.toml"
        )
        self.registry = benchmark_registry.load_toml(
            ROOT / "containers/benchmarks/campaigns.toml"
        )

    def test_campaigns_cover_every_retained_tool_for_the_stage(self) -> None:
        campaigns = benchmark_registry.validate_registry(
            self.registry, self.catalog
        )

        self.assertEqual(campaigns[0]["stage_id"], "bam.genotyping")
        self.assertEqual(campaigns[0]["tool_ids"], ["bcftools"])

    def test_omitting_a_retained_tool_is_rejected(self) -> None:
        self.registry["campaigns"][0]["tool_ids"] = []

        with self.assertRaisesRegex(
            benchmark_registry.RegistryError,
            "tool_ids must be a non-empty text list",
        ):
            benchmark_registry.validate_registry(self.registry, self.catalog)

    def test_coverage_does_not_claim_missing_campaigns_are_defined(self) -> None:
        campaigns = benchmark_registry.validate_registry(
            self.registry, self.catalog
        )
        report = benchmark_registry.coverage_report(campaigns, self.catalog)

        self.assertEqual(report["summary"]["defined_stage_count"], 1)
        self.assertGreater(report["summary"]["missing_stage_count"], 1)


if __name__ == "__main__":
    unittest.main()
