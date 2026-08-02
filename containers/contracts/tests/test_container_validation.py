from __future__ import annotations

import importlib.util
from pathlib import Path
import tempfile
import unittest
from unittest import mock


EXECUTOR = Path(__file__).parents[1] / "container_validation.py"
SPEC = importlib.util.spec_from_file_location("container_validation", EXECUTOR)
assert SPEC is not None and SPEC.loader is not None
container_validation = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(container_validation)


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


if __name__ == "__main__":
    unittest.main()
