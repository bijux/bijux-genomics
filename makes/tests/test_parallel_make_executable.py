"""Keep parallel governance consumers independent of Cargo output replacement."""

from pathlib import Path
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest


ROOT = Path(__file__).resolve().parents[2]
CARGO_MAKE = ROOT / "makes/cargo.mk"
PREFIX = "artifacts/target/debug/bijux-dna-dev"


class ParallelMakeExecutableTests(unittest.TestCase):
    def setUp(self):
        products = ROOT / "artifacts/tests/parallel-make-executable"
        products.mkdir(parents=True, exist_ok=True)
        owner = tempfile.TemporaryDirectory(dir=products)
        self.addCleanup(owner.cleanup)
        self.root = Path(owner.name)
        self.env = dict(os.environ)
        for name in ("MAKEFLAGS", "MFLAGS", "MAKEOVERRIDES", "SHELL"):
            self.env.pop(name, None)
        self.env.update({
            "CALLS": str(self.root / "calls.jsonl"),
            "READY_FIFO": str(self.root / "ready.fifo"),
            "PYTHONDONTWRITEBYTECODE": "1",
        })
        os.mkfifo(self.env["READY_FIFO"])
        # Keep the pipe alive across the writer exiting and the reader starting.
        pipe = os.open(self.env["READY_FIFO"], os.O_RDWR | os.O_NONBLOCK)
        self.addCleanup(os.close, pipe)
        self.mock_bin = self.root / "mock-bin"
        self.mock_bin.mkdir()
        self.env["PATH"] = str(self.mock_bin) + os.pathsep + self.env["PATH"]
        self.script(self.mock_bin / "sh", """
import os
import select
import sys
if len(sys.argv) == 7 and sys.argv[-1].endswith("checks run consume"):
    fd = os.open(os.environ["READY_FIFO"], os.O_RDONLY | os.O_NONBLOCK)
    try:
        if not select.select([fd], [], [], 10)[0] or os.read(fd, 1) != b"1":
            raise SystemExit("relink handshake missing")
    finally:
        os.close(fd)
os.execv("/bin/sh", ["/bin/sh", *sys.argv[1:]])
""")
        self.bootstrap = self.root / "bootstrap"
        self.script(self.bootstrap, """
import os
import sys
if os.environ.get("BOOTSTRAP_FAIL"):
    raise SystemExit(19)
assert os.access(sys.argv[1], os.X_OK)
""")
        (self.root / "Makefile").write_text(
            f'ensure_artifact_env = true\ninclude {CARGO_MAKE}\n'
        )
        self.commands = self.root / "commands.txt"
        self.logs = self.root / "artifacts/gates/lint-parallel"

    @staticmethod
    def script(path, body):
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text(f"#!{sys.executable}\n" + body)
        path.chmod(0o755)

    def invoke(self, commands, *, custom=False, failure=False):
        program = self.root / ("custom/debug/bijux-dna-dev" if custom else PREFIX)
        self.env["CANONICAL_BIN"] = str(program)
        if failure:
            self.env["BOOTSTRAP_FAIL"] = "1"
        self.script(program, r"""
import json
import os
from pathlib import Path
import stat
import sys
args = sys.argv[1:]
with open(os.environ["CALLS"], "a") as output:
    output.write(json.dumps({"args": args, "program": sys.argv[0],
                            "mode": stat.S_IMODE(Path(sys.argv[0]).stat().st_mode)}) + "\n")
if args == ["docs", "run", "relink"]:
    Path(os.environ["CANONICAL_BIN"]).unlink()
    fd = os.open(os.environ["READY_FIFO"], os.O_WRONLY | os.O_NONBLOCK)
    try:
        os.write(fd, b"1")
    finally:
        os.close(fd)
if args == ["checks", "run", "fail"]:
    print("declared gate failure", file=sys.stderr)
    raise SystemExit(17)
print("gate observed: " + " ".join(args))
""")
        self.commands.write_text("".join(f"{PREFIX} {command}\n" for command in commands))
        return subprocess.run(
            ["make", "--no-print-directory", "_lint-automation",
             f"DEV_DNA_BIN={program}", f"DEV_DNA_BOOTSTRAP={self.bootstrap}",
             f"ARTIFACTS_DIR={self.logs.parent}",
             f"LINT_PARALLEL_COMMANDS_FILE={self.commands}", "LINT_PARALLEL_JOBS=2"],
            cwd=self.root, env=self.env, capture_output=True, text=True, timeout=25,
        )

    def test_parallel_consumers_survive_canonical_executable_unlink(self):
        result = self.invoke(["docs run relink", "checks run consume"])
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("jobs=2", result.stdout)
        self.assert_batch_copy()

    def test_custom_executable_is_copied_before_parallel_consumers(self):
        result = self.invoke(["docs run relink", "checks run consume"], custom=True)
        self.assertEqual(result.returncode, 0, result.stdout + result.stderr)
        self.assertIn("jobs=2", result.stdout)
        self.assert_batch_copy()

    def assert_batch_copy(self):
        self.assertFalse(Path(self.env["CANONICAL_BIN"]).exists())
        copy = self.logs / "bijux-dna-dev"
        self.assertTrue(os.access(copy, os.X_OK))
        self.assertEqual(stat.S_IMODE(copy.stat().st_mode) & 0o222, 0)
        rows = [json.loads(line) for line in Path(self.env["CALLS"]).read_text().splitlines()]
        self.assertEqual([row["args"] for row in rows[:2]], [
            ["tooling", "run", "repo-doctor", "--fast"], ["domain", "run", "lock-registry"],
        ])
        self.assertEqual({tuple(row["args"]) for row in rows[2:]}, {
            ("docs", "run", "relink"), ("checks", "run", "consume"),
        })
        self.assertTrue(all(row["program"] == str(copy) for row in rows))
        self.assertTrue(all(row["mode"] & 0o222 == 0 for row in rows))
        self.assertEqual((self.logs / "commands.txt").read_text(), self.commands.read_text())
        logs = sorted(self.logs.glob("*.log"))
        self.assertEqual(len(logs), 2)
        self.assertTrue(any("checks run consume" in path.read_text() for path in logs))

    def test_gate_failure_propagates_with_log_and_peer_output(self):
        result = self.invoke(["checks run pass", "checks run fail"])
        self.assertNotEqual(result.returncode, 0)
        self.assertIn("FAILED", result.stderr)
        self.assertIn("declared gate failure", result.stderr)
        text = "\n".join(path.read_text() for path in self.logs.glob("*.log"))
        self.assertIn("declared gate failure", text)
        self.assertIn("gate observed: checks run pass", text)

    def test_bootstrap_failure_prevents_batch_execution(self):
        result = self.invoke(["checks run pass"], failure=True)
        self.assertNotEqual(result.returncode, 0)
        self.assertFalse(Path(self.env["CALLS"]).exists())
        self.assertFalse(self.logs.exists())


if __name__ == "__main__":
    unittest.main()
