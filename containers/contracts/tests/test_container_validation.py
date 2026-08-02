from __future__ import annotations

import importlib.util
from pathlib import Path
import subprocess
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
        self.assertEqual(resolved["study_root"], (root / "study").resolve())
        self.assertEqual(resolved["active_repository"], (root / "active-repository").resolve())
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

    def test_repository_freeze_produces_a_clean_commit_checkout(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance = self.governance(root)
            repository = root / "active-repository"
            subprocess.run(
                ["git", "init", "--quiet", "--initial-branch", "main", str(repository)],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(repository), "config", "user.name", "Bijux Test"],
                check=True,
            )
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repository),
                    "config",
                    "user.email",
                    "test@bijux.io",
                ],
                check=True,
            )
            contract_directory = repository / "containers" / "contracts"
            contract_directory.mkdir(parents=True)
            (contract_directory / "tracked.txt").write_text(
                "governed\n",
                encoding="utf-8",
            )
            unselected_directory = repository / "other"
            unselected_directory.mkdir()
            (unselected_directory / "unselected.txt").write_text(
                "complete tree\n",
                encoding="utf-8",
            )
            subprocess.run(
                [
                    "git",
                    "-C",
                    str(repository),
                    "add",
                    "containers/contracts/tracked.txt",
                    "other/unselected.txt",
                ],
                check=True,
            )
            subprocess.run(
                ["git", "-C", str(repository), "commit", "--quiet", "-m", "fixture"],
                check=True,
            )
            commit = subprocess.run(
                ["git", "-C", str(repository), "rev-parse", "HEAD"],
                check=True,
                capture_output=True,
                text=True,
            ).stdout.strip()
            governance_path = root / "governance.yml"
            governance_path.write_text(
                container_validation.yaml.safe_dump(governance, sort_keys=False),
                encoding="utf-8",
            )

            with mock.patch.object(
                container_validation.socket,
                "gethostname",
                return_value="worker-7",
            ):
                frozen = container_validation.freeze_repository(
                    commit,
                    governance_path,
                )

            self.assertEqual(frozen.name, commit)
            self.assertEqual(
                (frozen / "containers" / "contracts" / "tracked.txt").read_text(),
                "governed\n",
            )
            self.assertFalse((frozen / "other" / "unselected.txt").exists())
            self.assertEqual(
                container_validation.git_output(
                    frozen,
                    "show",
                    "HEAD:other/unselected.txt",
                ),
                "complete tree",
            )
            self.assertEqual(
                container_validation.git_output(frozen, "status", "--porcelain"),
                "",
            )


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
