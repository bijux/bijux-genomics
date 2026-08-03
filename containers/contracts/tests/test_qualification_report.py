from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "qualification_report.py"
sys.path.insert(0, str(SCRIPT.parent))
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


def write_benchmark_report(
    path: Path,
    *,
    image_sha256: str,
    tool_closure: list[str] | None = None,
) -> None:
    closure = ["tool"] if tool_closure is None else tool_closure
    path.write_text(
        json.dumps(
            {
                "schema_version": qualification.BENCHMARK_REPORT_SCHEMA,
                "verdict": "passed",
                "run_id": "run-2001",
                "stage_id": "fastq.trim_reads",
                "design": {
                    "runtime": "apptainer",
                    "scheduler": "slurm",
                    "container_only": True,
                    "warmup_repetitions": 1,
                    "measured_repetitions": 3,
                    "tool_closure": closure,
                },
                "tools": [
                    {
                        "tool": "tool",
                        "verdict": "passed",
                        "image": {"sha256": image_sha256},
                        "performance": {"wall_seconds": {"mean": 1.0}},
                        "scientific_observations": {"retained_reads": [10, 10, 10]},
                    }
                ],
            }
        ),
        encoding="utf-8",
    )


class QualificationReportTests(unittest.TestCase):
    def test_recipe_freshness_includes_committed_files_inputs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            repository = Path(directory)
            definition = repository / "containers/apptainer/tool.def"
            source = repository / "containers/tools/tool/tool"
            definition.parent.mkdir(parents=True)
            source.parent.mkdir(parents=True)
            definition.write_text(
                "Bootstrap: docker\nFrom: example@sha256:abc\n\n%files\n"
                "    containers/tools/tool/tool /usr/local/bin/tool\n",
                encoding="utf-8",
            )
            source.write_text("original\n", encoding="utf-8")
            subprocess.run(
                ["git", "init", "--quiet", "--initial-branch", "main"],
                cwd=repository,
                check=True,
            )
            subprocess.run(
                ["git", "config", "user.name", "Bijux Test"],
                cwd=repository,
                check=True,
            )
            subprocess.run(
                ["git", "config", "user.email", "test@bijux.io"],
                cwd=repository,
                check=True,
            )
            subprocess.run(["git", "add", "."], cwd=repository, check=True)
            subprocess.run(
                ["git", "commit", "--quiet", "-m", "fixture"],
                cwd=repository,
                check=True,
            )
            source_object = subprocess.run(
                ["git", "rev-parse", "HEAD:containers/tools/tool/tool"],
                cwd=repository,
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            image = {
                "state": "current",
                "selected_image_sha256": "a" * 64,
                "candidates": [
                    {
                        "sha256": "a" * 64,
                        "admitted": True,
                        "build_evidence": [
                            {
                                "status": "passed",
                                "definition_sha256": qualification.sha256_file(
                                    definition
                                ),
                                "build_context": [
                                    {
                                        "path": "containers/tools/tool/tool",
                                        "git_object": source_object,
                                    }
                                ],
                            }
                        ],
                    }
                ],
            }
            tool = {"apptainer_definition": "containers/apptainer/tool.def"}

            self.assertTrue(
                qualification.published_recipe_matches_repository(
                    tool, image, repository
                )
            )
            source.write_text("changed\n", encoding="utf-8")
            subprocess.run(["git", "add", "."], cwd=repository, check=True)
            subprocess.run(
                ["git", "commit", "--quiet", "-m", "change source"],
                cwd=repository,
                check=True,
            )
            self.assertFalse(
                qualification.published_recipe_matches_repository(
                    tool, image, repository
                )
            )

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

    def test_admitted_recipe_benchmark_passes_only_benchmark_dimension(self) -> None:
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
                'stage_ids = ["fastq.trim_reads"]\n'
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
                        "selected_image_sha256": "d" * 64,
                        "candidates": [
                            {
                                "sha256": "c" * 64,
                                "admitted": True,
                                "current_recipe": True,
                            }
                        ],
                    }
                ],
                {
                    "tool": {
                        "version": "1.0",
                        "status": "passed",
                        "image_sha256": "d" * 64,
                    }
                },
            )
            runs = root / "runs.json"
            runs.write_text('{"runs": []}\n', encoding="utf-8")
            benchmark = root / "benchmark.json"
            write_benchmark_report(benchmark, image_sha256="c" * 64)

            report = qualification.derive_report(
                catalog, inventory, runtime, runs, (benchmark,)
            )

        tool = report["tools"][0]
        self.assertEqual(tool["dimensions"]["benchmark"], "passed")
        self.assertEqual(tool["dimensions"]["science"], "missing")
        self.assertEqual(tool["passing_benchmark_runs"], ["run-2001"])
        self.assertEqual(tool["benchmark_stages"], ["fastq.trim_reads"])
        self.assertEqual(tool["unadmitted_image_benchmark_runs"], [])

    def test_benchmark_requires_complete_stage_tool_closure(self) -> None:
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
                'stage_ids = ["fastq.trim_reads"]\n'
                'real_test_stage = "fastq.trim_reads"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            runtime = root / "runtime.json"
            write_compatible_evidence(catalog, inventory, runtime, [], {})
            runs = root / "runs.json"
            runs.write_text('{"runs": []}\n', encoding="utf-8")
            benchmark = root / "benchmark.json"
            write_benchmark_report(
                benchmark, image_sha256="c" * 64, tool_closure=["another-tool"]
            )

            with self.assertRaisesRegex(
                qualification.QualificationError, "complete retained-tool closure"
            ):
                qualification.derive_report(
                    catalog, inventory, runtime, runs, (benchmark,)
                )

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

    def test_raw_execution_evidence_is_not_publishable_qualification_input(
        self,
    ) -> None:
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
