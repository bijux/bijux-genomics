"""Probe Make temporary-directory defaults without Cargo or writes outside fixtures."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MACROS = ROOT / "makes/_macro.mk"


class MakeArtifactEnvironmentTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/make-artifact-environment"
        scratch.mkdir(parents=True, exist_ok=True)
        owner = tempfile.TemporaryDirectory(dir=scratch, prefix="workspace-")
        self.addCleanup(owner.cleanup)
        self.root = Path(owner.name)
        self.artifacts = self.root / "artifacts"
        self.env = os.environ.copy()
        for name in ["MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES", "ARTIFACT_ROOT", "ARTIFACT_TMP_DIR", "TMPDIR", "TMP", "TEMP"]:
            self.env.pop(name, None)
        self.env["PYTHONDONTWRITEBYTECODE"] = "1"
        self.makefile = self.root / "Makefile"
        self.makefile.write_text(
            f"include {MACROS}\n.PHONY: inspect nested origins\n"
            f"inspect:\n\t@\"{sys.executable}\" -B -c 'import os,json; "
            'print(json.dumps({k:os.environ.get(k) for k in ["TMPDIR","TMP","TEMP"]}))' + "'\n"
            "nested:\n\t@$(MAKE) --no-print-directory inspect\n"
            "origins:\n\t@printf '%s\\n' '{\"TMPDIR\":\"$(origin TMPDIR)\",\"TMP\":\"$(origin TMP)\",\"TEMP\":\"$(origin TEMP)\"}'\n"
        )

    def inspect(self, *variables: str, nested: bool = False, environment_override: bool = False, origins: bool = False) -> dict[str, str]:
        args = ["make", "--no-print-directory"]
        if environment_override:
            args.append("-e")
        process = subprocess.run(
            [*args, "origins" if origins else "nested" if nested else "inspect", *variables],
            cwd=self.root, env=self.env, text=True, capture_output=True, timeout=15,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return json.loads(process.stdout)

    def ambient(self) -> None:
        for name in ["TMPDIR", "TMP", "TEMP"]:
            self.env[name] = str(self.root / "host-temporary" / name)

    def test_ambient_host_paths_rebase_and_nested_make_keeps_selection(self) -> None:
        self.ambient()
        for nested in [False, True]:
            with self.subTest(nested=nested):
                observed = self.inspect(nested=nested)
                self.assertEqual(observed, dict.fromkeys(["TMPDIR", "TMP", "TEMP"], str(self.artifacts / "tmp")))
        self.assertFalse((self.root / "host-temporary").exists())

    def test_missing_or_empty_environment_uses_artifact_defaults(self) -> None:
        for value in [None, ""]:
            with self.subTest(value=value):
                if value is not None:
                    self.env.update(dict.fromkeys(["TMPDIR", "TMP", "TEMP"], value))
                self.assertEqual(self.inspect(), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], str(self.artifacts / "tmp")))

    def test_owned_environment_paths_preserve_spaces_and_quotes(self) -> None:
        self.env.update({name: str(self.artifacts / f"{name} caller's scratch") for name in ["TMPDIR", "TMP", "TEMP"]})
        self.assertEqual(self.inspect(), {name: self.env[name] for name in ["TMPDIR", "TMP", "TEMP"]})

    def test_component_prefix_and_parent_traversal_do_not_gain_ambient_authority(self) -> None:
        for value in [str(self.root / "artifacts-other/tmp"), str(self.artifacts / "../outside")]:
            with self.subTest(value=value):
                self.env.update(dict.fromkeys(["TMPDIR", "TMP", "TEMP"], value))
                self.assertEqual(self.inspect(), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], str(self.artifacts / "tmp")))

    def test_explicit_make_scratch_selection_survives(self) -> None:
        self.ambient()
        selected = str(self.root / "declared-scratch")
        self.assertEqual(self.inspect(f"TMPDIR={selected}"), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], selected))
        explicit = {name: str(self.root / ("explicit-" + name)) for name in ["TMPDIR", "TMP", "TEMP"]}
        self.assertEqual(self.inspect(*[f"{name}={value}" for name, value in explicit.items()]), explicit)

    def test_explicit_artifact_scratch_authority_is_the_fallback(self) -> None:
        self.ambient()
        scratch = str(self.root / "declared-scratch")
        self.assertEqual(self.inspect(f"ARTIFACT_TMP_DIR={scratch}"), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], scratch))
        self.env["TMPDIR"] = scratch + "/existing"
        self.assertEqual(self.inspect(f"ARTIFACT_TMP_DIR={scratch}"), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], scratch + "/existing"))

    def test_normalized_environment_is_not_an_explicit_make_override(self) -> None:
        self.ambient()
        self.assertEqual(self.inspect(origins=True), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], "file"))
        self.assertEqual(self.inspect(origins=True, environment_override=True), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], "override"))
        explicit = [f"{name}={self.artifacts / name}" for name in ["TMPDIR", "TMP", "TEMP"]]
        self.assertEqual(self.inspect(*explicit, origins=True), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], "command line"))

    def test_current_directory_is_shell_quoted(self) -> None:
        makefile = self.makefile.read_text()
        self.root = self.root / "cwd $(touch unexpected-shell-write) ' quoted"
        self.root.mkdir()
        self.artifacts = self.root / "artifacts"
        self.makefile = self.root / "Makefile"
        self.makefile.write_text(makefile)
        self.ambient()
        self.assertEqual(self.inspect(), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], str(self.artifacts / "tmp")))
        self.assertFalse((self.root / "unexpected-shell-write").exists())

    def test_environment_precedence_cannot_restore_ambient_host_paths(self) -> None:
        self.ambient()
        self.assertEqual(self.inspect(environment_override=True), dict.fromkeys(["TMPDIR", "TMP", "TEMP"], str(self.artifacts / "tmp")))


if __name__ == "__main__":
    unittest.main()
