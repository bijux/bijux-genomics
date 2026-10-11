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
                f'source_sha256 = "{"a" * 64}"\n',
                encoding="utf-8",
            )
            inventory = root / "inventory.json"
            inventory.write_text(
                json.dumps(
                    {
                        "repository_commit": "commit",
                        "tools": [{"tool": "tool", "state": "current"}],
                    }
                ),
                encoding="utf-8",
            )
            runtime = root / "runtime.json"
            runtime.write_text(
                json.dumps(
                    {
                        "repository_commit": "commit",
                        "tools": {"tool": {"status": "passed"}},
                    }
                ),
                encoding="utf-8",
            )
            runs = root / "runs.json"
            runs.write_text(
                json.dumps(
                    {
                        "runs": [
                            {
                                "tool": "tool",
                                "execution_status": "passed",
                                "scientific_status": "not_assessed",
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
        self.assertEqual(tool["qualification"], "unverified")
        self.assertEqual(report["summary"]["qualification"], {"unverified": 1})

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
