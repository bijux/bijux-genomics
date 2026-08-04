from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

CONTRACTS = Path(__file__).parents[1]
sys.path.insert(0, str(CONTRACTS))
SCRIPT = CONTRACTS / "image_campaign.py"
SPEC = importlib.util.spec_from_file_location("container_image_campaign", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
campaign = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(campaign)


class ImageCampaignTests(unittest.TestCase):
    def test_operator_docs_do_not_advertise_retired_build_commands(self) -> None:
        repository = CONTRACTS.parents[1]
        for relative_path in (
            "containers/README.md",
            "containers/docs/LUNARC_EXECUTION_CONTRACT.md",
        ):
            content = (repository / relative_path).read_text(encoding="utf-8")
            self.assertNotIn(
                "python3 -B containers/contracts/image_campaign.py",
                content,
            )
            self.assertNotIn(
                "python3 containers/contracts/container_build.py",
                content,
            )

    def test_image_source_roots_follow_definition_file_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            definition = repository / "containers/apptainer/bijux_dna.def"
            definition.parent.mkdir(parents=True)
            (repository / "containers/tools/bijux_dna").mkdir(parents=True)
            (repository / "crates").mkdir()
            (repository / "Cargo.toml").write_text("[workspace]\n")
            definition.write_text(
                "Bootstrap: docker\n\n"
                "%files\n"
                "    containers/tools/bijux_dna /opt/tools\n"
                "    crates /opt/source/crates\n"
                "    Cargo.toml /opt/source/Cargo.toml\n"
                "%post\n"
                "    true\n",
                encoding="utf-8",
            )

            roots = campaign.image_source_roots(repository, ["bijux_dna"])

        self.assertEqual(roots, ("containers", "crates"))

    def test_image_source_roots_reject_missing_sources(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            definition = repository / "containers/apptainer/samtools.def"
            definition.parent.mkdir(parents=True)
            definition.write_text(
                "Bootstrap: docker\n\n%files\n    missing/tool /opt/tool\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(campaign.ContractError, "source is absent"):
                campaign.image_source_roots(repository, ["samtools"])

    def test_python_image_build_fails_closed(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository = root / "repository"
            definition = repository / "containers/apptainer/samtools.def"
            definition.parent.mkdir(parents=True)
            definition.write_text("Bootstrap: docker\n", encoding="utf-8")
            image_root = root / "images"
            evidence_root = root / "evidence"
            with mock.patch.object(
                campaign.subprocess,
                "run",
                return_value=mock.Mock(returncode=1),
            ) as run:
                with self.assertRaisesRegex(
                    campaign.ContractError,
                    "Python image campaign execution is retired",
                ):
                    campaign.build_one(
                        "samtools",
                        "1.20",
                        "a" * 40,
                        root / "governance.yml",
                        repository,
                        image_root,
                        evidence_root,
                        1,
                    )

        run.assert_not_called()

    def test_python_campaign_execution_fails_before_state_mutation(self) -> None:
        with self.assertRaisesRegex(
            campaign.ContractError,
            "Python image campaign execution is retired",
        ):
            campaign.execute_campaign(Path("missing.json"), Path("missing.yml"), 2)

    def test_python_campaign_preparation_fails_before_state_creation(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / "execution.json"
            with self.assertRaisesRegex(
                campaign.ContractError,
                "Python image campaign execution is retired",
            ):
                campaign.create_execution(
                    Path("campaign.json"),
                    Path("inventory.json"),
                    Path("catalog.toml"),
                    Path("governance.yml"),
                    None,
                    output,
                )
            self.assertFalse(output.exists())

    def test_selection_contains_only_supported_noncurrent_images(self) -> None:
        selected = campaign.selected_tools(
            {
                "schema_version": campaign.INVENTORY_SCHEMA,
                "tools": [
                    {
                        "tool": "samtools",
                        "apptainer_supported": True,
                        "state": "current",
                    },
                    {
                        "tool": "bcftools",
                        "apptainer_supported": True,
                        "state": "stale_recipe",
                    },
                    {
                        "tool": "fastqc",
                        "apptainer_supported": True,
                        "state": "missing",
                    },
                    {
                        "tool": "impute5",
                        "apptainer_supported": False,
                        "state": "unsupported",
                    },
                ],
            },
            "catalog_apptainer_supported_noncurrent",
        )

        self.assertEqual(selected, ["bcftools", "fastqc"])

    def test_complete_selection_includes_current_supported_images(self) -> None:
        selected = campaign.selected_tools(
            {
                "schema_version": campaign.INVENTORY_SCHEMA,
                "tools": [
                    {
                        "tool": "samtools",
                        "apptainer_supported": True,
                        "state": "current",
                    },
                    {
                        "tool": "impute5",
                        "apptainer_supported": False,
                        "state": "unsupported",
                    },
                ],
            },
            "catalog_apptainer_supported_all",
        )

        self.assertEqual(selected, ["samtools"])

    def test_targeted_selection_is_a_subset_of_inventory_eligibility(self) -> None:
        self.assertEqual(
            campaign.requested_tool_selection(
                ["bijux_dna", "picard"],
                ["picard"],
            ),
            ["picard"],
        )
        with self.assertRaisesRegex(campaign.ContractError, "current or unsupported"):
            campaign.requested_tool_selection(["picard"], ["samtools"])

    def test_isolated_admission_keeps_images_and_evidence_together(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            containers_root = Path(directory) / "containers"
            image_root = (
                containers_root / "rebuild-verification" / "commit" / "apptainer"
            )

            admitted, evidence = campaign.admission_roots(containers_root, image_root)

        self.assertEqual(admitted, image_root.resolve())
        self.assertEqual(
            evidence,
            (
                containers_root / "rebuild-verification" / "commit" / "evidence"
            ).resolve(),
        )

    def test_admission_outside_governed_containers_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with self.assertRaisesRegex(campaign.ContractError, "must remain under"):
                campaign.admission_roots(root / "containers", root / "outside")

    def test_campaign_requires_fixed_frontend_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "campaign.json"
            path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_SCHEMA,
                        "run_id": "run-1000",
                        "runtime": "apptainer",
                        "site": "lunarc",
                        "selection": "catalog_apptainer_supported_noncurrent",
                    }
                ),
                encoding="utf-8",
            )

            loaded = campaign.load_campaign(path)

        self.assertEqual(loaded["run_id"], "run-1000")

    def test_other_run_identity_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "campaign.json"
            path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_SCHEMA,
                        "run_id": "run-1001",
                        "runtime": "apptainer",
                        "site": "lunarc",
                        "selection": "catalog_apptainer_supported_noncurrent",
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(campaign.ContractError, "run-1000"):
                campaign.load_campaign(path)

    def test_python_campaign_retry_cannot_mutate_attempts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            execution_path = Path(directory) / "execution.json"
            execution_path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_EVIDENCE_SCHEMA,
                        "status": "failed",
                        "selected_tools": ["addeam", "fastp"],
                        "tools": {
                            "addeam": {
                                "version": "1.0.0",
                                "definition": "containers/apptainer/addeam.def",
                                "status": "failed",
                                "exit_code": 255,
                                "build_log": "/evidence/addeam.log",
                                "build_log_sha256": "a" * 64,
                            },
                            "fastp": {
                                "version": "0.23.4",
                                "definition": "containers/apptainer/fastp.def",
                                "status": "passed",
                                "attempts": [{"attempt": 1, "status": "passed"}],
                            },
                        },
                        "summary": {"failed": 1, "passed": 1},
                        "finished_at_utc": "2026-08-03T00:00:00+00:00",
                    }
                ),
                encoding="utf-8",
            )

            before = execution_path.read_bytes()
            with self.assertRaisesRegex(
                campaign.ContractError,
                "Python image campaign execution is retired",
            ):
                campaign.queue_failed_tools(execution_path, ["addeam"])

            self.assertEqual(execution_path.read_bytes(), before)

    def test_python_campaign_retry_fails_before_reading_record(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            execution_path = Path(directory) / "execution.json"
            execution_path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_EVIDENCE_SCHEMA,
                        "status": "passed",
                        "selected_tools": ["fastp"],
                        "tools": {"fastp": {"status": "passed", "attempts": []}},
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                campaign.ContractError,
                "Python image campaign execution is retired",
            ):
                campaign.queue_failed_tools(execution_path, ["fastp"])

    def test_campaign_without_pending_work_still_fails_if_a_tool_failed(self) -> None:
        summary, status = campaign.summarize_tool_states(
            ["addeam", "fastp"],
            {"addeam": {"status": "failed"}, "fastp": {"status": "passed"}},
        )

        self.assertEqual(summary, {"failed": 1, "passed": 1})
        self.assertEqual(status, "failed")

    def test_interrupted_attempt_uses_a_new_output_identity(self) -> None:
        tool = {"status": "pending", "attempt": 1, "attempts": []}

        self.assertEqual(campaign.next_attempt_number(tool), 2)

    def test_rebuild_report_records_retry_without_expanding_science_claim(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            execution_path = root / "execution.json"
            inventory_path = root / "after-build.json"
            execution_path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.CAMPAIGN_EVIDENCE_SCHEMA,
                        "run_id": "run-1000",
                        "status": "passed",
                        "site": {"id": "lunarc", "host": "cosmos3"},
                        "repository_commit": "a" * 40,
                        "finished_at_utc": "2026-08-03T00:00:00+00:00",
                        "selected_tools": ["addeam"],
                        "summary": {"passed": 1},
                        "tools": {
                            "addeam": {
                                "version": "1.0.0",
                                "status": "passed",
                                "attempt": 2,
                                "attempts": [
                                    {"attempt": 1, "status": "failed"},
                                    {"attempt": 2, "status": "passed"},
                                ],
                                "image": "/study/containers/rebuild-verification/"
                                + "a" * 40
                                + "/apptainer/addeam.sif",
                                "image_sha256": "0123456789abcdef" * 4,
                                "image_size_bytes": 123,
                                "build_log": "/study/containers/rebuild-verification/"
                                + "a" * 40
                                + "/evidence/addeam-attempt-2.log",
                                "build_log_sha256": "fedcba9876543210" * 4,
                            }
                        },
                        "evidence_root": "/study/containers/rebuild-verification/"
                        + "a" * 40
                        + "/evidence",
                    }
                ),
                encoding="utf-8",
            )
            inventory_path.write_text(
                json.dumps(
                    {
                        "schema_version": campaign.INVENTORY_SCHEMA,
                        "repository_commit": "a" * 40,
                        "summary": {
                            "states": {"current": 85, "unsupported": 4},
                            "staged_image_count": 0,
                            "unknown_image_count": 0,
                            "unreferenced_obsolete_count": 0,
                        },
                        "tools": [
                            {
                                "tool": "addeam",
                                "version": "1.0.0",
                                "state": "current",
                                "selected_image_sha256": "0123456789abcdef" * 4,
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )

            report, markdown = campaign.campaign_report(execution_path, inventory_path)

        self.assertEqual(report["verdict"], "passed")
        self.assertEqual(report["campaign_scope"], "targeted")
        self.assertEqual(report["tools"][0]["attempt_count"], 2)
        self.assertEqual(report["tools"][0]["failed_attempt_count"], 1)
        self.assertTrue(report["tools"][0]["inventory_matches_execution"])
        self.assertEqual(
            report["source_artifacts"]["execution"]["study_relative_path"],
            "containers/rebuild-verification/" + "a" * 40 + "/campaign/execution.json",
        )
        self.assertEqual(
            report["evidence_dimensions"], ["source", "build", "freshness"]
        )
        self.assertIn("science", report["not_evidence_for"])
        self.assertIn("does not prove runtime behavior", markdown)
        self.assertIn("only for the selected tools", markdown)
        self.assertNotIn("complete retained Apptainer roster", markdown)


if __name__ == "__main__":
    unittest.main()
