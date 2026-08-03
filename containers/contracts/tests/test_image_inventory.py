from __future__ import annotations

import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "image_inventory.py"
SPEC = importlib.util.spec_from_file_location("container_image_inventory", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
inventory = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(inventory)


class ImageInventoryTests(unittest.TestCase):
    def make_repository(self, root: Path) -> tuple[Path, str, str]:
        repository = root / "repository"
        definition = repository / "containers" / "apptainer" / "samtools.def"
        definition.parent.mkdir(parents=True)
        definition.write_text(
            "Bootstrap: docker\nFrom: samtools:1.20\n", encoding="utf-8"
        )
        subprocess.run(["git", "init", "-q", repository], check=True)
        subprocess.run(["git", "-C", repository, "add", definition], check=True)
        subprocess.run(
            [
                "git",
                "-C",
                repository,
                "-c",
                "user.name=Bijux Test",
                "-c",
                "user.email=test@bijux.io",
                "commit",
                "-qm",
                "test fixture",
            ],
            check=True,
        )
        commit = subprocess.run(
            ["git", "-C", repository, "rev-parse", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        return repository, commit, inventory.sha256_file(definition)

    def test_audit_requires_matching_image_recipe_and_build_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository, commit, definition_digest = self.make_repository(root)
            catalog = repository / "containers" / "contracts" / "catalog.toml"
            catalog.parent.mkdir()
            catalog.write_text(
                "[[tools]]\n"
                'id = "samtools"\n'
                'version = "1.20"\n'
                'support_disposition = "retained"\n'
                "apptainer_supported = true\n"
                'apptainer_definition = "containers/apptainer/samtools.def"\n',
                encoding="utf-8",
            )
            subprocess.run(["git", "-C", repository, "add", catalog], check=True)
            subprocess.run(
                [
                    "git",
                    "-C",
                    repository,
                    "-c",
                    "user.name=Bijux Test",
                    "-c",
                    "user.email=test@bijux.io",
                    "commit",
                    "-qm",
                    "add container catalog",
                ],
                check=True,
            )
            image_root = root / "images"
            evidence_root = root / "evidence"
            runs_root = root / "runs"
            image_root.mkdir()
            evidence_root.mkdir()
            runs_root.mkdir()
            image_bytes = b"immutable-sif"
            image_digest = __import__("hashlib").sha256(image_bytes).hexdigest()
            image = image_root / f"samtools-1.20-linux-amd64-{image_digest[:12]}.sif"
            image.write_bytes(image_bytes)
            previous_bytes = b"previous-immutable-sif"
            previous_digest = __import__("hashlib").sha256(previous_bytes).hexdigest()
            previous = (
                image_root
                / f"samtools-1.19-linux-amd64-{previous_digest[:12]}.sif"
            )
            previous.write_bytes(previous_bytes)
            (evidence_root / "samtools.build.json").write_text(
                json.dumps(
                    {
                        "schema_version": "bijux.container.build_evidence.v1",
                        "status": "passed",
                        "tool": "samtools",
                        "version": "1.20",
                        "repository_commit": commit,
                        "definition": {"sha256": definition_digest},
                        "image": {
                            "sha256": image_digest,
                            "size_bytes": len(image_bytes),
                        },
                    }
                ),
                encoding="utf-8",
            )
            output = root / "inventory.json"

            result = inventory.audit_images(
                catalog,
                image_root,
                evidence_root,
                runs_root,
                repository,
                output,
            )

        self.assertEqual(result["summary"]["states"], {"current": 1})
        self.assertEqual(result["image_root"], str(image_root.resolve()))
        self.assertEqual(result["evidence_root"], str(evidence_root.resolve()))
        self.assertEqual(result["runs_root"], str(runs_root.resolve()))
        self.assertEqual(
            result["tools"][0]["candidates"][0]["disposition"], "canonical"
        )
        self.assertEqual(
            result["unknown_images"][0]["catalog_tool"],
            "samtools",
        )
        self.assertEqual(
            result["unknown_images"][0]["disposition"],
            "unreferenced_obsolete",
        )
        self.assertEqual(result["summary"]["unreferenced_obsolete_count"], 1)

    def test_report_does_not_claim_release_or_science_qualification(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "inventory.json"
            source.write_text(
                json.dumps(
                    {
                        "schema_version": inventory.INVENTORY_SCHEMA,
                        "summary": {
                            "tool_count": 1,
                            "states": {"missing": 1},
                            "unknown_image_count": 0,
                            "staged_image_count": 0,
                            "unreferenced_obsolete_count": 0,
                        },
                        "tools": [
                            {
                                "tool": "samtools",
                                "version": "1.20",
                                "support_disposition": "retained",
                                "state": "missing",
                                "candidates": [],
                            }
                        ],
                    }
                ),
                encoding="utf-8",
            )
            output = root / "report.md"

            inventory.render_report(source, output)
            report = output.read_text(encoding="utf-8")

        self.assertIn("not a release certificate", report)
        self.assertIn("Scientific and security", report)
        self.assertIn("**missing**", report)

    def test_audit_refuses_to_replace_immutable_inventory(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository, _, _ = self.make_repository(root)
            output = root / "inventory.json"
            output.write_text("preserved evidence\n", encoding="utf-8")

            with self.assertRaisesRegex(
                inventory.ImageInventoryError, "refusing to replace image inventory"
            ):
                inventory.audit_images(
                    repository / "containers" / "contracts" / "catalog.toml",
                    root / "images",
                    root / "evidence",
                    root / "runs",
                    repository,
                    output,
                )


if __name__ == "__main__":
    unittest.main()
