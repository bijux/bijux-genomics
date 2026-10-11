from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "image_storage.py"
SPEC = importlib.util.spec_from_file_location("container_image_storage", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
storage = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(storage)


class ImageStorageTests(unittest.TestCase):
    def test_quarantine_plan_is_verified_and_reversible(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            images = root / "images"
            staged = images / ".building"
            staged.mkdir(parents=True)
            obsolete = images / "tool-old.sif"
            interrupted = staged / "tool-build.sif"
            superseded = images / "tool-0.9-linux-amd64-deadbeef0000.sif"
            obsolete.write_bytes(b"old")
            interrupted.write_bytes(b"partial")
            superseded.write_bytes(b"superseded")
            inventory = root / "inventory.json"
            inventory.write_text(
                json.dumps(
                    {
                        "schema_version": storage.INVENTORY_SCHEMA,
                        "tools": [
                            {
                                "candidates": [
                                    {
                                        "path": str(obsolete),
                                        "size_bytes": 3,
                                        "sha256": storage.sha256_file(obsolete),
                                        "real_run_references": 0,
                                        "disposition": "unreferenced_obsolete",
                                    }
                                ]
                            }
                        ],
                        "staged_images": [
                            {
                                "path": str(interrupted),
                                "size_bytes": 7,
                                "sha256": storage.sha256_file(interrupted),
                            }
                        ],
                        "unknown_images": [
                            {
                                "path": str(superseded),
                                "size_bytes": 10,
                                "sha256": storage.sha256_file(superseded),
                                "real_run_references": 0,
                                "catalog_tool": "tool",
                                "disposition": "unreferenced_obsolete",
                            }
                        ],
                    }
                )
            )
            plan = root / "plan.json"
            quarantine = images / ".quarantine" / "inventory"

            self.assertEqual(
                storage.plan_storage(inventory, images, quarantine, plan), 3
            )
            self.assertEqual(storage.apply_storage(plan, images), 3)
            self.assertEqual(storage.verify_storage(plan, images), 3)
            self.assertFalse(obsolete.exists())
            self.assertFalse(interrupted.exists())
            self.assertFalse(superseded.exists())
            self.assertEqual(storage.rollback_storage(plan, images), 3)
            self.assertEqual(obsolete.read_bytes(), b"old")
            self.assertEqual(interrupted.read_bytes(), b"partial")
            self.assertEqual(superseded.read_bytes(), b"superseded")

    def test_storage_report_proves_reversible_canonical_state(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governed = root / "STUDY" / "containers"
            plan = governed / "evidence" / "inventory" / ("a" * 40) / "plan.json"
            after = plan.parent / "after-storage.json"
            plan.parent.mkdir(parents=True)
            quarantined = (
                governed
                / "apptainer"
                / ".quarantine"
                / ("a" * 40)
                / "obsolete"
                / "old.sif"
            )
            plan.write_text(
                json.dumps(
                    {
                        "schema_version": storage.PLAN_SCHEMA,
                        "state": "applied",
                        "file_count": 1,
                        "size_bytes": 3,
                        "moves": [
                            {
                                "category": "obsolete",
                                "source": str(governed / "apptainer" / "old.sif"),
                                "target": str(quarantined),
                                "size_bytes": 3,
                                "sha256": "0123456789abcdef" * 4,
                                "applied": True,
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            after.write_text(
                json.dumps(
                    {
                        "schema_version": storage.INVENTORY_SCHEMA,
                        "repository_commit": "a" * 40,
                        "evidence_root": str(governed / "evidence"),
                        "summary": {
                            "tool_count": 2,
                            "states": {"current": 1, "unsupported": 1},
                            "historical_dependency_count": 1,
                            "unknown_image_count": 0,
                            "staged_image_count": 0,
                            "unreferenced_obsolete_count": 0,
                        },
                    }
                ),
                encoding="utf-8",
            )

            report, markdown = storage.storage_report(plan, after)

        self.assertEqual(report["verdict"], "passed")
        self.assertEqual(report["moved_file_count"], 1)
        self.assertEqual(report["historical_dependencies_preserved"], 1)
        self.assertEqual(
            report["moved_images"][0]["quarantine_path"],
            "containers/apptainer/.quarantine/" + "a" * 40 + "/obsolete/old.sif",
        )
        self.assertIn("No SIF was deleted", markdown)


if __name__ == "__main__":
    unittest.main()
