"""Exercise Cargo artifact admission with bounded developer-bootstrap fixtures."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "makes/bin/developer_build_artifact.py"
BOOTSTRAP = ROOT / "makes/bin/dev_dna_bootstrap.sh"
SPEC = importlib.util.spec_from_file_location("bijux_developer_build_artifact", HELPER)
assert SPEC is not None and SPEC.loader is not None
ADMISSION = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(ADMISSION)


class CargoArtifactAdmissionTests(unittest.TestCase):
    """Refuse malformed, ambiguous, foreign or unsuccessful artifact evidence."""

    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/developer-bootstrap"
        scratch.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix="artifact-", dir=scratch)
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        self.executable = self.root / "debug/bijux-dna-dev"
        self.executable.parent.mkdir()
        self.executable.write_text("#!/bin/sh\nexit 0\n")
        self.executable.chmod(0o755)
        self.manifest = self.root / "Cargo.toml"
        self.messages = self.root / "cargo.jsonl"
        self.artifact = {
            "reason": "compiler-artifact",
            "target": {"name": "bijux-dna-dev", "kind": ["bin"]},
            "manifest_path": str(self.manifest),
            "executable": str(self.executable),
            "fresh": True,
        }
        self.finished = {"reason": "build-finished", "success": True}

    def verify(self, records: list[object]) -> None:
        self.messages.write_text("".join(json.dumps(row) + "\n" for row in records))
        ADMISSION.verify_executable(self.messages, self.executable, self.manifest)

    def test_fingerprint_noop_and_fresh_build_are_admitted(self) -> None:
        self.verify([self.artifact, self.finished])
        self.verify([dict(self.artifact, fresh=False), self.finished])

    def test_invalid_artifact_records_fail_closed(self) -> None:
        cases = [
            [],
            [self.finished],
            [self.artifact],
            [self.artifact, dict(self.finished, success=False)],
            [self.artifact, dict(self.finished, success=1)],
            [self.artifact, self.finished, self.finished],
            [self.artifact, self.artifact, self.finished],
            [dict(self.artifact, executable=None), self.finished],
            [dict(self.artifact, executable=""), self.finished],
            [dict(self.artifact, executable=str(self.root / "other")), self.finished],
            [
                dict(self.artifact, manifest_path=str(self.root / "foreign")),
                self.finished,
            ],
            [
                dict(self.artifact, target={"name": "other", "kind": ["bin"]}),
                self.finished,
            ],
            [
                dict(self.artifact, target={"name": "bijux-dna-dev", "kind": ["lib"]}),
                self.finished,
            ],
            [[], self.finished],
        ]
        for records in cases:
            with self.subTest(records=records), self.assertRaises(ValueError):
                self.verify(records)

    def test_malformed_messages_and_missing_executable_refuse(self) -> None:
        self.messages.write_text("not JSON\n")
        with self.assertRaises(ValueError):
            ADMISSION.verify_executable(self.messages, self.executable, self.manifest)
        self.executable.chmod(0o644)
        with self.assertRaises(ValueError):
            self.verify([self.artifact, self.finished])
        self.executable.unlink()
        with self.assertRaises(ValueError):
            self.verify([self.artifact, self.finished])


class DeveloperBootstrapProcessTests(unittest.TestCase):
    """Use dependency-free Cargo fixtures without executing scientific products."""

    def setUp(self) -> None:
        scratch = ROOT / "artifacts/tests/developer-bootstrap"
        scratch.mkdir(parents=True, exist_ok=True)
        self.directory = tempfile.TemporaryDirectory(prefix="workspace-", dir=scratch)
        self.addCleanup(self.directory.cleanup)
        self.root = Path(self.directory.name)
        (self.root / "crates/bijux-dna-dev/src").mkdir(parents=True)
        (self.root / "crates/domain/src").mkdir(parents=True)
        (self.root / "Cargo.toml").write_text(
            '[workspace]\nmembers=["crates/bijux-dna-dev", "crates/domain"]\nresolver="2"\n'
            '[profile.ci]\ninherits="dev"\ndebug=0\nincremental=false\n'
        )
        (self.root / "crates/bijux-dna-dev/Cargo.toml").write_text(
            '[package]\nname="bijux-dna-dev"\nversion="0.1.0"\nedition="2021"\n'
            '[dependencies]\ndomain={path="../domain"}\n'
        )
        (self.root / "crates/domain/Cargo.toml").write_text(
            '[package]\nname="domain"\nversion="0.1.0"\nedition="2021"\n'
        )
        (self.root / "crates/bijux-dna-dev/src/main.rs").write_text(
            'fn main() { println!("{}", domain::value()); }\n'
        )
        self.source = self.root / "crates/domain/src/lib.rs"
        self.source.write_text("pub fn value() -> u8 { 1 }\n")
        self.binary = self.root / "compiled artifacts/debug/bijux-dna-dev"
        self.env = dict(os.environ)
        for name in ["CARGO_BUILD_TARGET", "CARGO_TARGET_DIR", "RUSTC_WRAPPER"]:
            self.env.pop(name, None)
        self.env["CARGO_HOME"] = str(ROOT / "artifacts/rust/cargo/home")
        subprocess.run(
            ["cargo", "generate-lockfile", "--offline"],
            cwd=self.root,
            env=self.env,
            check=True,
            capture_output=True,
        )

    def run_bootstrap(
        self, path: Path | None = None, profile: str | None = None
    ) -> subprocess.CompletedProcess[str]:
        return subprocess.run(
            ["sh", str(BOOTSTRAP), str(path or self.binary), *([profile] if profile else [])],
            cwd=self.root,
            env=self.env,
            capture_output=True,
            text=True,
        )

    def test_ci_profile_uses_cargo_declared_artifact_and_refuses_wrong_profile_path(self):
        domain_manifest = self.root / "crates/domain/Cargo.toml"
        domain_manifest.write_text(
            domain_manifest.read_text() + '[features]\nobserved=[]\n'
        )
        self.source.write_text(
            'pub fn value() -> u8 { if cfg!(feature = "observed") { 9 } else { 1 } }\n'
        )
        default = self.run_bootstrap()
        self.assertEqual(default.returncode, 0, default.stderr)
        self.assertEqual(subprocess.check_output([str(self.binary)]), b"1\n")
        binary = self.root / "compiled artifacts/ci/bijux-dna-dev"
        result = self.run_bootstrap(binary, "ci")
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertEqual(subprocess.check_output([str(binary)], text=True).strip(), "9")
        restored = self.run_bootstrap()
        self.assertEqual(restored.returncode, 0, restored.stderr)
        self.assertEqual(subprocess.check_output([str(self.binary)]), b"1\n")
        self.assertNotEqual(self.run_bootstrap(binary, "dev").returncode, 0)
        self.assertNotEqual(self.run_bootstrap(binary, "unregistered").returncode, 0)

    def host(self) -> str:
        output = subprocess.check_output(["rustc", "-vV"], text=True, env=self.env)
        return next(
            line.removeprefix("host: ")
            for line in output.splitlines()
            if line.startswith("host: ")
        )

    def stale_binary(self) -> bytes:
        self.binary.parent.mkdir(parents=True, exist_ok=True)
        self.binary.write_text("#!/bin/sh\necho stale\n")
        self.binary.chmod(0o755)
        return self.binary.read_bytes()

    def test_transitive_rebuild_noop_and_compile_failure(self) -> None:
        first = self.run_bootstrap()
        self.assertEqual(first.returncode, 0, first.stderr)
        self.assertEqual(subprocess.check_output([str(self.binary)]), b"1\n")
        self.source.write_text("pub fn value() -> u8 { 2 }\n")
        second = self.run_bootstrap()
        self.assertEqual(second.returncode, 0, second.stderr)
        self.assertEqual(subprocess.check_output([str(self.binary)]), b"2\n")
        stamp = self.binary.stat().st_mtime_ns
        noop = self.run_bootstrap()
        self.assertEqual(noop.returncode, 0, noop.stderr)
        self.assertEqual(self.binary.stat().st_mtime_ns, stamp)
        self.source.write_text("not Rust\n")
        self.assertNotEqual(self.run_bootstrap().returncode, 0)

    def test_environment_target_cannot_hide_stale_output(self) -> None:
        old = self.stale_binary()
        host = self.host()
        self.env["CARGO_BUILD_TARGET"] = host
        result = self.run_bootstrap()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match", result.stderr)
        self.assertEqual(self.binary.read_bytes(), old)
        self.assertTrue(
            (self.binary.parent.parent / host / "debug/bijux-dna-dev").is_file()
        )

    def test_configured_target_cannot_hide_stale_output(self) -> None:
        old = self.stale_binary()
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo/config.toml").write_text(
            f'[build]\ntarget="{self.host()}"\n'
        )
        result = self.run_bootstrap()
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("does not match", result.stderr)
        self.assertEqual(self.binary.read_bytes(), old)

    def test_requested_directory_overrides_configured_directory(self) -> None:
        (self.root / ".cargo").mkdir()
        (self.root / ".cargo/config.toml").write_text(
            '[build]\ntarget-dir="other-target"\n'
        )
        result = self.run_bootstrap()
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertTrue(self.binary.is_file())
        self.assertFalse((self.root / "other-target").exists())

    def test_unrelated_output_path_refuses_before_compilation(self) -> None:
        result = self.run_bootstrap(self.root / "unrelated")
        self.assertEqual(result.returncode, 2)
        self.assertFalse(self.binary.parent.exists())


if __name__ == "__main__":
    unittest.main()
