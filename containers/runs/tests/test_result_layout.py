from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "result_layout.py"
SPEC = importlib.util.spec_from_file_location("container_result_layout", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
layout = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(layout)


class ResultLayoutTests(unittest.TestCase):
    def make_archive(self, root: Path) -> tuple[Path, Path]:
        runs = root / "runs"
        result_root = root / "results" / "container-validation"
        result = result_root / "samtools" / "run-1001"
        result.mkdir(parents=True)
        output = result / "sample.bam"
        output.write_bytes(b"governed-bam")
        lock_root = runs / "run-1001-samtools"
        lock_root.mkdir(parents=True)
        lock = {
            "run_id": "run-1001",
            "tool": "samtools",
            "source_result": "results/container-validation/samtools/run-1001",
            "source_files": [
                {
                    "path": "sample.bam",
                    "size_bytes": output.stat().st_size,
                    "sha256": layout.sha256_file(output),
                }
            ],
        }
        (lock_root / "evidence.lock.json").write_text(
            json.dumps(lock), encoding="utf-8"
        )
        return runs, result_root

    def test_plan_apply_verify_and_rollback_preserve_result(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runs, result_root = self.make_archive(root)
            plan = result_root / ".governance" / "result-layout.json"

            self.assertEqual(layout.plan_layout(runs, result_root, plan), 1)
            self.assertEqual(layout.apply_layout(plan, runs, result_root), 1)
            self.assertEqual(layout.verify_layout(plan, runs, result_root), 1)
            flat = result_root / "run-1001-samtools" / "sample.bam"
            self.assertEqual(flat.read_bytes(), b"governed-bam")
            self.assertFalse((result_root / "samtools").exists())
            self.assertEqual(layout.rollback_layout(plan, runs, result_root), 1)
            restored = result_root / "samtools" / "run-1001" / "sample.bam"
            self.assertEqual(restored.read_bytes(), b"governed-bam")

    def test_unrecorded_result_file_blocks_plan(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runs, result_root = self.make_archive(root)
            (result_root / "samtools" / "run-1001" / "untracked.log").write_text(
                "not inventoried", encoding="utf-8"
            )

            with self.assertRaisesRegex(layout.ResultLayoutError, "inventory differs"):
                layout.plan_layout(runs, result_root, root / "plan.json")

    def test_changed_result_blocks_apply(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runs, result_root = self.make_archive(root)
            plan = result_root / ".governance" / "result-layout.json"
            layout.plan_layout(runs, result_root, plan)
            (result_root / "samtools" / "run-1001" / "sample.bam").write_bytes(
                b"changed"
            )

            with self.assertRaisesRegex(layout.ResultLayoutError, "size changed"):
                layout.apply_layout(plan, runs, result_root)

    def make_superseded_archive(self, root: Path) -> tuple[Path, Path, Path]:
        runs = root / "runs"
        result_root = root / "results" / "container-validation"
        final = result_root / "run-1002-samtools"
        superseded = result_root / "run-1001-samtools"
        cache = result_root / "python-cache"
        final.mkdir(parents=True)
        superseded.mkdir()
        cache.mkdir()
        (final / "sample.bam").write_bytes(b"published")
        (superseded / "sample.bam").write_bytes(b"superseded")
        (cache / "module.pyc").write_bytes(b"cache")
        lock_root = runs / "run-1002-samtools"
        lock_root.mkdir(parents=True)
        (lock_root / "evidence.lock.json").write_text(
            json.dumps(
                {
                    "run_id": "run-1002",
                    "tool": "samtools",
                    "source_result": "results/container-validation/run-1002-samtools",
                    "source_files": [
                        {
                            "path": "sample.bam",
                            "size_bytes": 9,
                            "sha256": layout.sha256_file(final / "sample.bam"),
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        ledger = runs / "attempt-ledger.json"
        ledger.write_text(
            json.dumps(
                {
                    "schema_version": layout.ATTEMPT_LEDGER_SCHEMA,
                    "attempts": [
                        {
                            "run_id": "run-1001",
                            "tool": "samtools",
                            "evidence_lock_sha256": "a" * 64,
                            "superseded_by": "run-1002",
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        return runs, result_root, ledger

    def test_superseded_archive_preserves_attempts_and_active_finals(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runs, result_root, ledger = self.make_superseded_archive(root)
            archive_root = root / "evidence" / "superseded-results"
            plan = root / "evidence" / "result-layout.json"

            self.assertEqual(
                layout.plan_superseded_archive(
                    runs, ledger, result_root, archive_root, plan
                ),
                2,
            )
            self.assertEqual(
                layout.apply_superseded_archive(
                    plan, runs, ledger, result_root, archive_root
                ),
                2,
            )
            self.assertEqual(
                layout.verify_superseded_archive(
                    plan, runs, ledger, result_root, archive_root
                ),
                2,
            )
            self.assertEqual(
                sorted(path.name for path in result_root.iterdir()),
                ["run-1002-samtools"],
            )
            self.assertEqual(
                (archive_root / "run-1001-samtools" / "sample.bam").read_bytes(),
                b"superseded",
            )
            self.assertEqual(
                layout.rollback_superseded_archive(
                    plan, runs, ledger, result_root, archive_root
                ),
                2,
            )
            self.assertTrue((result_root / "run-1001-samtools").is_dir())
            self.assertTrue((result_root / "python-cache").is_dir())

    def test_changed_superseded_result_blocks_archive(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            runs, result_root, ledger = self.make_superseded_archive(root)
            archive_root = root / "evidence" / "superseded-results"
            plan = root / "evidence" / "result-layout.json"
            layout.plan_superseded_archive(
                runs, ledger, result_root, archive_root, plan
            )
            (result_root / "run-1001-samtools" / "sample.bam").write_bytes(
                b"changed"
            )

            with self.assertRaisesRegex(layout.ResultLayoutError, "size changed"):
                layout.apply_superseded_archive(
                    plan, runs, ledger, result_root, archive_root
                )


if __name__ == "__main__":
    unittest.main()
