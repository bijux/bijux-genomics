"""Exercise owned workflow admission and process supervision without Rust execution."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[2]
HELPER = ROOT / "makes/bin/github_workflow_gates.py"
_spec = importlib.util.spec_from_file_location("bijux_workflow_gates", HELPER)
assert _spec is not None and _spec.loader is not None
GATES = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(GATES)


def workflow(*commands: str) -> bytes:
    """Build quoted workflow keys with concrete named run steps."""
    return json.dumps(
        {
            "jobs": {
                "owned": {
                    "steps": [{"name": "gate", "run": command} for command in commands]
                }
            }
        }
    ).encode()


class WorkflowAdmissionTests(unittest.TestCase):
    """Keep unsupported execution and preparation boundaries fail closed."""

    def test_actual_retained_workflow(self) -> None:
        self.assertEqual(
            GATES.workflow_gates(
                (ROOT / ".github/workflows/ci.yml").read_bytes(), "actual"
            ),
            ["fmt", "lint", "audit"],
        )

    def test_quoted_and_block_keys_preserve_order_and_deduplicate(self) -> None:
        self.assertEqual(
            GATES.workflow_gates(
                workflow(
                    "make fmt", "set -euo pipefail\nmake lint\nmake audit", "make fmt"
                ),
                "quoted",
            ),
            ["fmt", "lint", "audit"],
        )
        self.assertEqual(
            GATES.workflow_gates(
                b"jobs:\n  owned:\n    steps:\n      - run: |\n          make test-all\n",
                "block",
            ),
            ["test-all"],
        )

    def test_malformed_empty_ambiguous_and_unsupported_refuse(self) -> None:
        cases = [
            b"jobs: [",
            b"jobs: {}",
            b"jobs: {}\njobs: {}",
            b"jobs:\n  a:\n    steps: wrong",
            b"jobs:\n  a:\n    uses: org/repo/workflow.yml@main",
            workflow("make fmt && make test"),
            workflow("make $TARGET"),
            workflow("make fmt lint"),
            workflow("env make fmt"),
            workflow("/usr/bin/make test"),
            workflow("gmake test"),
            workflow("make --directory=other fmt"),
            workflow("make _gate", "make gate"),
            b"jobs:\n  a:\n    steps:\n      - run: 7",
            b"jobs:\n  a:\n    steps:\n      - run: make fmt\n        uses: actions/a@main",
        ]
        for content in cases:
            with self.subTest(content=content), self.assertRaises(ValueError):
                GATES.workflow_gates(content, "negative")


class WorkflowProcessTests(unittest.TestCase):
    """Run real shell supervision using bounded executable fixtures."""

    def run_fixture(
        self,
        commands: tuple[str, ...],
        *,
        preparation_exit: int = 0,
        bootstrap_exit: int = 0,
        child_exit: int = 0,
        metrics_exit: int = 0,
        final_metrics_exit: int = 0,
    ) -> tuple[subprocess.CompletedProcess[str], list[dict], dict[str, str]]:
        products = (
            ROOT
            / "artifacts/delivery-admission/incoming-wrapper-admission/process-fixtures"
        )
        products.mkdir(parents=True, exist_ok=True)
        with tempfile.TemporaryDirectory(dir=products) as directory:
            root = Path(directory)
            for relative in [
                "makes/bin",
                ".github/workflows",
                ".github/scripts/workflow_execution",
                "bin",
            ]:
                (root / relative).mkdir(parents=True)
            for source in [
                "makes/bin/run_github_workflow_gate.sh",
                "makes/bin/github_workflow_gates.py",
                ".github/scripts/workflow_execution/yaml_io.py",
            ]:
                shutil.copyfile(ROOT / source, root / source)
            (root / ".github/workflows/ci.yml").write_bytes(workflow(*commands))
            calls = root / "calls"
            calls.mkdir()
            executable = root / "bin/fixture-tool"
            executable.write_text(
                "#!"
                + sys.executable
                + "\n"
                + "\n".join(
                    [
                        "import json,os,pathlib,sys",
                        "name=pathlib.Path(sys.argv[0]).name;args=sys.argv[1:]",
                        "root=pathlib.Path(" + repr(str(root)) + ")",
                        "record={'tool':name,'args':args,'target':os.environ.get('CARGO_TARGET_DIR'),'dev':os.environ.get('DEV_DNA_BIN')}",
                        "(root/'calls'/str(os.getpid())).write_text(json.dumps(record))",
                        "if name=='git': print(root);sys.exit(0)",
                        "if name=='sccache': print('{}');sys.exit("
                        + str(metrics_exit)
                        + " if sum(json.loads(p.read_text())['tool']=='sccache' for p in (root/'calls').iterdir())==1 else "
                        + str(final_metrics_exit)
                        + ")",
                        "if name=='cargo': sys.exit(" + str(preparation_exit) + ")",
                        "if name=='cargo-nextest': sys.exit(99)",
                        "if name=='make' and args[-1]=='_dev-dna-bin': sys.exit("
                        + str(bootstrap_exit)
                        + ")",
                        "if name=='make': sys.exit("
                        + str(child_exit)
                        + " if args[-1]=='lint' else 0)",
                        "raise SystemExit('unrecognized fixture tool')",
                    ]
                )
                + "\n"
            )
            executable.chmod(0o755)
            for name in ["git", "cargo", "cargo-nextest", "sccache", "make"]:
                (root / "bin" / name).symlink_to(executable)
            env = {
                name: value
                for name, value in os.environ.items()
                if name not in {"MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES", "DEV_DNA_BIN"}
            }
            env.update(
                PATH=str(root / "bin") + os.pathsep + os.environ["PATH"],
                PYTHONDONTWRITEBYTECODE="1",
                ARTIFACT_ROOT=str(root / "artifacts"),
            )
            result = subprocess.run(
                [
                    os.environ.get("BIJUX_GATE_TEST_BASH", "bash"),
                    str(root / "makes/bin/run_github_workflow_gate.sh"),
                ],
                env=env,
                cwd=root,
                text=True,
                capture_output=True,
                timeout=20,
            )
            captured = [json.loads(path.read_text()) for path in calls.iterdir()]
            files = {
                str(path.relative_to(root)): path.read_text()
                for path in (root / "artifacts").rglob("*")
                if path.is_file()
            }
            return result, captured, files

    def test_retained_gates_never_invoke_test_preparation(self) -> None:
        result, calls, files = self.run_fixture(("make fmt", "make lint", "make audit"))
        self.assertEqual(result.returncode, 0, result.stderr)
        self.assertFalse(
            any(call["tool"] in {"cargo", "cargo-nextest"} for call in calls)
        )
        make = [call for call in calls if call["tool"] == "make"]
        self.assertEqual({call["args"][-1] for call in make}, {"fmt", "lint", "audit"})
        self.assertTrue(all(call["dev"] is None for call in make))
        self.assertEqual(len({call["target"] for call in make}), 3)
        self.assertIn("artifacts/github-all/build-authority/sccache-after.json", files)

    def test_selected_test_gate_prepares_locked_test_binaries(self) -> None:
        result, calls, _ = self.run_fixture(("make test",))
        self.assertEqual(result.returncode, 0, result.stderr)
        cargo = [call for call in calls if call["tool"] == "cargo"]
        self.assertEqual(len(cargo), 1)
        self.assertEqual(cargo[0]["args"][:2], ["nextest", "list"])
        self.assertIn("--locked", cargo[0]["args"])
        self.assertIn("--all-features", cargo[0]["args"])
        self.assertTrue(any(call["args"][-1] == "_dev-dna-bin" for call in calls))
        self.assertTrue(
            next(
                call
                for call in calls
                if call["tool"] == "make" and call["args"][-1] == "test"
            )["dev"]
        )

    def test_preparation_failure_prevents_children(self) -> None:
        result, calls, files = self.run_fixture(("make test",), bootstrap_exit=7)
        self.assertEqual(result.returncode, 7)
        self.assertFalse(any(call["tool"] == "cargo" for call in calls))
        self.assertFalse(
            any(call["tool"] == "make" and call["args"][-1] == "test" for call in calls)
        )
        self.assertIn("artifacts/github-all/build-authority/sccache-after.json", files)

    def test_nextest_preparation_failure_prevents_children(self) -> None:
        result, calls, files = self.run_fixture(("make test",), preparation_exit=8)
        self.assertEqual(result.returncode, 8)
        self.assertTrue(any(call["tool"] == "cargo" for call in calls))
        self.assertFalse(
            any(call["tool"] == "make" and call["args"][-1] == "test" for call in calls)
        )
        self.assertIn("artifacts/github-all/build-authority/sccache-after.json", files)

    def test_final_metrics_failure_is_fatal(self) -> None:
        result, calls, _ = self.run_fixture(("make fmt",), final_metrics_exit=13)
        self.assertEqual(result.returncode, 1)
        self.assertTrue(any(call["tool"] == "make" for call in calls))
        self.assertIn("failed to publish final sccache metrics", result.stderr)

    def test_child_failure_propagates_and_retains_summary(self) -> None:
        result, _, files = self.run_fixture(
            ("make fmt", "make lint", "make audit"), child_exit=11
        )
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("lint\tlint\t11", files["artifacts/github-all/summary.tsv"])
        self.assertIn("artifacts/github-all/build-authority/sccache-after.json", files)

    def test_malformed_workflow_runs_no_children(self) -> None:
        result, calls, _ = self.run_fixture(("make fmt && make test",))
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(any(call["tool"] != "git" for call in calls))

    def test_metrics_failure_is_fatal(self) -> None:
        result, calls, _ = self.run_fixture(("make fmt",), metrics_exit=12)
        self.assertEqual(result.returncode, 12)
        self.assertFalse(any(call["tool"] == "make" for call in calls))


if __name__ == "__main__":
    unittest.main()
