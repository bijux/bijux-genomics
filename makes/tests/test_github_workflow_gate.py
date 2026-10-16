"""Exercise owned workflow admission and process supervision without Rust execution."""

from __future__ import annotations

import importlib.util
import json
import os
from pathlib import Path
import shutil
import signal
import subprocess
import sys
import tempfile
import time
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
        interrupt_signal: int | None = None,
        interrupt_phase: str = "gate",
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
                        "if name=='make' and args[-1]==" + repr("fmt" if interrupt_phase == "gate" else "_dev-dna-bin")
                        + " and " + repr(interrupt_signal is not None) + ":",
                        " import signal,subprocess,time",
                        " child_source=" + repr(
                            "import pathlib,signal,sys,time\n"
                            f"root=pathlib.Path({str(root)!r})\n"
                            "def stop(number, frame):\n"
                            " (root/'artifacts/grandchild-signal').write_text(str(number))\n"
                            " raise SystemExit(0)\n"
                            "signal.signal(signal.SIGTERM,stop)\n"
                            "signal.signal(signal.SIGINT,stop)\n"
                            "(root/'artifacts/grandchild-ready').write_text(str(__import__('os').getpid()))\n"
                            "while True: time.sleep(0.05)\n"
                        ),
                        " child=subprocess.Popen([sys.executable,'-c',child_source])",
                        " (root/'artifacts/grandchild-pid').write_text(str(child.pid))",
                        " def stop(number,frame):",
                        "  try: child.wait(timeout=4)",
                        "  except subprocess.TimeoutExpired: raise SystemExit(97)",
                        "  (root/'artifacts/grandchild-reaped').write_text(str(child.returncode))",
                        "  raise SystemExit(128+number)",
                        " signal.signal(signal.SIGTERM,stop)",
                        " signal.signal(signal.SIGINT,stop)",
                        " while True: time.sleep(0.05)",
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
            args = [os.environ.get("BIJUX_GATE_TEST_BASH", "bash"),
                    str(root / "makes/bin/run_github_workflow_gate.sh")]
            if interrupt_signal is None:
                result = subprocess.run(args, env=env, cwd=root, text=True,
                                        capture_output=True, timeout=20)
            else:
                process = subprocess.Popen(args, env=env, cwd=root, text=True,
                                           stdout=subprocess.PIPE, stderr=subprocess.PIPE,
                                           start_new_session=True)
                try:
                    deadline = time.monotonic() + 10
                    ready = root / "artifacts/grandchild-ready"
                    while not ready.exists() and process.poll() is None and time.monotonic() < deadline:
                        time.sleep(0.01)
                    self.assertTrue(ready.exists(), "gate grandchild did not start")
                    process.send_signal(interrupt_signal)
                    stdout, stderr = process.communicate(timeout=15)
                    result = subprocess.CompletedProcess(args, process.returncode, stdout, stderr)
                finally:
                    # The fixture session and recorded child are exclusively owned, even when
                    # exercising a broken supervisor which leaves descendants behind.
                    try:
                        os.killpg(process.pid, signal.SIGKILL)
                    except ProcessLookupError:
                        pass
                    child_pid = root / "artifacts/grandchild-pid"
                    if child_pid.exists() and not (root / "artifacts/grandchild-reaped").exists():
                        try:
                            os.kill(int(child_pid.read_text()), signal.SIGKILL)
                        except ProcessLookupError:
                            pass
                    if process.poll() is None:
                        process.wait(timeout=5)
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

    def test_signals_terminate_gate_grandchildren_and_preserve_terminal_summary(self) -> None:
        for phase in ("gate", "preparation"):
            for interrupted in (signal.SIGTERM, signal.SIGINT):
                with self.subTest(phase=phase, signal=interrupted):
                    result, calls, files = self.run_fixture(
                        ("make fmt" if phase == "gate" else "make test",),
                        interrupt_signal=interrupted, interrupt_phase=phase,
                    )
                    self.assertEqual(result.returncode, 128 + interrupted, result.stderr)
                    self.assertEqual(files.get("artifacts/grandchild-signal"), str(signal.SIGTERM))
                    self.assertEqual(files.get("artifacts/grandchild-reaped"), "0")
                    row = "fmt\tfmt\t143\t" if phase == "gate" else "workspace-preparation\t_dev-dna-bin/nextest-list\t143\t"
                    self.assertIn(row, files["artifacts/github-all/summary.tsv"])
                    if phase == "preparation":
                        self.assertFalse(any(call["tool"] == "cargo" for call in calls))
                        self.assertFalse(any(call["tool"] == "make" and call["args"][-1] == "test" for call in calls))
                    self.assertIn("artifacts/github-all/build-authority/sccache-after.json", files)


if __name__ == "__main__":
    unittest.main()
