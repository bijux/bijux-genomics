"""Check developer Make delegation and cleanup without building Rust products."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
DEVELOPER_MAKE = ROOT / "makes/cargo-dev.mk"
MACROS = ROOT / "makes/_macro.mk"
SHARED = ROOT / ".bijux/shared"
PATH_VARIABLES = (
    "ARTIFACT_TARGET_DIR ARTIFACT_CARGO_HOME ARTIFACT_TMP_DIR CARGO_TARGET_DIR "
    "CARGO_HOME TMPDIR TMP TEMP TEST_TMP_DIR ISO_ROOT ARTIFACT_DIR "
    "PYTHONPYCACHEPREFIX RUFF_CACHE_DIR PYTEST_DEBUG_TEMPROOT ARTIFACTS_DIR "
    "RS_ARTIFACT_ROOT RS_TARGET_DIR RS_CARGO_HOME RS_TMP_DIR RS_NEXTEST_CACHE_DIR "
    "RS_NEXTEST_CONFIG_HOME NEXTEST_CACHE_DIR DEV_DNA_BIN"
).split()


class DeveloperMakeEnvironmentTests(unittest.TestCase):
    """Exercise actual Make exports against lightweight child gate recipes."""

    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/developer-make"
        scratch.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(dir=scratch, prefix="workspace-")
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.report = self.root / "observed.json"
        self.selected = self.root / "artifacts/selected"
        self.env = dict(os.environ)
        for name in [*PATH_VARIABLES, "ARTIFACT_ROOT", "DEV_ARTIFACT_ROOT", "MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES"]:
            self.env.pop(name, None)
        self.child = self.root / "child.mk"
        self.child.write_text(
            f"BIJUX_MAKES_SHARED_ROOT := {SHARED}\n"
            f"include {SHARED}/bijux-makes/environment.mk\n"
            f"include {MACROS}\n"
            f"include {SHARED}/bijux-makes-rs/bijux.mk\n"
            f"include {DEVELOPER_MAKE}\n"
            ".PHONY: _test-fast _fmt _lint _lint-automation _audit\n"
            "_test-fast _fmt _lint _lint-automation _audit:\n"
            f"\t@{sys.executable} -c 'import json,os; from pathlib import Path; "
            f"Path({json.dumps(str(self.report))}).write_text(json.dumps(dict(os.environ)))'\n"
        )
        # The included form reproduces the parent artifact exports in root.mk.
        self.parent = self.root / "parent.mk"
        self.parent.write_text(f"include {MACROS}\ninclude {DEVELOPER_MAKE}\n")

    def invoke(self, *arguments: str, included: bool = False) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["make", "--no-print-directory", "-f", str(self.parent if included else DEVELOPER_MAKE),
             f"ROOT_MAKE={self.child}", *arguments],
            cwd=self.root, env=self.env, capture_output=True, text=True, timeout=20,
        )

    def observed(self, process: subprocess.CompletedProcess[str]) -> dict[str, str]:
        self.assertEqual(process.returncode, 0, process.stdout + process.stderr)
        return json.loads(self.report.read_text())

    def test_custom_root_rebases_all_inherited_artifact_paths(self) -> None:
        for included in [False, True]:
            with self.subTest(included=included):
                for name in PATH_VARIABLES:
                    self.env[name] = str(self.root / "artifacts/prior" / name.lower())
                values = self.observed(self.invoke("dev-test", f"DEV_ARTIFACT_ROOT={self.selected}", included=included))
                self.assertEqual(values["ARTIFACT_ROOT"], str(self.selected))
                self.assertEqual(values["ISO_ROOT"], str(self.selected))
                for name in PATH_VARIABLES:
                    if name in values:
                        self.assertTrue(Path(values[name]).is_relative_to(self.selected), (name, values[name]))
                self.assertEqual(values["CARGO_TARGET_DIR"], str(self.selected / "target"))
                self.assertEqual(values["CARGO_HOME"], str(self.selected / "cargo/home"))
                self.assertEqual(values["TMPDIR"], str(self.selected / "tmp"))

    def test_valid_environment_and_command_line_overrides_are_preserved(self) -> None:
        for name in PATH_VARIABLES:
            self.env[name] = str(self.selected / "explicit" / name.lower())
        values = self.observed(self.invoke("dev-test", f"DEV_ARTIFACT_ROOT={self.selected}",
                                          f"CARGO_TARGET_DIR={self.selected}/command-target"))
        self.assertEqual(values["CARGO_TARGET_DIR"], str(self.selected / "command-target"))
        for name in PATH_VARIABLES:
            if name not in {"CARGO_TARGET_DIR", "ISO_ROOT"}:
                self.assertEqual(values[name], self.env[name], name)
        self.assertEqual(values["ISO_ROOT"], str(self.selected))

    def test_invalid_command_line_override_refuses_before_child(self) -> None:
        result = self.invoke("dev-test", f"DEV_ARTIFACT_ROOT={self.selected}",
                             f"CARGO_TARGET_DIR={self.root}/artifacts/other-target")
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("override escapes selected root", result.stderr)
        self.assertFalse(self.report.exists())

    def test_artifact_root_falls_back_to_callers_selected_root(self) -> None:
        values = self.observed(self.invoke("dev-test", f"ARTIFACT_ROOT={self.selected}"))
        self.assertEqual(values["ARTIFACT_ROOT"], str(self.selected))
        self.assertEqual(values["CARGO_TARGET_DIR"], str(self.selected / "target"))

    def test_aliases_forward_once_without_returning_to_prior_root(self) -> None:
        for target in ["dev-fmt", "dev-lint", "dev-audit", "dev-lint-scripts"]:
            with self.subTest(target=target):
                values = self.observed(self.invoke(target, f"DEV_ARTIFACT_ROOT={self.selected}"))
                self.assertEqual(values["ARTIFACT_ROOT"], str(self.selected))

    def test_clean_removes_only_selected_scratch_and_recreates_it(self) -> None:
        selected_tmp = self.selected / "tmp"
        selected_tmp.mkdir(parents=True)
        (selected_tmp / "scratch").write_text("remove")
        other = self.root / "artifacts/tmp/keep"
        other.parent.mkdir(parents=True)
        other.write_text("retain")
        result = self.invoke("dev-clean", f"DEV_ARTIFACT_ROOT={self.selected}")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(list(selected_tmp.iterdir()), [])
        self.assertEqual(other.read_text(), "retain")

    @unittest.skipUnless(hasattr(os, "symlink"), "symlink support required")
    def test_clean_refuses_root_or_symlink_escape_before_deletion(self) -> None:
        outside = self.root / "outside"
        (outside / "tmp").mkdir(parents=True)
        marker = outside / "tmp/retain"
        marker.write_text("retain")
        (self.root / "artifacts/link").parent.mkdir(parents=True)
        (self.root / "artifacts/link").symlink_to(outside, target_is_directory=True)
        for selected in [self.root, self.root / "artifacts/link"]:
            with self.subTest(selected=selected):
                result = self.invoke("dev-clean", f"DEV_ARTIFACT_ROOT={selected}")
                self.assertNotEqual(result.returncode, 0)
                self.assertIn("must stay under workspace artifacts/", result.stderr)
                self.assertEqual(marker.read_text(), "retain")


if __name__ == "__main__":
    unittest.main()
