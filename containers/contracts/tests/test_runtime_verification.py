from __future__ import annotations

import importlib.util
from pathlib import Path
import re
import sys
import tempfile
import unittest

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
    def test_every_supported_image_has_a_runtime_contract(self) -> None:
        contracts = runtime_verification.catalog_contracts(
            ROOT / "containers/contracts/catalog.toml"
        )

        self.assertEqual(len(contracts), 87)
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


if __name__ == "__main__":
    unittest.main()
