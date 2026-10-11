from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "qualification_report.py"
SPEC = importlib.util.spec_from_file_location("container_qualification", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
qualification = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(qualification)


def write_compatible_evidence(
    catalog: Path,
    inventory: Path,
    runtime: Path,
    inventory_tools: list[dict[str, object]],
    runtime_tools: dict[str, dict[str, object]],
) -> None:
    catalog_sha256 = qualification.sha256_file(catalog)
    raw_inventory_sha256 = "b" * 64
    inventory.write_text(
        json.dumps(
            {
                "schema_version": qualification.INVENTORY_PUBLICATION_SCHEMA,
                "repository_commit": "commit",
                "catalog_sha256": catalog_sha256,
                "publication": {"raw_sha256": raw_inventory_sha256},
                "tools": inventory_tools,
            }
        ),
        encoding="utf-8",
    )
    runtime.write_text(
        json.dumps(
            {
                "schema_version": qualification.RUNTIME_PUBLICATION_SCHEMA,
                "repository_commit": "commit",
                "catalog": {"sha256": catalog_sha256},
                "inventory": {"sha256": raw_inventory_sha256},
                "tools": runtime_tools,
            }
        ),
        encoding="utf-8",
    )


class QualificationReportTests(unittest.TestCase):
    def test_successful_runtime_is_not_scientific_qualification(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.toml"
            catalog.write_text(
                'evidence_dimensions = ["catalog", "source", "build", "runtime", '
                '"stage", "real_data", "science", "benchmark", "security", '
                '"release", "freshness"]\n'
                "[[tools]]\n"
                'id = "tool"\n'
                'version = "1.0"\n'
                'support_disposition = "retained"\n'
                'upstream = "https://example.org/tool"\n'
                'documentation = "https://example.org/tool/docs"\n'
                'license = "MIT"\n'
                'real_test_stage = "fastq.trim_reads"\n'
                f'source_sha256 = "{"a" * 64}"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            runtime = root / "runtime.json"
            write_compatible_evidence(
                catalog,
                inventory,
                runtime,
                [
                    {
                        "tool": "tool",
                        "version": "1.0",
                        "support_disposition": "retained",
                        "apptainer_supported": True,
                        "state": "current",
                        "selected_image_sha256": "c" * 64,
                    }
                ],
                {
                    "tool": {
                        "version": "1.0",
                        "status": "passed",
                        "image_sha256": "c" * 64,
                    }
                },
            )
            runs = root / "runs.json"
            runs.write_text(
                json.dumps(
                    {
                        "runs": [
                            {
                                "run_id": "run-1001",
                                "tool": "tool",
                                "execution_status": "passed",
                                "scientific_status": "not_assessed",
                                "corpus": "public-corpus",
                                "sample": "sample-a",
                                "domain": "fastq",
                                "stage": "fastq.trim_reads",
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            report = qualification.derive_report(catalog, inventory, runtime, runs)

        tool = report["tools"][0]
        self.assertEqual(tool["dimensions"]["runtime"], "passed")
        self.assertEqual(tool["dimensions"]["real_data"], "passed")
        self.assertEqual(tool["dimensions"]["science"], "missing")
        self.assertEqual(tool["evidence_mismatches"], [])
        self.assertEqual(tool["qualification"], "unverified")
        self.assertEqual(report["summary"]["qualification"], {"unverified": 1})

    def test_stale_catalog_evidence_cannot_pass_build_runtime_or_freshness(
        self,
    ) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.toml"
            catalog.write_text(
                'evidence_dimensions = ["catalog", "source", "build", "runtime", '
                '"stage", "real_data", "science", "benchmark", "security", '
                '"release", "freshness"]\n'
                "[[tools]]\n"
                'id = "tool"\n'
                'version = "1.0"\n'
                'support_disposition = "retained"\n'
                'upstream = "https://example.org/tool"\n'
                'documentation = "https://example.org/tool/docs"\n'
                'license = "MIT"\n'
                'real_test_stage = "fastq.trim_reads"\n'
                f'source_sha256 = "{"a" * 64}"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            runtime = root / "runtime.json"
            write_compatible_evidence(
                catalog,
                inventory,
                runtime,
                [
                    {
                        "tool": "tool",
                        "version": "1.0",
                        "support_disposition": "retained",
                        "apptainer_supported": True,
                        "state": "current",
                        "selected_image_sha256": "c" * 64,
                    }
                ],
                {
                    "tool": {
                        "version": "1.0",
                        "status": "passed",
                        "image_sha256": "c" * 64,
                    }
                },
            )
            catalog.write_text(
                catalog.read_text(encoding="utf-8").replace(
                    'version = "1.0"', 'version = "1.1"'
                ),
                encoding="utf-8",
            )
            runs = root / "runs.json"
            runs.write_text('{"runs": []}\n', encoding="utf-8")

            report = qualification.derive_report(catalog, inventory, runtime, runs)

        tool = report["tools"][0]
        self.assertEqual(tool["dimensions"]["build"], "missing")
        self.assertEqual(tool["dimensions"]["runtime"], "missing")
        self.assertEqual(tool["dimensions"]["freshness"], "missing")
        self.assertEqual(
            tool["evidence_mismatches"],
            [
                "inventory_catalog",
                "runtime_catalog",
                "inventory_tool_identity",
                "runtime_tool_identity",
            ],
        )

    def test_passing_execution_for_another_stage_is_not_stage_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.toml"
            catalog.write_text(
                'evidence_dimensions = ["catalog", "source", "build", "runtime", '
                '"stage", "real_data", "science", "benchmark", "security", '
                '"release", "freshness"]\n'
                "[[tools]]\n"
                'id = "tool"\n'
                'version = "1.0"\n'
                'support_disposition = "retained"\n'
                'upstream = "https://example.org/tool"\n'
                'documentation = "https://example.org/tool/docs"\n'
                'license = "MIT"\n'
                'real_test_stage = "fastq.merge_pairs"\n'
                f'source_sha256 = "{"a" * 64}"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            runtime = root / "runtime.json"
            write_compatible_evidence(
                catalog,
                inventory,
                runtime,
                [
                    {
                        "tool": "tool",
                        "version": "1.0",
                        "support_disposition": "retained",
                        "apptainer_supported": True,
                        "state": "current",
                        "selected_image_sha256": "c" * 64,
                    }
                ],
                {
                    "tool": {
                        "version": "1.0",
                        "status": "passed",
                        "image_sha256": "c" * 64,
                    }
                },
            )
            runs = root / "runs.json"
            runs.write_text(
                json.dumps(
                    {
                        "runs": [
                            {
                                "run_id": "run-1001",
                                "tool": "tool",
                                "execution_status": "passed",
                                "scientific_status": "passed",
                                "corpus": "public-corpus",
                                "sample": "sample-a",
                                "domain": "fastq",
                                "stage": "fastq.trim_reads",
                            }
                        ]
                    }
                ),
                encoding="utf-8",
            )

            report = qualification.derive_report(catalog, inventory, runtime, runs)

        tool = report["tools"][0]
        self.assertEqual(tool["dimensions"]["stage"], "missing")
        self.assertEqual(tool["dimensions"]["real_data"], "missing")
        self.assertEqual(tool["dimensions"]["science"], "missing")
        self.assertEqual(tool["mismatched_passing_stages"], ["fastq.trim_reads"])

    def test_unresolved_license_keeps_catalog_dimension_missing(self) -> None:
        tool = {
            "id": "tool",
            "version": "1.0",
            "upstream": "https://example.org/tool",
            "documentation": "https://example.org/tool/docs",
            "real_test_stage": "fastq.trim_reads",
            "license": "NOASSERTION",
        }

        self.assertFalse(qualification.catalog_identity_is_resolved(tool))

        tool["license"] = "MIT"
        self.assertTrue(qualification.catalog_identity_is_resolved(tool))

    def test_unsupported_tool_keeps_its_own_declared_stage(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.toml"
            catalog.write_text(
                'evidence_dimensions = ["catalog", "source", "build", "runtime", '
                '"stage", "real_data", "science", "benchmark", "security", '
                '"release", "freshness"]\n'
                "[[tools]]\n"
                'id = "retained"\n'
                'version = "1.0"\n'
                'support_disposition = "retained"\n'
                'upstream = "https://example.org/retained"\n'
                'documentation = "https://example.org/retained/docs"\n'
                'license = "MIT"\n'
                'real_test_stage = "fastq.trim_reads"\n'
                f'source_sha256 = "{"a" * 64}"\n'
                "[[tools]]\n"
                'id = "unsupported"\n'
                'version = "2.0"\n'
                'support_disposition = "unsupported"\n'
                'real_test_stage = "vcf.impute"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            inventory.write_text(
                json.dumps(
                    {
                        "schema_version": qualification.INVENTORY_PUBLICATION_SCHEMA,
                        "repository_commit": "commit",
                        "tools": [],
                    }
                ),
                encoding="utf-8",
            )
            runtime = root / "runtime.json"
            runtime.write_text(
                json.dumps(
                    {
                        "schema_version": qualification.RUNTIME_PUBLICATION_SCHEMA,
                        "repository_commit": "commit",
                        "tools": {},
                    }
                ),
                encoding="utf-8",
            )
            runs = root / "runs.json"
            runs.write_text('{"runs": []}\n', encoding="utf-8")

            report = qualification.derive_report(catalog, inventory, runtime, runs)

        unsupported = report["tools"][1]
        self.assertEqual(unsupported["declared_real_test_stage"], "vcf.impute")
        self.assertEqual(unsupported["qualification"], "unsupported")

    def test_raw_execution_evidence_is_not_publishable_qualification_input(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            catalog = root / "catalog.toml"
            catalog.write_text(
                'evidence_dimensions = ["catalog", "source", "build", "runtime", '
                '"stage", "real_data", "science", "benchmark", "security", '
                '"release", "freshness"]\n'
                "tools = []\n",
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            inventory.write_text(
                json.dumps(
                    {
                        "schema_version": "bijux.container.image_inventory.v1",
                        "tools": [],
                    }
                ),
                encoding="utf-8",
            )
            runtime = root / "runtime.json"
            runtime.write_text(
                json.dumps(
                    {
                        "schema_version": qualification.RUNTIME_PUBLICATION_SCHEMA,
                        "tools": {},
                    }
                ),
                encoding="utf-8",
            )
            runs = root / "runs.json"
            runs.write_text('{"runs": []}\n', encoding="utf-8")

            with self.assertRaisesRegex(
                qualification.QualificationError, "published image inventory"
            ):
                qualification.derive_report(catalog, inventory, runtime, runs)

    def test_generated_report_check_rejects_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            report = Path(directory) / "qualification.md"
            report.write_text("current\n", encoding="utf-8")

            qualification.require_current(report, "current\n")
            with self.assertRaisesRegex(
                qualification.QualificationError, "generated report is stale"
            ):
                qualification.require_current(report, "expected\n")


if __name__ == "__main__":
    unittest.main()
