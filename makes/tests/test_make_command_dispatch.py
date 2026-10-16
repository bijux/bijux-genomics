"""Bind lab and benchmark-sync Make wrappers to native registry operations."""

from __future__ import annotations

import json
import os
from pathlib import Path
import re
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
OPS = ROOT / "crates/bijux-dna-dev/src/catalog/ops.rs"
SYNC_ENVIRONMENT = (
    "BENCHMARK_SYNC_CLEAN_CONTEXT BENCHMARK_SYNC_ALLOW_DIRTY "
    "BENCHMARK_SYNC_PULL_BASE BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST "
    "BENCHMARK_SYNC_DATA_MANIFEST_GLOB BENCHMARK_SYNC_MODE BENCHMARK_SYNC_PULL_DEST"
).split()


class MakeCommandDispatchTests(unittest.TestCase):
    """Observe wrapper argv with fake Cargo; never execute lab or HPC operations."""

    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/make-command-dispatch"
        scratch.mkdir(parents=True, exist_ok=True)
        self.owner = tempfile.TemporaryDirectory(dir=scratch, prefix="workspace-")
        self.addCleanup(self.owner.cleanup)
        self.root = Path(self.owner.name)
        source = OPS.read_text()
        registry = {}
        for group in ["lab", "hpc"]:
            start = source.index(f"pub fn {group}_registry()")
            end = source.index("\n}", start)
            registry[group] = re.findall(r'native\(\s*"([^"]+)"', source[start:end])
            self.assertTrue(registry[group], group)
        self.assertIn("run-pipelines", registry["lab"])
        self.assertIn("run-bench", registry["lab"])
        self.assertIn("benchmark-sync-push", registry["hpc"])
        self.assertIn("benchmark-sync-pull", registry["hpc"])
        self.registry = self.root / "registry.json"
        self.registry.write_text(json.dumps(registry))
        self.calls = self.root / "calls.json"
        self.bin = self.root / "bin"
        self.bin.mkdir()
        cargo = self.bin / "cargo"
        cargo.write_text(
            f"#!{sys.executable}\n"
            "import json, os, sys\nfrom pathlib import Path\n"
            "args = sys.argv[1:]\n"
            "registry = json.loads(Path(os.environ['MOCK_REGISTRY']).read_text())\n"
            "if args[:5] != ['run', '-q', '-p', 'bijux-dna-dev', '--']: sys.exit(64)\n"
            "group, action, operation = args[5:8]\n"
            "if action != 'run' or operation not in registry.get(group, []): sys.exit(65)\n"
            "record = dict(argv=args, environment={k: os.environ.get(k) for k in "
            f"{(SYNC_ENVIRONMENT + ['CONFIG_PATH'])!r}}})\n"
            "Path(os.environ['MOCK_CALLS']).write_text(json.dumps(record))\n"
            "sys.exit(int(os.environ.get('MOCK_CARGO_EXIT', '0')))\n"
        )
        cargo.chmod(0o755)
        ssh = self.bin / "ssh"
        ssh.write_text("#!/bin/sh\necho 'unexpected SSH invocation' >&2\nexit 99\n")
        ssh.chmod(0o755)
        self.env = os.environ.copy()
        for name in ["MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES", *SYNC_ENVIRONMENT]:
            self.env.pop(name, None)
        self.env.update(PATH=str(self.bin) + os.pathsep + self.env["PATH"],
                        MOCK_REGISTRY=str(self.registry), MOCK_CALLS=str(self.calls),
                        CONFIG_PATH=str(self.root / "lab.toml"))
        self.makefile = self.root / "Makefile"
        self.makefile.write_text(
            f"SHELL := bash\ninclude {ROOT / 'makes/lab.mk'}\n"
            f"include {ROOT / 'makes/lunarc.mk'}\n"
        )
        self.settings = {
            "CLEAN_CONTEXT": "0", "ALLOW_DIRTY": "1",
            "BENCHMARK_PULL_BASE_DEFAULT": "mirror base",
            "BENCHMARK_PULL_MODE_DEFAULT": "mirror",
            "INCLUDE_CONTAINERS_MANIFEST": "1",
            "DATA_MANIFEST_GLOB": "bench/fixtures/**",
            "BENCHMARK_INCLUDE_PROFILE_DEFAULT": "include scope",
            "BENCHMARK_EXCLUDE_PROFILE_DEFAULT": "exclude scope",
            "BENCHMARK_LOCAL_RESULTS_ROOT": str(self.root / "local results"),
        }

    def invoke(self, target: str) -> tuple[subprocess.CompletedProcess[str], dict]:
        self.calls.unlink(missing_ok=True)
        result = subprocess.run(
            ["make", "--no-print-directory", "-f", str(self.makefile), target,
             *[f"{key}={value}" for key, value in self.settings.items()]],
            cwd=self.root, env=self.env, capture_output=True, text=True, timeout=15,
        )
        return result, json.loads(self.calls.read_text()) if self.calls.exists() else {}

    def assert_operation(self, target: str, group: str, operation: str, extra: list[str]) -> dict:
        result, call = self.invoke(target)
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(call["argv"], ["run", "-q", "-p", "bijux-dna-dev", "--",
                                        group, "run", operation, *extra])
        return call["environment"]

    def test_lab_targets_use_the_registered_hyphenated_operations(self) -> None:
        for target, operation in [("_lab-fastq", "run-pipelines"), ("_lab-bam", "run-bench")]:
            with self.subTest(target=target):
                env = self.assert_operation(target, "lab", operation, [])
                self.assertEqual(env["CONFIG_PATH"], self.env["CONFIG_PATH"])

    def test_push_aliases_preserve_context_and_confirmation(self) -> None:
        for target in ["_benchmark-sync-push", "benchmark-sync-push", "push-lunarc",
                       "benchmark-sync-push-confirm", "push-lunarc-confirm"]:
            with self.subTest(target=target):
                extra = ["--confirm"] if target.endswith("-confirm") else []
                env = self.assert_operation(target, "hpc", "benchmark-sync-push", extra)
                self.assertEqual(env["BENCHMARK_SYNC_CLEAN_CONTEXT"], "0")
                self.assertEqual(env["BENCHMARK_SYNC_ALLOW_DIRTY"], "1")

    def test_pull_aliases_preserve_profiles_and_results_scope(self) -> None:
        for target in ["_benchmark-sync-pull", "benchmark-sync-pull", "pull-lunarc",
                       "_benchmark-sync-pull-results", "benchmark-sync-pull-results", "pull-lunarc-results"]:
            with self.subTest(target=target):
                env = self.assert_operation(target, "hpc", "benchmark-sync-pull",
                                            ["--include-profile", "include scope", "--exclude-profile", "exclude scope"])
                self.assertEqual(env["BENCHMARK_SYNC_PULL_BASE"], "mirror base")
                self.assertEqual(env["BENCHMARK_SYNC_INCLUDE_CONTAINERS_MANIFEST"], "1")
                self.assertEqual(env["BENCHMARK_SYNC_DATA_MANIFEST_GLOB"], "bench/fixtures/**")
                results = target.endswith("-results")
                self.assertEqual(env["BENCHMARK_SYNC_MODE"], "results" if results else "mirror")
                self.assertEqual(env["BENCHMARK_SYNC_PULL_DEST"],
                                 self.settings["BENCHMARK_LOCAL_RESULTS_ROOT"] if results else None)

    def test_native_command_failure_propagates(self) -> None:
        self.env["MOCK_CARGO_EXIT"] = "19"
        result, call = self.invoke("benchmark-sync-push-confirm")
        self.assertNotEqual(result.returncode, 0)
        self.assertEqual(call["argv"][-1], "--confirm")


if __name__ == "__main__":
    unittest.main()
