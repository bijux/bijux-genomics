from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
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
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                resolved = container_validation.validate_governance(
                    self.governance(root)
                )

        self.assertEqual(resolved["site_id"], "test-cluster")
        self.assertEqual(resolved["study_root"], (root / "study").resolve())
        self.assertEqual(
            resolved["active_repository"], (root / "active-repository").resolve()
        )
        self.assertEqual(resolved["slurm_account"], "science")
        self.assertEqual(resolved["slurm_partitions"], ["compute"])

    def test_execution_outside_the_declared_site_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            with (
                mock.patch.object(
                    container_validation.socket, "gethostname", return_value="login-1"
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError,
                    "outside governed site",
                ),
            ):
                container_validation.validate_governance(self.governance(root))

    def test_sbatch_cluster_suffix_is_removed_from_numeric_identity(self) -> None:
        self.assertEqual(
            container_validation.parse_sbatch_job_id("3457577;lunarc\n"),
            "3457577",
        )

    def test_non_numeric_sbatch_identity_is_rejected(self) -> None:
        with self.assertRaisesRegex(
            container_validation.ContractError, "numeric job id"
        ):
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
                complete = container_validation.freeze_repository(
                    commit,
                    governance_path,
                    checkout_scope="complete",
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
            self.assertEqual(complete.parent.name, "image-builds")
            self.assertEqual(
                (complete / "other" / "unselected.txt").read_text(),
                "complete tree\n",
            )
            self.assertEqual(
                container_validation.git_output(complete, "status", "--porcelain"),
                "",
            )


class CurrentPipelineContractTests(unittest.TestCase):
    def make_contracts(self, root: Path) -> tuple[Path, Path, Path]:
        study = root / "study"
        image_root = study / "containers" / "apptainer"
        data_root = study / "data" / "corpora" / "sample"
        image_root.mkdir(parents=True)
        data_root.mkdir(parents=True)
        image = image_root / "tool-1.0-linux-amd64-identity.sif"
        image.write_bytes(b"sif-image")
        input_path = data_root / "sample.fastq"
        input_path.write_text("@read\nA\n+\nI\n", encoding="utf-8")

        repository = root / "repository"
        site_root = repository / "containers" / "contracts" / "sites"
        site_root.mkdir(parents=True)
        governance_path = site_root / "cluster.yml"
        governance = {
            "schema_version": "bijux.container.governance.v1",
            "site": {"id": "cluster", "hostname_regex": "^worker-7$"},
            "paths": {
                "study_root": str(study),
                "active_repository": str(repository),
                "frozen_repositories": str(root / "frozen"),
            },
            "runtime": {"apptainer_architecture": "linux/amd64"},
            "slurm": {"account": "science", "partitions": ["compute"]},
        }
        governance_path.write_text(
            container_validation.yaml.safe_dump(governance, sort_keys=False),
            encoding="utf-8",
        )
        subprocess.run(
            ["git", "init", "--quiet", "--initial-branch", "main", str(repository)],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repository), "config", "user.name", "Bijux Test"],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repository), "config", "user.email", "test@bijux.io"],
            check=True,
        )
        subprocess.run(
            [
                "git",
                "-C",
                str(repository),
                "add",
                "containers/contracts/sites/cluster.yml",
            ],
            check=True,
        )
        subprocess.run(
            ["git", "-C", str(repository), "commit", "--quiet", "-m", "fixture"],
            check=True,
        )
        commit = container_validation.git_output(repository, "rev-parse", "HEAD")

        contracts = root / "contracts"
        contracts.mkdir()
        pipeline_path = contracts / "config-pipeline.yml"
        credential_path = contracts / "config-credential.yml"
        pipeline = {
            "schema_version": "bijux.container.pipeline.v2",
            "run": {
                "id": "run-2000",
                "campaign": "real-sample-validation",
                "corpus": "public-corpus",
                "sample": "sample",
                "domain": "fastq",
                "stage": "fastq.validate",
                "tool": "tool",
                "operator": "scientist",
                "submitted_at_utc": "2026-08-03T10:00:00Z",
            },
            "repository": {"path": str(repository), "commit": commit, "clean": True},
            "image": {
                "path": str(image),
                "sha256": container_validation.sha256_file(image),
            },
            "data": {
                "inputs": [
                    {
                        "id": "public_corpus_sample",
                        "kind": "data",
                        "path": str(input_path),
                        "mount_path": "/input/public_corpus_sample",
                        "sha256": container_validation.sha256_file(input_path),
                    }
                ]
            },
            "commands": {
                "version": ["tool", "--version"],
                "help": ["tool", "--help"],
                "real": ["tool", "run"],
                "negative": ["tool", "missing.fastq"],
            },
            "outputs": {
                "result_root": str(
                    study / "results" / "container-validation" / "run-2000-tool"
                ),
                "mount_path": "/result",
                "required_files": ["metrics.json"],
                "collection_rule": "required_files_and_metrics",
            },
            "resources": {
                "account": "science",
                "partition": "compute",
                "cpus": 2,
                "memory_mb": 1024,
                "walltime": "00:10:00",
                "scratch_mb": 1024,
                "retry": {
                    "max_attempts": 1,
                    "backoff_seconds": 0,
                    "retryable_exit_codes": [],
                },
            },
            "execution": {
                "locale": "C",
                "timezone": "UTC",
                "runtime_network": False,
                "threads": 2,
                "random_seed": 17,
                "cache_policy": "cold",
                "determinism": "seeded",
            },
            "dependencies": [],
            "acceptance": {
                "version_regex": "1[.]0",
                "negative_exit_nonzero": True,
                "metrics": [
                    {
                        "id": "valid_reads",
                        "path": "metrics.json",
                        "json_pointer": "/valid_reads",
                        "operator": "ge",
                        "expected": 1,
                        "unit": "reads",
                    }
                ],
            },
        }
        pipeline_path.write_text(
            container_validation.yaml.safe_dump(pipeline, sort_keys=False),
            encoding="utf-8",
        )
        credential_path.write_text(
            "schema_version: bijux.container.credential_reference.v1\n"
            "credentials_required: false\n"
            "references: []\n"
            "redaction:\n"
            "  publish_values: false\n",
            encoding="utf-8",
        )
        return governance_path, pipeline_path, credential_path

    def test_current_contract_freezes_science_and_execution_parameters(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = self.make_contracts(Path(directory))
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                contract = container_validation.validate_contracts(*paths)

        self.assertEqual(contract["pipeline_schema"], "bijux.container.pipeline.v2")
        self.assertEqual(contract["execution"]["random_seed"], 17)
        self.assertEqual(contract["metrics"][0]["id"], "valid_reads")
        self.assertEqual(
            contract["input_binds"][0][1], Path("/input/public_corpus_sample")
        )
        self.assertEqual(contract["result_mount_path"], Path("/result"))

    def test_current_contract_rejects_host_paths_as_container_mounts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = self.make_contracts(Path(directory))
            pipeline = container_validation.load_yaml(paths[1])
            pipeline["data"]["inputs"][0]["mount_path"] = pipeline["data"]["inputs"][0][
                "path"
            ]
            paths[1].write_text(
                container_validation.yaml.safe_dump(pipeline, sort_keys=False),
                encoding="utf-8",
            )
            with (
                mock.patch.object(
                    container_validation.socket, "gethostname", return_value="worker-7"
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError,
                    "mount_path must equal /input/public_corpus_sample",
                ),
            ):
                container_validation.validate_contracts(*paths)

    def test_current_contract_refuses_unimplemented_automatic_retries(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = self.make_contracts(Path(directory))
            pipeline = container_validation.load_yaml(paths[1])
            pipeline["resources"]["retry"]["max_attempts"] = 2
            paths[1].write_text(
                container_validation.yaml.safe_dump(pipeline, sort_keys=False),
                encoding="utf-8",
            )
            with (
                mock.patch.object(
                    container_validation.socket, "gethostname", return_value="worker-7"
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError,
                    "until governed resubmission exists",
                ),
            ):
                container_validation.validate_contracts(*paths)

    def test_metric_evaluation_records_observed_value(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            (root / "metrics.json").write_text(
                json.dumps({"valid_reads": 12}), encoding="utf-8"
            )
            records, failures = container_validation.evaluate_metrics(
                root,
                [
                    {
                        "id": "valid_reads",
                        "path": "metrics.json",
                        "json_pointer": "/valid_reads",
                        "operator": "ge",
                        "expected": 10,
                        "unit": "reads",
                    }
                ],
            )

        self.assertEqual(failures, [])
        self.assertEqual(records[0]["observed"], 12)
        self.assertEqual(records[0]["status"], "passed")

    def test_current_slurm_script_enforces_scratch_and_no_automatic_retry(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root)
            script = root / "submit.sh"
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                container_validation.render_slurm(*paths, script)
            rendered = script.read_text(encoding="utf-8")

        self.assertIn("#SBATCH --no-requeue", rendered)
        self.assertIn("SLURM_TMPDIR:?Slurm scratch is unavailable", rendered)
        self.assertIn("available_scratch_kib < 1048576", rendered)
        self.assertIn("umask 027", rendered)


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

    def test_definition_file_sources_are_repository_relative(self) -> None:
        definition = b"""Bootstrap: docker
%files
    containers/tools/tool/wrapper /usr/local/bin/tool
    containers/tools/tool/data /opt/tool/data

%post
    true
"""
        self.assertEqual(
            container_build.definition_file_sources(definition),
            [
                Path("containers/tools/tool/wrapper"),
                Path("containers/tools/tool/data"),
            ],
        )

    def test_definition_file_sources_reject_parent_traversal(self) -> None:
        with self.assertRaisesRegex(
            container_build.ContractError, "repository-relative"
        ):
            container_build.definition_file_sources(
                b"%files\n    ../secret /opt/secret\n"
            )


if __name__ == "__main__":
    unittest.main()
