from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

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
            "  campaign: public-read-trimming\n"
            "  corpus: ena-public-reads\n"
            "  sample: ERR000001\n"
            "  domain: fastq\n"
            "  stage: fastq.trim_reads\n"
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
            manage.publish_final_evidence(
                runs, runs / "attempt-ledger.json", True
            )

            count = manage.render_report(runs, report)
            rendered = report.read_text(encoding="utf-8")

        self.assertEqual(count, 1)
        self.assertIn("| `adapterremoval` | [run-1001]", rendered)
        self.assertIn("not_assessed", rendered)
        self.assertIn("successful process is not", rendered)
        self.assertIn("`ena-public-reads`", rendered)
        self.assertIn("`fastq.trim_reads`", rendered)

    def test_index_exposes_scientific_execution_identity(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            runs = root / "runs"
            index = root / "run-evidence.json"
            manage.import_evidence(root / "results", runs, None, None)
            manage.publish_final_evidence(runs, runs / "attempt-ledger.json", True)

            manage.render_index(runs, index)
            record = json.loads(index.read_text(encoding="utf-8"))["runs"][0]

        self.assertEqual(record["corpus"], "ena-public-reads")
        self.assertEqual(record["stage"], "fastq.trim_reads")
        self.assertEqual(record["domain"], "fastq")

    def test_final_publication_keeps_latest_success_and_ledgers_attempts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.make_result(root / "results")
            runs = root / "runs"
            manage.import_evidence(root / "results", runs, None, None)
            second = result.parent / "run-1002"
            second.mkdir()
            for source in result.iterdir():
                target = second / source.name
                target.write_bytes(source.read_bytes())
            pipeline = second / "config-pipeline.yml"
            pipeline.write_text(
                pipeline.read_text(encoding="utf-8").replace("run-1001", "run-1002"),
                encoding="utf-8",
            )
            evidence = json.loads((second / "evidence.json").read_text(encoding="utf-8"))
            evidence["run_id"] = "run-1002"
            evidence["status"] = "failed"
            evidence["failures"] = ["governed failure"]
            (second / "evidence.json").write_text(json.dumps(evidence), encoding="utf-8")
            manage.import_evidence(root / "results", runs, 1002, 1002)
            ledger = runs / "attempt-ledger.json"

            selected, superseded = manage.publish_final_evidence(runs, ledger, True)

            self.assertEqual((selected, superseded), (1, 1))
            self.assertTrue((runs / "run-1001-adapterremoval").is_dir())
            self.assertFalse((runs / "run-1002-adapterremoval").exists())
            attempts = json.loads(ledger.read_text(encoding="utf-8"))["attempts"]
            self.assertEqual(attempts[0]["failures"], ["governed failure"])
            records = manage.validate_runs(runs)
            manage.validate_final_evidence(records)
            manage.validate_attempt_ledger(ledger, records)

            third = result.parent / "run-1003"
            third.mkdir()
            for source in result.iterdir():
                target = third / source.name
                target.write_bytes(source.read_bytes())
            pipeline = third / "config-pipeline.yml"
            pipeline.write_text(
                pipeline.read_text(encoding="utf-8").replace("run-1001", "run-1003"),
                encoding="utf-8",
            )
            evidence = json.loads((third / "evidence.json").read_text(encoding="utf-8"))
            evidence["run_id"] = "run-1003"
            (third / "evidence.json").write_text(json.dumps(evidence), encoding="utf-8")
            manage.import_evidence(root / "results", runs, 1003, 1003)

            manage.publish_final_evidence(runs, ledger, True)

            attempts = json.loads(ledger.read_text(encoding="utf-8"))["attempts"]
            self.assertEqual({row["run_id"] for row in attempts}, {"run-1001", "run-1002"})
            self.assertEqual({row["superseded_by"] for row in attempts}, {"run-1003"})

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

    def test_report_rejects_superseded_attempt_directories(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            result = self.make_result(root / "results")
            runs = root / "runs"
            report = root / "run-evidence.md"
            manage.import_evidence(root / "results", runs, None, None)
            duplicate = result.parent / "run-1002"
            duplicate.mkdir()
            for source in result.iterdir():
                (duplicate / source.name).write_bytes(source.read_bytes())
            pipeline = duplicate / "config-pipeline.yml"
            pipeline.write_text(
                pipeline.read_text(encoding="utf-8").replace("run-1001", "run-1002"),
                encoding="utf-8",
            )
            evidence = json.loads((duplicate / "evidence.json").read_text(encoding="utf-8"))
            evidence["run_id"] = "run-1002"
            (duplicate / "evidence.json").write_text(json.dumps(evidence), encoding="utf-8")
            manage.import_evidence(root / "results", runs, 1002, 1002)

            with self.assertRaisesRegex(manage.RunEvidenceError, "superseded attempts"):
                manage.render_report(runs, report)

    def test_report_rejects_attempt_ledger_count_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            self.make_result(root / "results")
            runs = root / "runs"
            report = root / "run-evidence.md"
            manage.import_evidence(root / "results", runs, None, None)
            ledger = runs / "attempt-ledger.json"
            manage.publish_final_evidence(runs, ledger, True)
            value = json.loads(ledger.read_text(encoding="utf-8"))
            value["summary"]["published_run_count"] = 2
            ledger.write_text(json.dumps(value), encoding="utf-8")

            with self.assertRaisesRegex(manage.RunEvidenceError, "published count drifted"):
                manage.render_report(runs, report)


if __name__ == "__main__":
    unittest.main()
