"""Keep real Make gate dispatch and refusal independent of same-named files."""

from pathlib import Path
import os
import subprocess
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]


class MakeGateDispatchTests(unittest.TestCase):
    def test_gate_and_bootstrap_files_cannot_bypass_execution(self):
        products = ROOT / "artifacts/tests/make-gate-dispatch"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            (root / "Makefile").write_text(
                f"ensure_artifact_env = true\ninclude {ROOT / 'makes/cargo.mk'}\n"
                f"include {ROOT / '.bijux/shared/bijux-makes-rs/bijux.mk'}\n"
            )
            tool = root / "gate"
            tool.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$GATE_CALLS"\nexit 7\n')
            tool.chmod(0o755)
            env = os.environ.copy()
            for key in ("MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES"):
                env.pop(key, None)
            env["GATE_CALLS"] = str(root / "calls")
            targets = [
                "test-all",
                "test-all-rs",
                "_dev-dna-bin",
                "audit-policy-rs",
                "_test",
                "_audit",
                "_coverage",
                "_doctor",
                "test-preflight",
                "generate-configs",
                "vcf-certification",
            ]
            for target in targets:
                (root / target).touch()
            for target in (
                "test-all",
                "_dev-dna-bin",
                "audit-policy-rs",
                "_test",
                "test-preflight",
            ):
                with self.subTest(target=target):
                    (root / "calls").unlink(missing_ok=True)
                    result = subprocess.run(
                        [
                            "make",
                            "--no-print-directory",
                            target,
                            f"RUST_GATE_BIN={tool}",
                            f"DEV_DNA_BOOTSTRAP={tool}",
                            f"DEV_DNA_BIN={tool}",
                        ],
                        cwd=root,
                        env=env,
                        capture_output=True,
                        text=True,
                        timeout=10,
                    )
                    self.assertNotEqual(result.returncode, 0, result.stdout)
                    self.assertTrue((root / "calls").is_file(), result.stderr)

    def test_quick_gate_runs_format_clippy_tests_and_contracts(self):
        products = ROOT / "artifacts/tests/make-gate-dispatch"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            (root / "Makefile").write_text(f"include {ROOT / 'makes/cargo.mk'}\n")
            tool = root / "make-gate"
            tool.write_text('#!/bin/sh\nprintf "%s\\n" "$*" >> "$GATE_CALLS"\n')
            tool.chmod(0o755)
            env = os.environ.copy()
            for key in ("MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES"):
                env.pop(key, None)
            env["GATE_CALLS"] = str(root / "calls")
            result = subprocess.run(
                ["make", "--no-print-directory", "_quick", f"MAKE={tool}"],
                cwd=root,
                env=env,
                capture_output=True,
                text=True,
                timeout=10,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertEqual(
                (root / "calls").read_text().splitlines(),
                [
                    "fmt",
                    "lint",
                    "test",
                    "lint-workspace",
                    "_test-profile-invariants",
                    "_registry-lint",
                ],
            )
