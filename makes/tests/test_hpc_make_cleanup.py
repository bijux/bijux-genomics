"""Admit HPC cleanup paths using owned local fixtures and fake commands."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
LUNARC_MAKE = ROOT / "makes/lunarc.mk"


class HpcMakeCleanupTests(unittest.TestCase):
    """No test contacts an HPC host or removes data outside its own fixture."""

    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/hpc-make-cleanup"
        scratch.mkdir(parents=True, exist_ok=True)
        self.owner = tempfile.TemporaryDirectory(dir=scratch, prefix="workspace-")
        self.addCleanup(self.owner.cleanup)
        self.root = Path(self.owner.name)
        self.sentinel = self.root / "frontend-repo"
        self.sentinel.mkdir()
        self.containers = self.root / "containers"
        self.selected = self.containers / "apptainer"
        self.selected.mkdir(parents=True)
        (self.selected / "owned.sif").write_text("owned output")
        self.sibling = self.containers / "base/retain.sif"
        self.sibling.parent.mkdir()
        self.sibling.write_text("retain")
        self.calls = self.root / "rm-calls.json"
        self.bin = self.root / "bin"
        self.bin.mkdir()
        self.env = dict(os.environ)
        for key in ["MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES"]:
            self.env.pop(key, None)
        self.env.update(PATH=str(self.bin) + os.pathsep + self.env["PATH"],
                        CLEANUP_FIXTURE=str(self.root), CLEANUP_CALLS=str(self.calls))
        fake_rm = self.bin / "rm"
        fake_rm.write_text(
            f"#!{sys.executable}\n"
            "import json, os, shutil, sys\nfrom pathlib import Path\n"
            "destination = Path(sys.argv[-1])\n"
            "owner = Path(os.environ['CLEANUP_FIXTURE']).resolve()\n"
            "assert owner in destination.resolve().parents\n"
            "Path(os.environ['CLEANUP_CALLS']).write_text(json.dumps(sys.argv[1:]))\n"
            "failure = int(os.environ.get('CLEANUP_FAILURE', '0'))\n"
            "if failure: sys.exit(failure)\n"
            "if destination.exists(): shutil.rmtree(destination)\n"
        )
        fake_rm.chmod(0o755)
        for name in ["ssh", "cargo"]:
            path = self.bin / name
            path.write_text("#!/bin/sh\necho 'unexpected remote or Cargo invocation' >&2\nexit 99\n")
            path.chmod(0o755)
        self.makefile = self.root / "Makefile"
        self.makefile.write_text(f"SHELL := bash\ninclude {LUNARC_MAKE}\n")

    def invoke(self, selected: Path | str | None = None, *, base: Path | str | None = None) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["make", "--no-print-directory", "-f", str(self.makefile), "apptainer-hpc-clean",
             f"BENCHMARK_REMOTE_CONTAINERS_ROOT={base if base is not None else self.containers}",
             f"LUNARC_APPTAINER_DIR={selected if selected is not None else self.selected}",
             f"LUNARC_FRONTEND_SENTINEL={self.sentinel}"],
            cwd=self.root, env=self.env, capture_output=True, text=True, timeout=15,
        )

    def test_owned_output_is_removed_without_touching_sibling_cache(self) -> None:
        result = self.invoke()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(self.selected.exists())
        self.assertEqual(self.sibling.read_text(), "retain")
        self.assertEqual(json.loads(self.calls.read_text()), ["-rf", "--", str(self.selected)])

    def test_custom_owned_descendant_is_supported(self) -> None:
        selected = self.containers / "builds/apptainer"
        selected.mkdir(parents=True)
        result = self.invoke(selected)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(selected.exists())
        self.assertTrue(self.selected.is_dir())

    def test_missing_frontend_refuses_before_cleanup(self) -> None:
        self.sentinel.rmdir()
        result = self.invoke()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("off frontend", result.stderr)
        self.assertFalse(self.calls.exists())
        self.assertTrue((self.selected / "owned.sif").is_file())

    def test_root_parent_prefix_relative_and_empty_paths_refuse(self) -> None:
        invalid = [self.containers, self.root, Path("/"),
                   Path(str(self.containers) + "-other/apptainer"), "containers/apptainer", ""]
        for selected in invalid:
            with self.subTest(selected=selected):
                result = self.invoke(selected)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.calls.exists())
                self.assertTrue((self.selected / "owned.sif").is_file())
        for base in [Path("/"), "containers", ""]:
            with self.subTest(base=base):
                self.assertNotEqual(self.invoke(base=base).returncode, 0)
                self.assertFalse(self.calls.exists())

    def test_symlink_destination_and_parent_escape_refuse(self) -> None:
        outside = self.root / "outside"
        (outside / "apptainer").mkdir(parents=True)
        alias = self.containers / "alias"
        alias.symlink_to(self.selected, target_is_directory=True)
        escape = self.containers / "escape"
        escape.symlink_to(outside, target_is_directory=True)
        for selected in [alias, escape / "apptainer"]:
            with self.subTest(selected=selected):
                result = self.invoke(selected)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse(self.calls.exists())
                self.assertTrue(self.selected.is_dir())
                self.assertTrue((outside / "apptainer").is_dir())

    def test_cleanup_failure_is_not_reported_as_success(self) -> None:
        self.env["CLEANUP_FAILURE"] = "23"
        result = self.invoke()
        self.assertNotEqual(result.returncode, 0)
        self.assertNotIn("removed ", result.stdout)
        self.assertTrue((self.selected / "owned.sif").is_file())


if __name__ == "__main__":
    unittest.main()
