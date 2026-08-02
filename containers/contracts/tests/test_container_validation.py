from __future__ import annotations

import importlib.util
from pathlib import Path
import sys
import tempfile
import unittest
from unittest import mock


EXECUTOR = Path(__file__).parents[1] / "container_validation.py"
SPEC = importlib.util.spec_from_file_location("container_validation", EXECUTOR)
assert SPEC is not None and SPEC.loader is not None
container_validation = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = container_validation
SPEC.loader.exec_module(container_validation)

BUILD_EXECUTOR = Path(__file__).parents[1] / "container_build.py"
BUILD_SPEC = importlib.util.spec_from_file_location("container_build", BUILD_EXECUTOR)
assert BUILD_SPEC is not None and BUILD_SPEC.loader is not None
container_build = importlib.util.module_from_spec(BUILD_SPEC)
BUILD_SPEC.loader.exec_module(container_build)


class GovernanceContractTests(unittest.TestCase):
    def governance(self, root: Path) -> dict[str, object]:
        study = root / "study"
        active = root / "active-repository"
        study.mkdir()
        active.mkdir()
        return {
            "schema_version": "bijux.container.governance.v1",
            "site": {"id": "test-cluster", "hostname_regex": "^worker-[0-9]+$"},
            "paths": {
                "study_root": str(study),
                "active_repository": str(active),
                "frozen_repositories": str(root / "frozen-repositories"),
            },
            "runtime": {"apptainer_architecture": "linux/amd64"},
            "slurm": {"account": "science", "partitions": ["compute"]},
        }

    def test_site_paths_are_supplied_by_governance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with mock.patch.object(container_validation.socket, "gethostname", return_value="worker-7"):
                resolved = container_validation.validate_governance(self.governance(root))

        self.assertEqual(resolved["site_id"], "test-cluster")
        self.assertEqual(resolved["study_root"], root / "study")
        self.assertEqual(resolved["active_repository"], root / "active-repository")
        self.assertEqual(resolved["slurm_account"], "science")
        self.assertEqual(resolved["slurm_partitions"], ["compute"])

    def test_execution_outside_the_declared_site_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with mock.patch.object(container_validation.socket, "gethostname", return_value="login-1"):
                with self.assertRaisesRegex(
                    container_validation.ContractError,
                    "outside governed site",
                ):
                    container_validation.validate_governance(self.governance(root))

    def test_sbatch_cluster_suffix_is_removed_from_numeric_identity(self) -> None:
        self.assertEqual(
            container_validation.parse_sbatch_job_id("3457577;lunarc\n"),
            "3457577",
        )

    def test_non_numeric_sbatch_identity_is_rejected(self) -> None:
        with self.assertRaisesRegex(container_validation.ContractError, "numeric job id"):
            container_validation.parse_sbatch_job_id("submission failed")


class BuildAdmissionTests(unittest.TestCase):
    def test_immutable_name_carries_runtime_and_content_identity(self) -> None:
        name = container_build.immutable_image_name(
            "adapterremoval",
            "2.3.4",
            "linux/amd64",
            "a" * 64,
        )
        self.assertEqual(
            name,
            "adapterremoval-2.3.4-linux-amd64-aaaaaaaaaaaa.sif",
        )


if __name__ == "__main__":
    unittest.main()
