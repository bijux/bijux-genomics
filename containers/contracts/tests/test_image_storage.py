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


if __name__ == "__main__":
    unittest.main()
