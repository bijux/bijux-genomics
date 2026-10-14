"""Exercise native case receipts against real unittest process outcomes."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "makes/bin/python_test_receipt.py"


class PythonCaseReceiptTests(unittest.TestCase):
    def setUp(self):
        scratch = ROOT / "artifacts/tests/python-case-receipts"
        scratch.mkdir(parents=True, exist_ok=True)
        self.owner = tempfile.TemporaryDirectory(dir=scratch, prefix="native-")
        self.addCleanup(self.owner.cleanup)
        self.root = Path(self.owner.name)
        self.output = self.root / "artifacts/outcome.json"
        self.case = self.root / "case_fixture.py"

    def invoke(self, body, *, discover=False, expected=None, selections=None, harness="unittest"):
        prefix = "import unittest\nclass NativeCases(unittest.TestCase):\n" if harness == "unittest" else "import pytest\n"
        self.case.write_text(prefix + body)
        args = [sys.executable, str(HELPER), "--source-root", str(self.root),
                "--output", str(self.output), "--scratch", str(self.root / "artifacts/scratch"),
                "--harness", harness]
        if discover:
            args.append("--discover")
        if expected is not None:
            admitted = self.root / "artifacts/inventory.json"
            admitted.parent.mkdir(parents=True, exist_ok=True)
            admitted.write_text(json.dumps({"expected": expected}))
            args.extend(["--expected", str(admitted)])
        args.extend(selections or (["case_fixture"] if harness == "unittest" else [str(self.case)]))
        process = subprocess.run(args, capture_output=True, text=True,
                                 env=dict(os.environ, PYTHONDONTWRITEBYTECODE="1", PYTEST_DISABLE_PLUGIN_AUTOLOAD="1"))
        return process, json.loads(self.output.read_text())

    def test_discovery_and_execution_preserve_native_identity_and_source_reads(self):
        body = "    def test_retained(self): self.assertEqual(2 + 2, 4)\n"
        process, discovered = self.invoke(body, discover=True)
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(discovered["expected"], ["case_fixture.NativeCases.test_retained"])
        self.assertEqual(discovered["outcomes"], [])
        process, executed = self.invoke(body, expected=discovered["expected"])
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertEqual(executed["outcomes"], [{"id": discovered["expected"][0], "outcome": "passed"}])
        self.assertIn("case_fixture.py", executed["source_reads"])

    def test_failed_skipped_expected_failure_and_duplicates_cannot_claim_green(self):
        bodies = [
            "    def test_case(self): self.fail('retained failure')\n",
            "    @unittest.skip('skip is not execution')\n    def test_case(self): pass\n",
            "    @unittest.expectedFailure\n    def test_case(self): self.fail('expected')\n",
            "    def test_case(self):\n        with self.subTest(value=1): self.fail('subtest')\n",
        ]
        for body in bodies:
            with self.subTest(body=body):
                process, facts = self.invoke(body)
                self.assertNotEqual(process.returncode, 0)
                self.assertFalse(facts["membership"]["all_passed"])
        process, facts = self.invoke("    def test_case(self): pass\n", selections=["case_fixture", "case_fixture"])
        self.assertNotEqual(process.returncode, 0)
        self.assertTrue(facts["membership"]["duplicate"])

    def test_execution_cannot_adopt_missing_or_changed_admitted_membership(self):
        for expected in [[], ["case_fixture.NativeCases.test_removed"],
                         ["case_fixture.NativeCases.test_case"] * 2]:
            with self.subTest(expected=expected):
                process, facts = self.invoke("    def test_case(self): pass\n", expected=expected)
                self.assertNotEqual(process.returncode, 0)
                self.assertFalse(facts["membership"]["all_passed"])

    def test_native_pytest_skip_xfail_and_xpass_cannot_claim_green(self):
        process, facts = self.invoke("def test_retained(): assert 6 * 7 == 42\n", harness="pytest")
        self.assertEqual(process.returncode, 0, process.stderr)
        self.assertTrue(facts["membership"]["all_passed"])
        for body in [
            "@pytest.mark.skip(reason='retained skip')\ndef test_retained(): pass\n",
            "@pytest.mark.xfail(reason='')\ndef test_retained(): assert False\n",
            "@pytest.mark.xfail(reason='')\ndef test_retained(): pass\n",
        ]:
            with self.subTest(body=body):
                process, facts = self.invoke(body, harness="pytest")
                self.assertNotEqual(process.returncode, 0)
                self.assertFalse(facts["membership"]["all_passed"])


if __name__ == "__main__":
    unittest.main()
