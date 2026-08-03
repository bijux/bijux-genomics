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
        self.assertEqual(
            records[0]["source_result"],
            "results/container-validation/adapterremoval/run-1001",
        )
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

    def test_inline_redaction_mapping_from_historical_runs_is_accepted(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config-credential.yml"
            path.write_text(
                "credentials_required: false\n"
                "references: []\n"
                "redaction: {publish_values: false}\n",
                encoding="utf-8",
            )

            manage.validate_credential_reference(path)

    def test_inline_historical_run_identity_is_read(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            path = Path(directory) / "config-pipeline.yml"
            path.write_text(
                "run: {id: run-1035, campaign: phasing, tool: bcftools}\n"
                f"repository: {{commit: {'a' * 40}, clean: true}}\n",
                encoding="utf-8",
            )

            values = manage.yaml_scalars(path)

        self.assertEqual(values[("run", "id")], "run-1035")
        self.assertEqual(values[("run", "tool")], "bcftools")
        self.assertEqual(values[("repository", "commit")], "a" * 40)

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

    def test_flat_result_path_is_recorded_once(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            nested = self.make_result(root / "staging")
            result_root = root / "results"
            flat = result_root / "run-1001-adapterremoval"
            flat.parent.mkdir()
            nested.rename(flat)
            output = root / "runs"

            manage.import_evidence(result_root, output, None, None)
            records = manage.validate_runs(output)

        self.assertEqual(
            records[0]["source_result"],
            "results/container-validation/run-1001-adapterremoval",
        )

    def test_report_keeps_execution_and_science_claims_separate(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            runs = root / "runs"
            report = root / "run-evidence.md"
            manage.import_evidence(root / "results", runs, None, None)

            count = manage.render_report(runs, report)
            rendered = report.read_text(encoding="utf-8")

        self.assertEqual(count, 1)
        self.assertIn("| `adapterremoval` | 1 | 1 | 0 |", rendered)
        self.assertIn("not_assessed", rendered)
        self.assertIn("successful process is not", rendered)

    def test_invalid_source_inventory_digest_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            runs = root / "runs"
            manage.import_evidence(root / "results", runs, None, None)
            lock = runs / "run-1001-adapterremoval" / "evidence.lock.json"
            record = json.loads(lock.read_text(encoding="utf-8"))
            record["source_files"][0]["sha256"] = "invalid"
            lock.write_text(json.dumps(record), encoding="utf-8")

            with self.assertRaisesRegex(manage.RunEvidenceError, "digest is invalid"):
                manage.validate_runs(runs)


if __name__ == "__main__":
    unittest.main()
