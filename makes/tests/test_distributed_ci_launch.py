"""Exercise CI consumer custody without invoking Cargo or scientific programs."""

import hashlib
import importlib.util
import json
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch


ROOT = Path(__file__).resolve().parents[2]


def load(name):
    spec = importlib.util.spec_from_file_location(name, ROOT / "makes/bin" / f"{name}.py")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


LAUNCH = load("run_distributed_ci")
RUNTIME = load("provision_ci_runtime")
PYTHON = load("run_ci_python")
WORKFLOWS = load("github_workflow_gates")


class DistributedCiAdmissionTests(unittest.TestCase):
    def setUp(self):
        owner = ROOT / "artifacts/tests/distributed-ci"
        owner.mkdir(parents=True, exist_ok=True)
        scratch = tempfile.TemporaryDirectory(dir=owner)
        self.addCleanup(scratch.cleanup)
        self.root = Path(scratch.name)
        self.build = self.root / "artifacts/ci/distributed/build"
        self.build.mkdir(parents=True)
        self.runner = self.build / "bijux-dna-dev"
        self.runner.write_bytes(b"CI executable fixture")
        self.metadata = {"source_commit": "source-identity",
                         "runner_sha256": hashlib.sha256(self.runner.read_bytes()).hexdigest()}
        self.publish()
        mock = patch.object(LAUNCH.subprocess, "check_output", return_value="source-identity\n")
        mock.start()
        self.addCleanup(mock.stop)

    def publish(self):
        (self.build / "producer.json").write_text(json.dumps(self.metadata))

    def test_valid_download_restores_executable_mode_without_compilation(self):
        self.runner.chmod(0o644)
        self.assertEqual(LAUNCH.admitted_runner(self.root), self.runner)
        self.assertEqual(self.runner.stat().st_mode & 0o777, 0o555)

    def test_foreign_source_and_corrupt_runner_refuse_before_executable_admission(self):
        self.metadata["source_commit"] = "foreign-source"
        self.publish()
        with self.assertRaisesRegex(ValueError, "different source"):
            LAUNCH.admitted_runner(self.root)
        self.metadata["source_commit"] = "source-identity"
        self.publish()
        self.runner.write_bytes(b"corrupt")
        with self.assertRaisesRegex(ValueError, "checksum"):
            LAUNCH.admitted_runner(self.root)

    def test_missing_runner_and_redirected_artifact_owner_refuse(self):
        self.runner.unlink()
        with self.assertRaisesRegex(ValueError, "physical artifact"):
            LAUNCH.admitted_runner(self.root)
        self.build.rename(self.build.with_name("other-build"))
        self.build.symlink_to(self.build.with_name("other-build"), target_is_directory=True)
        with self.assertRaisesRegex(ValueError, "redirected artifact"):
            LAUNCH.admitted_runner(self.root)

    def test_runtime_checksum_path_escape_and_wrong_version_fail_closed(self):
        archive = self.root / "download.tar.bz2"
        archive.write_bytes(b"corrupt upstream download")
        with self.assertRaisesRegex(ValueError, "checksum"):
            RUNTIME.checked_archive(archive)
        with self.assertRaisesRegex(ValueError, "artifacts"):
            RUNTIME.provision(self.root, self.root / "outside", 2)
        with patch.object(RUNTIME.subprocess, "run", return_value=subprocess.CompletedProcess(
                [], 0, stdout="bcftools 1.21\n", stderr="")):
            with self.assertRaisesRegex(ValueError, "bcftools 1.20"):
                RUNTIME.require_version(archive)

    def test_python_catalog_discovery_preserves_explicit_workflow_exclusion(self):
        lane = {"harness": "unittest", "discovery_roots": [],
                "explicit_modules": ["makes.tests.test_github_workflow_gate", "makes.tests.test_distributed_ci_launch"],
                "excluded_modules": ["makes.tests.test_github_workflow_gate"]}
        self.assertEqual(PYTHON.selections(lane), ["makes.tests.test_distributed_ci_launch"])

    def test_hosted_graph_maps_to_complete_local_coverage_without_parallel_setup_races(self):
        self.assertEqual(WORKFLOWS.local_gate_equivalents([
            "fmt", "lint", "audit", "ci-test-runtime", "ci-test-build",
            "ci-test-shard", "ci-test-python", "ci-test-aggregate",
        ]), ["fmt", "lint", "audit", "test-all", "ci-test-python"])
        for missing in WORKFLOWS.DISTRIBUTED_NATIVE_TARGETS | {"ci-test-python"}:
            targets = sorted((WORKFLOWS.DISTRIBUTED_NATIVE_TARGETS | {"ci-test-python"}) - {missing})
            with self.subTest(missing=missing), self.assertRaisesRegex(ValueError, "omits"):
                WORKFLOWS.local_gate_equivalents(targets)


if __name__ == "__main__":
    unittest.main()
