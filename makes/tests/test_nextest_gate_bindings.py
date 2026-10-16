"""Exercise ordinary Rust gate binary preparation using local fake programs only."""

import json
import os
from pathlib import Path
import subprocess
import shutil
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "makes/bin/run_nextest_gate.py"


class NextestGateBindingsTests(unittest.TestCase):
    def fixture(self, root):
        tools = root / "tools"
        tools.mkdir()
        cargo = tools / "cargo"
        cargo.write_text(f"#!{sys.executable}\n" + '''import json, os, pathlib, sys
args = sys.argv[1:]
with open(os.environ["CALLS"], "a") as log:
    log.write(json.dumps({"args": args, "home": os.environ["CARGO_HOME"], "tmp": os.environ["TMPDIR"]}) + "\\n")
mode = os.environ.get("INVENTORY_MODE", "valid")
if mode == "failed":
    print("producer refused", file=sys.stderr)
    sys.exit(7)
if mode == "malformed":
    print("not json")
    sys.exit(0)
target = pathlib.Path(args[args.index("--target-dir") + 1])
entries = {}
for name in ("bijux-dna", "bijux-dna-dev"):
    path = target / "debug" / name
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text("native fixture")
    path.chmod(0o755)
    entries[name] = [{"name": name, "kind": "bin-exe", "path": "debug/" + name}]
if mode == "missing": del entries["bijux-dna"]
if mode == "duplicate": entries["duplicate"] = entries["bijux-dna"]
if mode == "wrong-kind": entries["bijux-dna"][0]["kind"] = "lib"
if mode == "traversal": entries["bijux-dna"][0]["path"] = "../bijux-dna"
if mode == "absent": (target / "debug/bijux-dna").unlink()
if mode == "wrong-target": target = target.parent
print(json.dumps({"rust-build-meta": {"target-directory": str(target), "non-test-binaries": entries}}))
''')
        cargo.chmod(0o755)
        shared = root / "shared gate"
        shared.write_text(f"#!{sys.executable}\n" + '''import json, os, sys
with open(os.environ["SHARED_CALL"], "w") as output:
    json.dump({"args": sys.argv[1:], "cli": os.environ["NEXTEST_BIN_EXE_bijux-dna"], "dev": os.environ["NEXTEST_BIN_EXE_bijux-dna-dev"]}, output)
sys.exit(int(os.environ.get("SHARED_STATUS", "0")))
''')
        shared.chmod(0o755)
        config = root / "custom nextest.toml"
        config.write_text("[profile.default]\n")
        environment = os.environ.copy()
        for key in list(environment):
            if key.startswith("RS_") or key.startswith("NEXTEST_"):
                environment.pop(key)
        environment.update({
            "PATH": str(tools), "PROJECT_ROOT": str(root),
            "ARTIFACT_ROOT": str(root / "artifacts"),
            "NEXTEST_CONFIG_FILE": str(config),
            "RS_TARGET_DIR": str(root / "artifacts/custom target"),
            "CALLS": str(root / "calls"), "SHARED_CALL": str(root / "shared.json"),
            "NEXTEST_BIN_EXE_bijux-dna": "/stale/cli",
            "NEXTEST_BIN_EXE_bijux-dna-dev": "/stale/dev",
        })
        return shared, environment

    def invoke(self, shared, environment, mode="test"):
        return subprocess.run(
            [sys.executable, "-B", str(HELPER), str(shared), mode, "unchanged-argument"],
            env=environment, capture_output=True, text=True, check=False,
        )

    def test_fresh_inventory_matches_shared_selection_and_replaces_stale_bindings(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products, prefix="custom workspace ") as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            environment["NEXTEST_PROFILE_FAST"] = "fast-unit"
            environment["TEST_FEATURES"] = "--no-default-features"
            for _ in range(2):
                result = self.invoke(shared, environment)
                self.assertEqual(result.returncode, 0, result.stderr)
            calls = [json.loads(line) for line in (root / "calls").read_text().splitlines()]
            self.assertEqual(len(calls), 2)
            args = calls[0]["args"]
            self.assertEqual(args[:4], ["nextest", "list", "--workspace", "--all-features"])
            self.assertIn("--locked", args)
            self.assertEqual(args[args.index("--config-file") + 1], environment["NEXTEST_CONFIG_FILE"])
            self.assertEqual(args[args.index("--profile") + 1], "fast-unit")
            self.assertEqual(args[args.index("--target-dir") + 1], environment["RS_TARGET_DIR"])
            self.assertEqual(args[-4:], ["--list-type", "binaries-only", "--message-format", "json"])
            self.assertNotIn("--no-default-features", args)
            self.assertTrue(Path(calls[0]["home"]).is_relative_to(root / "artifacts"))
            self.assertTrue(Path(calls[0]["tmp"]).is_relative_to(root / "artifacts"))
            selected = json.loads((root / "shared.json").read_text())
            self.assertEqual(selected["args"], ["test", "unchanged-argument"])
            self.assertEqual(selected["cli"], str(Path(environment["RS_TARGET_DIR"]) / "debug/bijux-dna"))
            self.assertEqual(selected["dev"], str(Path(environment["RS_TARGET_DIR"]) / "debug/bijux-dna-dev"))

    def test_owned_shell_wrapper_dispatches_through_preparation(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            gate = root / ".bijux/shared/bijux-makes-rs/scripts/rust_gate.sh"
            gate.parent.mkdir(parents=True)
            shutil.copy2(shared, gate)
            helper = root / "makes/bin/run_nextest_gate.py"
            helper.parent.mkdir(parents=True)
            shutil.copy2(HELPER, helper)
            git = root / "tools/git"
            git.write_text(f"#!{sys.executable}\nimport os\nprint(os.environ['PROJECT_ROOT'])\n")
            git.chmod(0o755)
            environment["PYTHON"] = sys.executable
            result = subprocess.run(
                ["/bin/bash", str(ROOT / "makes/bin/run_genomics_rust_gate.sh"), "test"],
                env=environment, capture_output=True, text=True, check=False,
            )
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertTrue((root / "calls").is_file())
            self.assertEqual(json.loads((root / "shared.json").read_text())["args"], ["test"])

    def test_slow_profile_locked_override_and_shared_failure_are_preserved(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            environment.update(NEXTEST_PROFILE_SLOW="slow-integration", RUST_CARGO_LOCKED="0", SHARED_STATUS="9")
            result = self.invoke(shared, environment, "test-slow")
            self.assertEqual(result.returncode, 9, result.stderr)
            args = json.loads((root / "calls").read_text())["args"]
            self.assertNotIn("--locked", args)
            self.assertEqual(args[args.index("--profile") + 1], "slow-integration")

    def test_failed_or_unusable_inventory_never_dispatches_shared_gate(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        for mode in ("failed", "malformed", "missing", "duplicate", "wrong-kind", "traversal", "absent", "wrong-target"):
            with self.subTest(mode=mode), tempfile.TemporaryDirectory(dir=products) as directory:
                root = Path(directory)
                shared, environment = self.fixture(root)
                environment["INVENTORY_MODE"] = mode
                result = self.invoke(shared, environment)
                self.assertNotEqual(result.returncode, 0)
                self.assertFalse((root / "shared.json").exists())
                if mode == "failed":
                    self.assertEqual(result.returncode, 7)
                    self.assertIn("producer refused", result.stderr)

    def test_inventory_log_symlink_refuses_before_cargo_or_external_write(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            foreign = root / "foreign"
            foreign.mkdir()
            rust = root / "artifacts/rust"
            rust.mkdir(parents=True)
            (rust / "test").symlink_to(foreign, target_is_directory=True)
            result = self.invoke(shared, environment)
            self.assertNotEqual(result.returncode, 0)
            self.assertEqual(list(foreign.iterdir()), [])
            self.assertFalse((root / "calls").exists())

    def test_empty_lock_setting_matches_managed_default_and_bad_run_id_refuses(self):
        import importlib.util
        spec = importlib.util.spec_from_file_location("nextest_gate_bindings", HELPER)
        helper = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(helper)
        args = helper.list_arguments({"NEXTEST_CONFIG_FILE": "config", "RUST_CARGO_LOCKED": ""}, "test", Path("target"))
        self.assertIn("--locked", args)
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            environment["RS_RUN_ID"] = "../foreign"
            result = self.invoke(shared, environment)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse((root / "calls").exists())

    def test_foreign_artifact_descendant_refuses_before_cargo_or_directory_creation(self):
        products = ROOT / "artifacts/tests/nextest-gate-bindings"
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            shared, environment = self.fixture(root)
            foreign = root / "foreign-target"
            environment["RS_TARGET_DIR"] = str(foreign)
            result = self.invoke(shared, environment)
            self.assertNotEqual(result.returncode, 0)
            self.assertFalse(foreign.exists())
            self.assertFalse((root / "calls").exists())


if __name__ == "__main__":
    unittest.main()
