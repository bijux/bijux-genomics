"""Exercise Make runtime selection through real child-process environments."""

from __future__ import annotations

import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
MAKE = shutil.which("make")


@unittest.skipUnless(MAKE, "make is required")
class MakeRuntimeEnvironmentTests(unittest.TestCase):
    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/make-runtime"
        scratch.mkdir(parents=True, exist_ok=True)
        owner = tempfile.TemporaryDirectory(prefix="workspace-", dir=scratch)
        self.addCleanup(owner.cleanup)
        self.root = Path(owner.name)
        self.artifacts = self.root / "artifacts"
        self.host = self.root / "host-bin"
        self.host.mkdir()
        self.program(self.host / "bcftools", "bcftools 1.21")
        self.program(self.host / "bash", "host bash")
        self.environment = {
            key: value
            for key, value in os.environ.items()
            if not key.startswith("MAKE")
            and key
            not in {
                "MFLAGS",
                "PYTHON",
                "BIJUX_GATE_TEST_BASH",
                "BIJUX_TEST_RUNTIME_ROOT",
            }
        }
        self.environment.update(
            PATH=f"{self.host}:/usr/bin:/bin", HOME=str(self.root / "caller-home")
        )
        self.inspector = self.root / "inspect.py"
        self.inspector.write_text(
            "import json, os, shutil, subprocess\n"
            "print(json.dumps({'path': os.environ['PATH'], 'home': os.environ['HOME'], "
            "'python': os.environ.get('PYTHON'), 'bash_override': os.environ.get('BIJUX_GATE_TEST_BASH'), "
            "'bcftools': shutil.which('bcftools'), 'bash': shutil.which('bash'), "
            "'version': subprocess.check_output(['bcftools', '--version'], text=True).strip()}))\n"
        )
        self.makefile = self.root / "Makefile"
        self.makefile.write_text(
            f"include {ROOT}/makes/_macro.mk\n"
            f"include {ROOT}/makes/runtime.mk\n"
            f"include {ROOT}/makes/python.mk\n"
            ".PHONY: inspect nested twice\n"
            f'inspect:\n\t@"{sys.executable}" "{self.inspector}"\n'
            "nested:\n\t@$(MAKE) --no-print-directory inspect\n"
            "twice:\n\t@$(MAKE) --no-print-directory nested\n"
        )

    def program(self, path: Path, output: str) -> Path:
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"#!/bin/sh\nprintf '%s\\n' '{output}'\n")
        path.chmod(0o755)
        return path

    def provision(self, runtime: Path | None = None) -> tuple[Path, Path]:
        runtime = runtime or self.artifacts / "runtime"
        bcftools = self.program(
            runtime / "bcftools/install/bin/bcftools", "bcftools 1.20"
        )
        bash = self.program(runtime / "bash/install/bin/bash", "owned bash")
        return bcftools, bash

    def invoke(
        self, target: str = "inspect", *variables: str
    ) -> subprocess.CompletedProcess[str]:
        process = subprocess.run(
            [MAKE, "--no-print-directory", target, *variables],
            cwd=self.root,
            env=self.environment,
            capture_output=True,
            text=True,
            timeout=15,
        )
        self.assertEqual(process.returncode, 0, process.stderr)
        return process

    def inspect(self, target: str = "inspect", *variables: str) -> dict[str, str]:
        return json.loads(self.invoke(target, *variables).stdout)

    def test_provisioned_tools_precede_host_and_keep_caller_home(self) -> None:
        bcftools, bash = self.provision()
        python = self.program(
            self.artifacts / "python/checks-environment/bin/python3", "owned python"
        )
        observed = self.inspect()
        self.assertEqual(observed["bcftools"], str(bcftools))
        self.assertEqual(observed["version"], "bcftools 1.20")
        self.assertEqual(observed["bash"], str(bash))
        self.assertEqual(observed["bash_override"], str(bash))
        self.assertEqual(observed["python"], str(python))
        self.assertEqual(observed["home"], self.environment["HOME"])

    def test_missing_and_nonexecutable_tools_preserve_host_fallback(self) -> None:
        missing = self.inspect()
        self.assertEqual(missing["path"], self.environment["PATH"])
        self.assertEqual(missing["version"], "bcftools 1.21")
        self.provision()
        for path in (self.artifacts / "runtime").rglob("bin/*"):
            path.chmod(0o644)
        observed = self.inspect()
        self.assertEqual(observed["path"], self.environment["PATH"])
        self.assertEqual(observed["version"], "bcftools 1.21")
        self.assertEqual(observed["python"], "python3")
        self.assertIsNone(observed["bash_override"])

    def test_runtime_only_path_does_not_gain_a_current_directory_entry(self) -> None:
        bcftools, bash = self.provision()
        self.environment["PATH"] = f"{bcftools.parent}:{bash.parent}"
        observed = self.inspect()
        self.assertEqual(observed["path"], self.environment["PATH"])

    def test_command_line_path_keeps_runtime_precedence(self) -> None:
        bcftools, _ = self.provision()
        observed = self.inspect("inspect", f"PATH={self.environment['PATH']}")
        self.assertEqual(observed["bcftools"], str(bcftools))
        self.assertEqual(observed["version"], "bcftools 1.20")

    def test_custom_artifact_root_selects_its_runtime_and_interpreter(self) -> None:
        self.artifacts = self.root / "owned-artifacts"
        bcftools, _ = self.provision()
        python = self.program(
            self.artifacts / "python/checks-environment/bin/python3", "custom python"
        )
        observed = self.inspect("inspect", f"ARTIFACT_ROOT={self.artifacts}")
        self.assertEqual(observed["bcftools"], str(bcftools))
        self.assertEqual(observed["python"], str(python))

    def test_explicit_runtime_root_and_python_and_bash_overrides_survive(self) -> None:
        self.provision()
        runtime = self.root / "selected-runtime"
        bcftools, _ = self.provision(runtime)
        python = self.program(self.root / "chosen-python", "explicit python")
        bash = self.program(self.root / "chosen-bash", "explicit bash")
        variables = (
            f"BIJUX_TEST_RUNTIME_ROOT={runtime}",
            f"PYTHON={python}",
            f"BIJUX_GATE_TEST_BASH={bash}",
        )
        observed = self.inspect("inspect", *variables)
        self.assertEqual(observed["bcftools"], str(bcftools))
        self.assertEqual(observed["python"], str(python))
        self.assertEqual(observed["bash_override"], str(bash))
        self.assertEqual(
            self.invoke("_python-test", *variables).stdout.strip(), "explicit python"
        )

    def test_environment_interpreter_and_bash_overrides_are_exported(self) -> None:
        self.provision()
        self.environment.update(
            PYTHON="caller-python", BIJUX_GATE_TEST_BASH="caller-bash"
        )
        observed = self.inspect()
        self.assertEqual(observed["python"], "caller-python")
        self.assertEqual(observed["bash_override"], "caller-bash")

    def test_nested_make_keeps_one_runtime_prefix(self) -> None:
        bcftools, bash = self.provision()
        direct = self.inspect()
        nested = self.inspect("twice")
        self.assertEqual(nested, direct)
        self.assertEqual(nested["path"].split(":").count(str(bcftools.parent)), 1)
        self.assertEqual(nested["path"].split(":").count(str(bash.parent)), 1)

    def test_existing_runtime_entries_move_first_without_changing_other_entries(
        self,
    ) -> None:
        bcftools, bash = self.provision()
        other = str(self.root / "caller's bin")
        self.environment["PATH"] = (
            f":{self.host}:{bcftools.parent}:{other}:{bash.parent}:/usr/bin:/bin:"
        )
        observed = self.inspect()
        self.assertEqual(
            observed["path"],
            f"{bcftools.parent}:{bash.parent}::{self.host}:{other}:/usr/bin:/bin:",
        )
        self.assertEqual(observed["version"], "bcftools 1.20")


if __name__ == "__main__":
    unittest.main()
