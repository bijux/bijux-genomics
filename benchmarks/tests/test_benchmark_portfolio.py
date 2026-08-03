from __future__ import annotations

import copy
import importlib.util
import sys
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "benchmark_portfolio", BENCHMARKS / "benchmark_portfolio.py"
)
assert SPEC is not None and SPEC.loader is not None
benchmark_portfolio = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = benchmark_portfolio
SPEC.loader.exec_module(benchmark_portfolio)

ROOT = Path(__file__).parents[2]


class BenchmarkPortfolioTests(unittest.TestCase):
    def setUp(self) -> None:
        self.portfolio = benchmark_portfolio.load_toml(
            ROOT / "benchmarks/contracts/portfolio.toml"
        )
        self.catalog = benchmark_portfolio.load_toml(
            ROOT / "containers/contracts/catalog.toml"
        )
        self.registry = benchmark_portfolio.load_toml(
            ROOT / "benchmarks/contracts/stage-campaigns.toml"
        )

    def validate(self, portfolio: dict | None = None) -> dict:
        return benchmark_portfolio.validate_portfolio(
            portfolio or self.portfolio, self.catalog, ROOT
        )

    def test_every_retained_stage_has_one_scientific_profile(self) -> None:
        validated = self.validate()

        self.assertEqual(len(validated["stage_tools"]), 71)
        self.assertEqual(set(validated["stage_tools"]), set(validated["assignments"]))
        self.assertEqual(set(validated["stage_tools"]), set(validated["stage_runs"]))
        self.assertEqual(
            validated["assignments"]["fastq.screen_taxonomy"], "fastq-taxonomy"
        )
        self.assertEqual(validated["assignments"]["bam.align"], "bam-alignment")
        self.assertEqual(
            validated["assignments"]["vcf.impute"], "vcf-phasing-imputation"
        )
        self.assertEqual(validated["stage_runs"]["fastq.screen_taxonomy"], "run-2047")
        self.assertEqual(validated["stage_runs"]["vcf.impute"], "run-2063")

    def test_stage_run_identity_must_be_unique_and_complete(self) -> None:
        portfolio = copy.deepcopy(self.portfolio)
        portfolio["stage_runs"]["vcf.impute"] = portfolio["stage_runs"]["vcf.qc"]

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError,
            "assigned more than once",
        ):
            self.validate(portfolio)

    def test_missing_stage_assignment_is_rejected(self) -> None:
        portfolio = copy.deepcopy(self.portfolio)
        portfolio["assignments"][0]["stage_ids"].remove("fastq.validate_reads")

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError, "profile closure failed"
        ):
            self.validate(portfolio)

    def test_comparative_profiles_require_real_cohort_scale(self) -> None:
        portfolio = copy.deepcopy(self.portfolio)
        portfolio["profiles"][0]["minimum_real_samples"] = 1

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError, "at least ten real samples"
        ):
            self.validate(portfolio)

    def test_measurement_contract_requires_io_cpu_and_throughput(self) -> None:
        portfolio = copy.deepcopy(self.portfolio)
        portfolio["measurement"]["required_resource_metrics"].remove(
            "filesystem_read_bytes"
        )

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError, "omits required resource metrics"
        ):
            self.validate(portfolio)

    def test_qualification_runs_do_not_count_as_scientific_readiness(self) -> None:
        report = benchmark_portfolio.readiness_report(self.validate(), self.registry)

        self.assertEqual(report["summary"]["stage_count"], 71)
        self.assertEqual(report["summary"]["ready_count"], 1)
        self.assertEqual(report["summary"]["qualification_campaign_count"], 4)
        self.assertEqual(report["summary"]["comparative_campaign_count"], 1)
        row = next(
            row for row in report["stages"] if row["stage_id"] == "bam.coverage"
        )
        self.assertEqual(row["campaign_tier"], "qualification")
        self.assertIn("active campaign is qualification-only", row["blockers"])
        profile_reads = next(
            row
            for row in report["stages"]
            if row["stage_id"] == "fastq.profile_reads"
        )
        self.assertEqual(profile_reads["readiness"], "ready")

    def test_taxonomy_stage_names_real_edna_and_database_blockers(self) -> None:
        report = benchmark_portfolio.readiness_report(self.validate(), self.registry)
        row = next(
            row
            for row in report["stages"]
            if row["stage_id"] == "fastq.screen_taxonomy"
        )

        self.assertEqual(
            row["retained_tools"], ["centrifuge", "kaiju", "kraken2", "krakenuniq"]
        )
        self.assertEqual(row["profile_id"], "fastq-taxonomy")
        self.assertTrue(
            any("environmental_taxonomy_truth" in item for item in row["blockers"])
        )
        profile = self.validate()["profiles"][row["profile_id"]]
        self.assertIn("lineage", profile["downstream_probe"])

    def test_comparative_campaign_must_bind_ready_corpora_and_all_roles(self) -> None:
        registry = copy.deepcopy(self.registry)
        campaign = next(
            value
            for value in registry["campaigns"]
            if value["stage_id"] == "bam.coverage"
        )
        campaign["evidence_tier"] = "comparative"

        report = benchmark_portfolio.readiness_report(self.validate(), registry)
        row = next(
            value for value in report["stages"] if value["stage_id"] == "bam.coverage"
        )

        self.assertIn(
            "campaign corpora are not comparative-ready: "
            "['ancient-horse-alignment-qualification']",
            row["blockers"],
        )
        self.assertIn(
            "campaign corpus portfolio omits roles ['bam_diversity']",
            row["blockers"],
        )


if __name__ == "__main__":
    unittest.main()
