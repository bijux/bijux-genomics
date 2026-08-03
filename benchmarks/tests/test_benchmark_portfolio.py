from __future__ import annotations

import copy
import importlib.util
import sys
import unittest
from pathlib import Path
from unittest import mock

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
        self.interfaces = benchmark_portfolio.load_toml(
            ROOT / "benchmarks/contracts/stage-interfaces.toml"
        )

    def validate(self, portfolio: dict | None = None) -> dict:
        return benchmark_portfolio.validate_portfolio(
            portfolio or self.portfolio, self.catalog, ROOT
        )

    def validated_interfaces(self, validated: dict | None = None) -> dict:
        portfolio = validated or self.validate()
        return benchmark_portfolio.validate_interfaces(
            self.interfaces, portfolio["stage_tools"], ROOT
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
        report = benchmark_portfolio.readiness_report(
            self.validate(), self.registry, self.validated_interfaces()
        )

        self.assertEqual(report["summary"]["stage_count"], 71)
        self.assertEqual(report["summary"]["ready_count"], 0)
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
        self.assertEqual(profile_reads["interface_status"], "closed")
        self.assertEqual(profile_reads["readiness"], "blocked")
        self.assertIn(
            "no admitted comparative execution evidence", profile_reads["blockers"]
        )

    def test_stage_interface_closes_schemas_adapters_defaults_and_consumer(self) -> None:
        interfaces = self.validated_interfaces()
        profile_reads = interfaces["fastq.profile_reads"]

        self.assertEqual(
            set(profile_reads["adapters"]), {"seqfu", "seqkit", "seqkit_stats"}
        )
        self.assertEqual(
            profile_reads["parameter_schema_version"],
            "bijux.fastq.params.stats.v1",
        )
        self.assertEqual(
            profile_reads["downstream_probe"]["consumer_stage_id"],
            "fastq.report_qc",
        )
        self.assertEqual(profile_reads["operator_override_keys"], ["threads"])
        self.assertEqual(profile_reads["input_derived_parameters"], ["paired_mode"])
        self.assertEqual(
            profile_reads["effective_parameters_artifact"],
            "effective-parameters.json",
        )

    def test_input_derived_parameters_cannot_be_operator_overrides(self) -> None:
        interfaces = copy.deepcopy(self.interfaces)
        interfaces["interfaces"][0]["operator_override_keys"].append("paired_mode")

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError, "parameter ownership overlaps"
        ):
            benchmark_portfolio.validate_interfaces(
                interfaces, self.validate()["stage_tools"], ROOT
            )

    def test_unknown_parameter_override_is_rejected(self) -> None:
        interfaces = copy.deepcopy(self.interfaces)
        original_loader = benchmark_portfolio.load_yaml

        def load_with_unknown_override(path: Path) -> dict[str, object]:
            value = original_loader(path)
            value["stages"]["fastq.profile_reads"]["overrides"]["paired_mode"] = (
                "single_end"
            )
            return value

        with (
            mock.patch.object(
                benchmark_portfolio, "load_yaml", side_effect=load_with_unknown_override
            ),
            self.assertRaisesRegex(
                benchmark_portfolio.PortfolioError, "unsupported overrides"
            ),
        ):
            benchmark_portfolio.validate_interfaces(
                interfaces, self.validate()["stage_tools"], ROOT
            )

    def test_partial_tool_adapter_closure_is_rejected(self) -> None:
        interfaces = copy.deepcopy(self.interfaces)
        del interfaces["interfaces"][0]["adapters"]["seqfu"]

        with self.assertRaisesRegex(
            benchmark_portfolio.PortfolioError, "exact tool closure"
        ):
            benchmark_portfolio.validate_interfaces(
                interfaces, self.validate()["stage_tools"], ROOT
            )

    def test_taxonomy_stage_names_real_edna_and_database_blockers(self) -> None:
        report = benchmark_portfolio.readiness_report(
            self.validate(), self.registry, self.validated_interfaces()
        )
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

        report = benchmark_portfolio.readiness_report(
            self.validate(), registry, self.validated_interfaces()
        )
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
