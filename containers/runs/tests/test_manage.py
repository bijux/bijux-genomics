from __future__ import annotations

import importlib.util
import json
from pathlib import Path
import tempfile
import unittest


SCRIPT = Path(__file__).parents[1] / "manage.py"
SPEC = importlib.util.spec_from_file_location("container_run_manage", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
manage = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(manage)


class RunEvidenceTests(unittest.TestCase):
    def make_result(self, root: Path) -> Path:
        result = root / "adapterremoval" / "run-1001"
        result.mkdir(parents=True)
        (result / "config-pipeline.yml").write_text(
            "schema_version: bijux.container.pipeline.v1\n"
            "run:\n"
            "  id: run-1001\n"
            "  tool: adapterremoval\n"
            "repository:\n"
            f"  commit: {'a' * 40}\n",
            encoding="utf-8",
        )
        (result / "config-credential.yml").write_text(
            "schema_version: bijux.container.credential_reference.v1\n"
            "credentials_required: false\n"
            "references: []\n"
            "redaction:\n"
            "  publish_values: false\n",
            encoding="utf-8",
        )
        (result / "evidence.json").write_text(
            json.dumps(
                {
                    "run_id": "run-1001",
                    "status": "passed",
                    "repository_commit": "a" * 40,
                    "runtime": {"slurm_job_id": "42"},
                    "image": {"sha256": "b" * 64},
                    "failures": [],
                }
            ),
            encoding="utf-8",
        )
        return result

    def test_import_preserves_contracts_and_locks_source_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            output = root / "runs"

            count = manage.import_evidence(root / "results", output, None, None)
            records = manage.validate_runs(output)

        self.assertEqual(count, 1)
        self.assertEqual(records[0]["run_id"], "run-1001")
        self.assertEqual(records[0]["execution_status"], "passed")
        self.assertEqual(records[0]["scientific_status"], "not_assessed")
        self.assertIn("config-pipeline.yml", records[0]["tracked_files"])

    def test_credential_value_fields_are_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config-credential.yml"
            path.write_text(
                "credentials_required: true\n"
                "references:\n"
                "  - id: service\n"
                "    token: disclosed\n"
                "redaction:\n"
                "  publish_values: false\n",
                encoding="utf-8",
            )

            with self.assertRaisesRegex(manage.RunEvidenceError, "forbidden"):
                manage.validate_credential_reference(path)

    def test_modified_tracked_evidence_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            output = root / "runs"
            manage.import_evidence(root / "results", output, None, None)
            pipeline = output / "run-1001-adapterremoval" / "config-pipeline.yml"
            pipeline.write_text("changed\n", encoding="utf-8")

            with self.assertRaisesRegex(manage.RunEvidenceError, "size changed"):
                manage.validate_runs(output)


if __name__ == "__main__":
    unittest.main()
