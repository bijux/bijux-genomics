from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
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


if __name__ == "__main__":
    unittest.main()
