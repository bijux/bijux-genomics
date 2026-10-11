from __future__ import annotations

import importlib.util
import json
import re
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

CONTRACTS = Path(__file__).parents[1]
sys.path.insert(0, str(CONTRACTS))
SPEC = importlib.util.spec_from_file_location(
    "runtime_verification", CONTRACTS / "runtime_verification.py"
)
assert SPEC is not None and SPEC.loader is not None
runtime_verification = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(runtime_verification)

ROOT = Path(__file__).parents[3]


class RuntimeVerificationTests(unittest.TestCase):
    def test_runtime_check_uses_a_resolvable_isolated_hostname(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with mock.patch.object(
                runtime_verification.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as execute:
                check = runtime_verification.run_check(
                    root / "tool.sif",
                    ["tool", "--version"],
                    root / "stdout.log",
                    root / "stderr.log",
                    30,
                )

        self.assertEqual(check["exit_code"], 0)
        command = execute.call_args.args[0]
        hostname_index = command.index("--hostname")
        self.assertEqual(
            command[hostname_index + 1],
            runtime_verification.ISOLATED_RUNTIME_HOSTNAME,
        )

    def test_prepare_accepts_the_inventory_frozen_repository(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study = root / "study"
            active = root / "active"
            frozen_root = root / "frozen"
            repository = frozen_root / "image-builds" / ("a" * 40)
            image = study / "containers/apptainer/tool-1.0-linux-amd64-id.sif"
            image.parent.mkdir(parents=True)
            image.write_bytes(b"sif")
            active.mkdir()
            repository.mkdir(parents=True)
            catalog = repository / "containers/contracts/catalog.toml"
            catalog.parent.mkdir(parents=True)
            catalog.write_text(
                "[[tools]]\n"
                'id = "tool"\n'
                'version = "1.0"\n'
                "apptainer_supported = true\n"
                'version_command = "tool --version"\n'
                'help_command = "tool --help"\n'
                'expected_version_regex = "tool 1[.]0"\n',
                encoding="utf-8",
            )
            subprocess.run(
                [
                    "git",
                    "init",
                    "--quiet",
                    "--initial-branch",
                    "main",
                    str(repository),
                ],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(repository), "config", "user.name", "Bijux Test"],
                check=True,
            )
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repository),
                    "config",
                    "user.email",
                    "test@bijux.io",
                ],
                check=True,
            )
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repository),
                    "add",
                    "containers/contracts/catalog.toml",
                ],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(repository), "commit", "--quiet", "-m", "fixture"],
                check=True,
            )
            commit = runtime_verification.git_output(repository, "rev-parse", "HEAD")
            inventory = root / "inventory.json"
            runtime_verification.write_json(
                inventory,
                {
                    "schema_version": runtime_verification.INVENTORY_SCHEMA,
                    "repository_commit": commit,
                    "catalog_sha256": runtime_verification.sha256_file(catalog),
                    "tools": [
                        {
                            "tool": "tool",
                            "apptainer_supported": True,
                            "state": "current",
                            "selected_image": str(image),
                            "selected_image_sha256": (
                                runtime_verification.sha256_file(image)
                            ),
                        }
                    ],
                },
            )
            governance = root / "governance.yml"
            governance.write_text(
                "schema_version: bijux.container.governance.v1\n"
                "site:\n"
                "  id: cluster\n"
                "  hostname_regex: '^worker-7$'\n"
                "paths:\n"
                f"  study_root: {study}\n"
                f"  active_repository: {active}\n"
                f"  frozen_repositories: {frozen_root}\n"
                "runtime:\n"
                "  apptainer_architecture: linux/amd64\n"
                "slurm:\n"
                "  account: science\n"
                f"  scratch_root: {root}\n"
                "  partitions: [compute]\n",
                encoding="utf-8",
            )
            output = root / "execution.json"
            with mock.patch.object(
                runtime_verification.socket,
                "gethostname",
                return_value="worker-7",
            ):
                execution = runtime_verification.prepare_execution(
                    inventory,
                    catalog,
                    governance,
                    output,
                    repository,
                )

        self.assertEqual(execution["repository_commit"], commit)
        self.assertEqual(execution["repository"]["path"], str(repository.resolve()))
        self.assertEqual(execution["tools"]["tool"]["status"], "pending")

    def test_every_supported_image_has_a_runtime_contract(self) -> None:
        contracts = runtime_verification.catalog_contracts(
            ROOT / "containers/contracts/catalog.toml"
        )

        self.assertEqual(len(contracts), 85)
        for tool_id, contract in contracts.items():
            with self.subTest(tool=tool_id):
                self.assertTrue(contract["version_command"])
                self.assertTrue(contract["help_command"])
                self.assertTrue(contract["version_regex"])

    def test_runtime_commands_retain_quoted_arguments(self) -> None:
        contracts = runtime_verification.catalog_contracts(
            ROOT / "containers/contracts/catalog.toml"
        )

        self.assertEqual(
            contracts["bcftools"]["version_command"], ["bcftools", "--version"]
        )

    def test_plink2_contract_matches_the_packaged_release_banner(self) -> None:
        contracts = runtime_verification.catalog_contracts(
            ROOT / "containers/contracts/catalog.toml"
        )

        self.assertIsNotNone(
            re.search(
                contracts["plink2"]["version_regex"],
                "PLINK v2.00a6 AVX2 (23 Nov 2023)",
            )
        )

    def test_report_separates_runtime_from_scientific_qualification(self) -> None:
        execution = {
            "schema_version": runtime_verification.EXECUTION_SCHEMA,
            "status": "passed",
            "repository_commit": "a" * 40,
            "site": {"id": "lunarc", "host": "cosmos3"},
            "summary": {"passed": 1},
            "catalog": {"sha256": "b" * 64},
            "inventory": {"sha256": "c" * 64},
            "timeout_seconds": 30,
            "tools": {
                "tool": {
                    "version": "1.0",
                    "status": "passed",
                    "failures": [],
                    "checks": {
                        "version": {"exit_code": 0, "timed_out": False},
                        "help": {"exit_code": 0, "timed_out": False},
                    },
                }
            },
        }
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "execution.json"
            output = root / "report.md"
            runtime_verification.write_json(source, execution)

            count = runtime_verification.render_report(source, output)
            report = output.read_text(encoding="utf-8")

        self.assertEqual(count, 1)
        self.assertIn("| `tool` | `1.0` | passed | passed | **passed** |", report)
        self.assertIn("does not establish real-data", report)

    def test_publication_preserves_hashes_and_removes_operator_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study_root = root / "study"
            repository = root / "repository"
            execution_path = root / "runtime.json"
            execution = {
                "schema_version": runtime_verification.EXECUTION_SCHEMA,
                "status": "passed",
                "repository_commit": "a" * 40,
                "catalog": {
                    "path": str(repository / "containers/contracts/catalog.toml"),
                    "sha256": "b" * 64,
                },
                "inventory": {
                    "path": str(study_root / "containers/evidence/inventory.json"),
                    "sha256": "c" * 64,
                },
                "tools": {
                    "samtools": {
                        "image": str(
                            study_root / "containers/apptainer/samtools.sif"
                        ),
                        "checks": {
                            "version": {
                                "stdout": str(
                                    study_root / "containers/evidence/version.log"
                                )
                            }
                        },
                    }
                },
            }
            execution_path.write_text(json.dumps(execution), encoding="utf-8")
            execution_sha256 = runtime_verification.sha256_file(execution_path)
            output = root / "published.json"

            result = runtime_verification.publish_execution(
                execution_path, study_root, repository, output
            )

        self.assertEqual(
            result["schema_version"], runtime_verification.PUBLICATION_SCHEMA
        )
        self.assertEqual(
            result["catalog"]["path"],
            "repository://containers/contracts/catalog.toml",
        )
        self.assertEqual(
            result["tools"]["samtools"]["image"],
            "study://containers/apptainer/samtools.sif",
        )
        self.assertEqual(
            result["tools"]["samtools"]["checks"]["version"]["stdout"],
            "study://containers/evidence/version.log",
        )
        self.assertEqual(
            result["publication"]["raw_sha256"],
            execution_sha256,
        )

    def test_publication_requires_terminal_execution(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            execution_path = root / "runtime.json"
            execution_path.write_text(
                json.dumps(
                    {
                        "schema_version": runtime_verification.EXECUTION_SCHEMA,
                        "status": "running",
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                runtime_verification.ContractError, "must be terminal"
            ):
                runtime_verification.publish_execution(
                    execution_path,
                    root / "study",
                    root / "repository",
                    root / "published.json",
                )


if __name__ == "__main__":
    unittest.main()
