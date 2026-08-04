from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "image_inventory.py"
sys.path.insert(0, str(SCRIPT.parent))
SPEC = importlib.util.spec_from_file_location("container_image_inventory", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
inventory = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(inventory)


class ImageInventoryTests(unittest.TestCase):
    def make_repository(self, root: Path) -> tuple[Path, str, str]:
        repository = root / "repository"
        definition = repository / "containers" / "apptainer" / "samtools.def"
        definition.parent.mkdir(parents=True)
        wrapper = repository / "containers" / "tools" / "samtools" / "samtools"
        wrapper.parent.mkdir(parents=True)
        wrapper.write_text("#!/bin/sh\nexec samtools \"$@\"\n", encoding="utf-8")
        definition.write_text(
            "Bootstrap: docker\n"
            "From: samtools:1.20\n"
            "%files\n"
            "    containers/tools/samtools/samtools /usr/local/bin/samtools\n",
            encoding="utf-8",
        )
        subprocess.run(["git", "init", "-q", repository], check=True)
        subprocess.run(
            ["git", "-C", repository, "add", definition, wrapper], check=True
        )
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
                image_root / f"samtools-1.19-linux-amd64-{previous_digest[:12]}.sif"
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
                        "build_context": [
                            {
                                "path": "containers/tools/samtools/samtools",
                                "git_object": subprocess.run(
                                    [
                                        "git",
                                        "-C",
                                        repository,
                                        "rev-parse",
                                        f"{commit}:containers/tools/samtools/samtools",
                                    ],
                                    check=True,
                                    capture_output=True,
                                    text=True,
                                ).stdout.strip(),
                            }
                        ],
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
            wrapper = repository / "containers/tools/samtools/samtools"
            wrapper.write_text(
                "#!/bin/sh\nexec /opt/samtools/bin/samtools \"$@\"\n",
                encoding="utf-8",
            )
            subprocess.run(["git", "-C", repository, "add", wrapper], check=True)
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
                    "change runtime wrapper",
                ],
                check=True,
            )
            stale_result = inventory.audit_images(
                catalog,
                image_root,
                evidence_root,
                runs_root,
                repository,
                root / "inventory-after-wrapper-change.json",
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
        self.assertEqual(stale_result["summary"]["states"], {"stale_recipe": 1})
        self.assertTrue(stale_result["tools"][0]["candidates"][0]["admitted"])
        self.assertFalse(
            stale_result["tools"][0]["candidates"][0]["current_recipe"]
        )

    def test_publication_replaces_governed_roots_with_stable_schemes(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study_root = root / "study"
            repository = root / "repository"
            image = study_root / "containers" / "apptainer" / "samtools.sif"
            catalog = repository / "containers" / "contracts" / "catalog.toml"
            image.parent.mkdir(parents=True)
            catalog.parent.mkdir(parents=True)
            source = root / "inventory.json"
            source.write_text(
                json.dumps(
                    {
                        "schema_version": inventory.INVENTORY_SCHEMA,
                        "image_root": str(image.parent),
                        "catalog": str(catalog),
                        "summary": {"tool_count": 1},
                        "tools": [{"tool": "samtools", "selected_image": str(image)}],
                    }
                ),
                encoding="utf-8",
            )
            source_sha256 = inventory.sha256_file(source)
            output = root / "published.json"

            result = inventory.publish_inventory(
                source, study_root, repository, output
            )

        self.assertEqual(result["schema_version"], inventory.PUBLICATION_SCHEMA)
        self.assertEqual(
            result["image_root"], "study://containers/apptainer"
        )
        self.assertEqual(
            result["catalog"], "repository://containers/contracts/catalog.toml"
        )
        self.assertEqual(
            result["tools"][0]["selected_image"],
            "study://containers/apptainer/samtools.sif",
        )
        self.assertEqual(
            result["publication"]["raw_sha256"], source_sha256
        )

    def test_publication_rejects_absolute_paths_outside_governed_roots(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            source = root / "inventory.json"
            source.write_text(
                json.dumps(
                    {
                        "schema_version": inventory.INVENTORY_SCHEMA,
                        "summary": {"tool_count": 0},
                        "tools": [],
                        "untrusted": "/unmanaged/evidence.json",
                    }
                ),
                encoding="utf-8",
            )

            with self.assertRaisesRegex(
                inventory.ImageInventoryError, "outside governed roots"
            ):
                inventory.publish_inventory(
                    source,
                    root / "study",
                    root / "repository",
                    root / "published.json",
                )

    def test_referenced_unsupported_image_is_a_historical_dependency(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            repository, _, _ = self.make_repository(root)
            catalog = repository / "containers" / "contracts" / "catalog.toml"
            catalog.parent.mkdir()
            catalog.write_text(
                "[[tools]]\n"
                'id = "fastuniq"\n'
                'version = "1.1"\n'
                'support_disposition = "unsupported"\n'
                "apptainer_supported = false\n",
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
                    "add unsupported catalog entry",
                ],
                check=True,
            )
            images = root / "images"
            evidence = root / "evidence"
            runs = root / "runs" / "run-1023-fastuniq"
            images.mkdir()
            evidence.mkdir()
            runs.mkdir(parents=True)
            image_bytes = b"historical-fastuniq-image"
            digest = __import__("hashlib").sha256(image_bytes).hexdigest()
            image = images / f"fastuniq-1.1-linux-amd64-{digest[:12]}.sif"
            image.write_bytes(image_bytes)
            (runs / "evidence.lock.json").write_text(
                json.dumps({"image": {"sha256": digest}}), encoding="utf-8"
            )

            result = inventory.audit_images(
                catalog,
                images,
                evidence,
                root / "runs",
                repository,
                root / "inventory.json",
            )

        self.assertEqual(result["summary"]["historical_dependency_count"], 1)
        self.assertEqual(result["summary"]["unknown_image_count"], 0)
        self.assertEqual(result["historical_images"][0]["catalog_tool"], "fastuniq")
        self.assertEqual(
            result["historical_images"][0]["disposition"],
            "historical_run_dependency",
        )

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
