from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from typing import Any
from unittest import mock

EXECUTOR = Path(__file__).parents[1] / "container_validation.py"
sys.path.insert(0, str(EXECUTOR.parent))
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
            "slurm": {
                "account": "science",
                "partitions": ["compute"],
                "scratch_root": str(root),
            },
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
            resolved["containers_root"], (root / "study" / "containers").resolve()
        )
        self.assertEqual(resolved["data_root"], (root / "study" / "data").resolve())
        self.assertEqual(
            resolved["results_root"], (root / "study" / "results").resolve()
        )
        self.assertEqual(
            resolved["active_repository"], (root / "active-repository").resolve()
        )
        self.assertEqual(resolved["slurm_account"], "science")
        self.assertEqual(resolved["slurm_partitions"], ["compute"])

    def test_compute_local_scratch_can_be_declared_by_environment(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance = self.governance(root)
            slurm = governance["slurm"]
            assert isinstance(slurm, dict)
            slurm.pop("scratch_root")
            slurm["scratch_environment"] = "SNIC_TMP"
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                resolved = container_validation.validate_governance(governance)

        self.assertEqual(resolved["slurm_scratch_environment"], "SNIC_TMP")
        self.assertIsNone(resolved["slurm_scratch_root"])

    def test_scratch_authority_must_be_unambiguous(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance = self.governance(root)
            slurm = governance["slurm"]
            assert isinstance(slurm, dict)
            slurm["scratch_environment"] = "SNIC_TMP"
            with (
                mock.patch.object(
                    container_validation.socket, "gethostname", return_value="worker-7"
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError, "exactly one"
                ),
            ):
                container_validation.validate_governance(governance)

    def test_compute_environment_resolves_to_existing_absolute_scratch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            scratch = Path(directory).resolve()
            governance = {
                "slurm_scratch_environment": "SNIC_TMP",
                "slurm_scratch_root": None,
            }
            with mock.patch.dict(
                container_validation.os.environ,
                {"SNIC_TMP": str(scratch)},
                clear=False,
            ):
                resolved = container_validation.governed_scratch_root(governance)

        self.assertEqual(resolved, scratch)

    def test_relative_compute_scratch_is_rejected(self) -> None:
        governance = {
            "slurm_scratch_environment": "SNIC_TMP",
            "slurm_scratch_root": None,
        }
        with (
            mock.patch.dict(
                container_validation.os.environ,
                {"SNIC_TMP": "relative-scratch"},
                clear=False,
            ),
            self.assertRaisesRegex(
                container_validation.ContractError, "existing absolute"
            ),
        ):
            container_validation.governed_scratch_root(governance)

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

    def test_explicit_product_roots_remain_within_the_study(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance = self.governance(root)
            paths = governance["paths"]
            assert isinstance(paths, dict)
            paths.update(
                {
                    "containers_root": str(root / "study" / "container-products"),
                    "data_root": str(root / "study" / "scientific-inputs"),
                    "results_root": str(root / "study" / "scientific-results"),
                }
            )
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                resolved = container_validation.validate_governance(governance)

        self.assertEqual(
            resolved["containers_root"],
            (root / "study" / "container-products").resolve(),
        )

    def test_product_root_outside_the_study_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance = self.governance(root)
            paths = governance["paths"]
            assert isinstance(paths, dict)
            paths["results_root"] = str(root / "outside-results")
            with (
                mock.patch.object(
                    container_validation.socket, "gethostname", return_value="worker-7"
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError,
                    "paths.results_root must remain under",
                ),
            ):
                container_validation.validate_governance(governance)

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
    def make_contracts(
        self,
        root: Path,
        *,
        scratch_environment: str | None = None,
    ) -> tuple[Path, Path, Path]:
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
            "slurm": {
                "account": "science",
                "partitions": ["compute"],
                **(
                    {"scratch_environment": scratch_environment}
                    if scratch_environment
                    else {"scratch_root": str(root)}
                ),
            },
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
                        "mount_path": "/input/public-corpus-sample.fastq.gz",
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
            contract["input_binds"][0][1], Path("/input/public-corpus-sample.fastq.gz")
        )
        self.assertEqual(contract["result_mount_path"], Path("/result"))
        self.assertEqual(
            container_validation.tool_output_root(contract),
            contract["result_root"] / "outputs",
        )

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
                    "mount_path must be one safe filename under /input",
                ),
            ):
                container_validation.validate_contracts(*paths)

    def test_benchmark_contract_uses_a_stage_and_tool_result_boundary(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root)
            pipeline = container_validation.load_yaml(paths[1])
            pipeline["run"]["kind"] = "benchmark"
            pipeline["run"]["domain"] = "bam"
            pipeline["run"]["stage"] = "bam.genotyping"
            pipeline["outputs"]["result_root"] = str(
                root / "study/results/benchmarks/run-2000-bam-genotyping/tool"
            )
            paths[1].write_text(
                container_validation.yaml.safe_dump(pipeline, sort_keys=False),
                encoding="utf-8",
            )
            with mock.patch.object(
                container_validation.socket,
                "gethostname",
                return_value="worker-7",
            ):
                contract = container_validation.validate_contracts(*paths)

        self.assertEqual(contract["evidence_kind"], "benchmark")
        self.assertEqual(
            contract["result_root"],
            (
                root / "study/results/benchmarks/run-2000-bam-genotyping/tool"
            ).resolve(),
        )

    def test_benchmark_contract_rejects_qualification_result_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = self.make_contracts(Path(directory))
            pipeline = container_validation.load_yaml(paths[1])
            pipeline["run"]["kind"] = "benchmark"
            paths[1].write_text(
                container_validation.yaml.safe_dump(pipeline, sort_keys=False),
                encoding="utf-8",
            )
            with (
                mock.patch.object(
                    container_validation.socket,
                    "gethostname",
                    return_value="worker-7",
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError,
                    "benchmark outputs.result_root must equal",
                ),
            ):
                container_validation.validate_contracts(*paths)

    def test_current_contract_rejects_duplicate_container_mounts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            paths = self.make_contracts(Path(directory))
            pipeline = container_validation.load_yaml(paths[1])
            duplicate = dict(pipeline["data"]["inputs"][0])
            duplicate["id"] = "second_input"
            pipeline["data"]["inputs"].append(duplicate)
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
                    "data input mount_path is duplicated",
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

    def test_current_manifest_accepts_governed_execution_specifications(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            active = root / "repository"
            study = root / "study"

            container_validation.require_current_manifest_path(
                study
                / "containers/evidence/specifications/run-2000-tool",
                active,
                study / "containers",
            )

    def test_current_manifest_accepts_governed_run_specs(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            active = root / "repository"
            study = root / "study"

            container_validation.require_current_manifest_path(
                study / "containers/runs/run-2000-tool",
                active,
                study / "containers",
            )

    def test_current_manifest_rejects_operator_scratch_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory).resolve()
            with self.assertRaisesRegex(
                container_validation.ContractError,
                "current manifest must remain under one of",
            ):
                container_validation.require_current_manifest_path(
                    root / "scratch/run-2000-tool",
                    root / "repository",
                    root / "study",
                )

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
            subprocess.run(
                ["bash", "-n", str(script)],
                check=True,
                capture_output=True,
                text=True,
            )

        self.assertIn("#SBATCH --no-requeue", rendered)
        self.assertIn("mktemp -d", rendered)
        self.assertIn("bijux-genomics-${SLURM_JOB_ID}-XXXXXX", rendered)
        self.assertIn("trap cleanup_scratch EXIT", rendered)
        self.assertIn("refusing to remove unexpected scratch path", rendered)
        self.assertIn("available_scratch_kib < 1048576", rendered)
        self.assertIn("umask 027", rendered)

    def test_slurm_script_uses_the_site_compute_local_scratch(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root, scratch_environment="SNIC_TMP")
            script = root / "submit.sh"
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                container_validation.render_slurm(*paths, script)
            rendered = script.read_text(encoding="utf-8")
            syntax = subprocess.run(
                ["bash", "-n", str(script)],
                check=False,
                capture_output=True,
                text=True,
            )

        self.assertEqual(syntax.returncode, 0, syntax.stderr)
        self.assertIn('readonly GOVERNED_SCRATCH_ROOT="${SNIC_TMP}"', rendered)
        self.assertIn("SNIC_TMP is unavailable on the allocated node", rendered)

    def test_slurm_render_reuses_the_submission_contract(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root)
            script = root / "submit.sh"
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                contract = container_validation.validate_contracts(*paths)
            with mock.patch.object(
                container_validation,
                "validate_contracts",
                side_effect=AssertionError("contract was revalidated"),
            ):
                container_validation.render_slurm(
                    *paths,
                    script,
                    validated_contract=contract,
                )
                self.assertTrue(script.is_file())

    def test_completed_dependency_uses_evidence_without_stale_scheduler_wait(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root)
            upstream = (
                root
                / "study/results/container-validation/run-1999-source"
            )
            upstream.mkdir(parents=True)
            output = upstream / "source.fastq.gz"
            output.write_bytes(b"governed upstream output")
            digest = container_validation.sha256_file(output)
            (upstream / "evidence.json").write_text(
                json.dumps(
                    {
                        "run_id": "run-1999",
                        "status": "passed",
                        "runtime": {"slurm_job_id": "3457000"},
                        "outputs": [{"path": output.name, "sha256": digest}],
                    }
                ),
                encoding="utf-8",
            )
            pipeline = container_validation.load_yaml(paths[1])
            pipeline["data"]["inputs"] = [
                {
                    "id": "source_fastq",
                    "kind": "upstream_result",
                    "path": str(output),
                    "mount_path": "/input/source.fastq.gz",
                    "sha256": digest,
                }
            ]
            pipeline["dependencies"] = [
                {
                    "run_id": "run-1999",
                    "slurm_job_id": "3457000",
                    "required_outputs": [digest],
                }
            ]
            paths[1].write_text(
                container_validation.yaml.safe_dump(pipeline, sort_keys=False),
                encoding="utf-8",
            )
            script = root / "submit.sh"
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                contract = container_validation.validate_contracts(*paths)
                container_validation.render_slurm(*paths, script)
                rendered = script.read_text(encoding="utf-8")

        self.assertEqual(contract["dependencies"][0]["run_id"], "run-1999")
        self.assertRegex(
            contract["dependencies"][0]["evidence_sha256"],
            "^[0-9a-f]{64}$",
        )
        self.assertNotIn("#SBATCH --dependency", rendered)

    def test_accounting_preserves_a_failure_before_executor_startup(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            paths = self.make_contracts(root)
            manifest = paths[1].parent
            with mock.patch.object(
                container_validation.socket, "gethostname", return_value="worker-7"
            ):
                contract = container_validation.validate_contracts(*paths)
            submission = container_validation.operational_submission_root(
                manifest,
                contract,
            )
            submission.mkdir(parents=True)
            (submission / "submit.sh").write_text("#!/bin/bash\nexit 1\n")
            (submission / "slurm-job-id.txt").write_text("3458665\n")
            (submission / "slurm-3458665.stderr.log").write_text(
                "scratch preflight failed\n"
            )
            fields = [
                "3458665",
                "fastq-validate-tool-sample",
                "FAILED",
                "1:0",
                "00:00:00",
                "0",
                "2",
                "1024M",
                "",
                "",
                "00:00:00",
                "0",
                "cn001",
            ]
            accounting = subprocess.CompletedProcess(
                ["sacct"],
                0,
                stdout="|".join(fields) + "\n",
                stderr="",
            )
            real_run = subprocess.run

            def run_command(
                argv: list[str],
                **kwargs: object,
            ) -> subprocess.CompletedProcess[str]:
                if argv[0] == "sacct":
                    return accounting
                return real_run(argv, **kwargs)

            with (
                mock.patch.object(
                    container_validation.socket,
                    "gethostname",
                    return_value="worker-7",
                ),
                mock.patch.object(
                    container_validation.subprocess,
                    "run",
                    side_effect=run_command,
                ),
            ):
                accounting_path = container_validation.collect_accounting(
                    manifest,
                    paths[0],
                )

            result = root / "study/results/container-validation/run-2000-tool"
            evidence = json.loads((result / "evidence.json").read_text())
            governance_archived = (result / "config-governance.yml").is_file()

        self.assertEqual(accounting_path.name, "slurm-accounting.json")
        self.assertEqual(evidence["status"], "failed")
        self.assertEqual(evidence["failure_class"], "slurm_pre_execution")
        self.assertEqual(evidence["checks"], {})
        self.assertEqual(evidence["outputs"], [])
        self.assertEqual(evidence["metrics"], [])
        self.assertEqual(evidence["runtime"]["scheduler_state"], "FAILED")
        self.assertTrue(governance_archived)
        self.assertEqual(
            evidence["scheduler_logs"][0]["path"],
            "slurm-3458665.stderr.log",
        )

    def test_failed_run_archive_frees_only_canonical_execution_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study = root / "study"
            containers_root = study / "containers"
            submission = (
                containers_root / "evidence/submissions/run-2000-tool"
            )
            result = study / "results/container-validation/run-2000-tool"
            manifest = containers_root / "evidence/specifications/run-2000-tool"
            submission.mkdir(parents=True)
            result.mkdir(parents=True)
            manifest.mkdir(parents=True)
            (submission / "slurm-job-id.txt").write_text("3458665\n")
            (submission / "submit.sh").write_text("#!/bin/bash\nexit 72\n")
            container_validation.write_json(
                result / "evidence.json",
                {
                    "schema_version": container_validation.EVIDENCE_SCHEMA_V2,
                    "run_id": "run-2000",
                    "status": "failed",
                },
            )
            container_validation.write_json(
                result / "slurm-accounting.json",
                {
                    "schema_version": "bijux.slurm.accounting.v1",
                    "run_id": "run-2000",
                    "slurm_job_id": "3458665",
                },
            )
            contract = {
                "pipeline_schema": container_validation.PIPELINE_SCHEMA_V2,
                "run_id": "run-2000",
                "pipeline": {"run": {"tool": "tool"}},
                "result_root": result,
                "governance": {
                    "containers_root": containers_root,
                    "results_root": root / "study/results",
                },
            }
            with mock.patch.object(
                container_validation,
                "validate_contracts",
                return_value=contract,
            ):
                archived = container_validation.archive_failed_run(manifest)

            archive = json.loads((archived / "archive.json").read_text())
            submission_removed = not submission.exists()
            result_removed = not result.exists()
            archived_submission = (archived / "submission/submit.sh").is_file()
            archived_result = (archived / "result/evidence.json").is_file()

        self.assertTrue(submission_removed)
        self.assertTrue(result_removed)
        self.assertEqual(archive["run_id"], "run-2000")
        self.assertEqual(archive["slurm_job_id"], "3458665")
        self.assertEqual(
            {record["path"] for record in archive["files"]},
            {
                "result/evidence.json",
                "result/slurm-accounting.json",
                "submission/slurm-job-id.txt",
                "submission/submit.sh",
            },
        )
        self.assertTrue(archived_submission)
        self.assertTrue(archived_result)

    def test_nonfailed_run_cannot_be_archived(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            containers_root = root / "study/containers"
            submission = (
                containers_root / "evidence/submissions/run-2000-tool"
            )
            result = root / "study/results/container-validation/run-2000-tool"
            manifest = containers_root / "evidence/specifications/run-2000-tool"
            submission.mkdir(parents=True)
            result.mkdir(parents=True)
            manifest.mkdir(parents=True)
            (submission / "slurm-job-id.txt").write_text("3458665\n")
            container_validation.write_json(
                result / "evidence.json",
                {
                    "schema_version": container_validation.EVIDENCE_SCHEMA_V2,
                    "run_id": "run-2000",
                    "status": "passed",
                },
            )
            container_validation.write_json(
                result / "slurm-accounting.json",
                {
                    "schema_version": "bijux.slurm.accounting.v1",
                    "run_id": "run-2000",
                    "slurm_job_id": "3458665",
                },
            )
            contract = {
                "pipeline_schema": container_validation.PIPELINE_SCHEMA_V2,
                "run_id": "run-2000",
                "pipeline": {"run": {"tool": "tool"}},
                "result_root": result,
                "governance": {"containers_root": containers_root},
            }
            with (
                mock.patch.object(
                    container_validation,
                    "validate_contracts",
                    return_value=contract,
                ),
                self.assertRaisesRegex(
                    container_validation.ContractError, "only terminal failed"
                ),
            ):
                container_validation.archive_failed_run(manifest)
            submission_preserved = submission.is_dir()
            result_preserved = result.is_dir()

        self.assertTrue(submission_preserved)
        self.assertTrue(result_preserved)

    def test_superseded_run_archive_preserves_reason_and_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study = root / "study"
            containers_root = study / "containers"
            submission = containers_root / "evidence/submissions/run-2001-tool"
            result = study / "results/benchmarks/run-2001-stage/tool"
            manifest = containers_root / "evidence/specifications/run-2001-tool"
            submission.mkdir(parents=True)
            result.mkdir(parents=True)
            manifest.mkdir(parents=True)
            (submission / "slurm-job-id.txt").write_text("3459727\n")
            (submission / "submit.sh").write_text("#!/bin/bash\nexit 0\n")
            container_validation.write_json(
                result / "evidence.json",
                {
                    "schema_version": container_validation.EVIDENCE_SCHEMA_V2,
                    "run_id": "run-2001",
                    "status": "passed",
                },
            )
            container_validation.write_json(
                result / "slurm-accounting.json",
                {
                    "schema_version": "bijux.slurm.accounting.v1",
                    "run_id": "run-2001",
                    "slurm_job_id": "3459727",
                },
            )
            contract = {
                "pipeline_schema": container_validation.PIPELINE_SCHEMA_V2,
                "run_id": "run-2001",
                "pipeline": {"run": {"tool": "tool"}},
                "result_root": result,
                "governance": {
                    "containers_root": containers_root,
                    "results_root": study / "results",
                },
            }
            reason = (
                "resource measurements used literal separators instead of TSV rows"
            )
            with mock.patch.object(
                container_validation,
                "validate_contracts",
                return_value=contract,
            ):
                archived = container_validation.archive_superseded_run(
                    manifest,
                    reason,
                )

            archive = json.loads((archived / "archive.json").read_text())

            self.assertFalse(submission.exists())
            self.assertFalse(result.exists())
            self.assertTrue((archived / "submission/submit.sh").is_file())
            self.assertTrue((archived / "result/evidence.json").is_file())

        self.assertEqual(
            archive["schema_version"],
            "bijux.container.superseded_run_archive.v1",
        )
        self.assertEqual(archive["disposition"], "superseded")
        self.assertEqual(archive["reason"], reason)

    def test_apptainer_execution_uses_a_resolvable_isolated_hostname(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            stdout = root / "stdout.log"
            stderr = root / "stderr.log"
            with mock.patch.object(
                container_validation.subprocess,
                "run",
                return_value=subprocess.CompletedProcess([], 0),
            ) as execute:
                status = container_validation.run_apptainer(
                    root / "tool.sif",
                    ["tool", "--version"],
                    [],
                    root,
                    stdout,
                    stderr,
                )

        self.assertEqual(status, 0)
        command = execute.call_args.args[0]
        hostname_index = command.index("--hostname")
        self.assertEqual(
            command[hostname_index + 1],
            container_validation.ISOLATED_RUNTIME_HOSTNAME,
        )


class BuildAdmissionTests(unittest.TestCase):
    def inspection(
        self,
        tool: str = "adapterremoval",
        version: str = "2.3.4",
    ) -> dict[str, Any]:
        return {
            "data": {
                "attributes": {
                    "labels": {
                        "org.opencontainers.image.tool": tool,
                        "io.bijux.tool.version": version,
                        "org.opencontainers.image.created": "governed-by-build-evidence",
                        "org.opencontainers.image.revision": "a" * 40,
                        "org.opencontainers.image.source": (
                            "https://github.com/MikkelSchubert/adapterremoval"
                        ),
                        "org.opencontainers.image.version": "24.04",
                    }
                }
            }
        }

    def test_image_identity_uses_tool_version_instead_of_base_version(self) -> None:
        container_build.validate_inspection_identity(
            self.inspection(),
            "adapterremoval",
            "2.3.4",
        )

    def test_image_identity_rejects_missing_tool_version(self) -> None:
        inspection = self.inspection()
        labels = inspection["data"]["attributes"]["labels"]
        del labels["io.bijux.tool.version"]
        with self.assertRaisesRegex(
            container_build.ContractError,
            "io.bijux.tool.version",
        ):
            container_build.validate_inspection_identity(
                inspection,
                "adapterremoval",
                "2.3.4",
            )

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
