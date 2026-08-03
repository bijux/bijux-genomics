from __future__ import annotations

import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "corpus_materialization_spec", BENCHMARKS / "corpus_materialization_spec.py"
)
assert SPEC is not None and SPEC.loader is not None
specification = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = specification
SPEC.loader.exec_module(specification)


class CorpusMaterializationSpecificationTests(unittest.TestCase):
    def test_specification_freezes_plan_code_resources_and_acceptance(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            data_root = root / "study" / "data"
            plan = data_root / "evidence" / "acquisition-plans" / "plan.json"
            plan.parent.mkdir(parents=True)
            plan.write_text(
                json.dumps(
                    {
                        "schema_version": specification.PLAN_SCHEMA,
                        "portfolio_id": "human-short-read-diversity",
                        "corpus_id": "corpus-01",
                        "sample_count": 20,
                        "file_count": 30,
                        "total_size_bytes": 3638085565,
                    }
                ),
                encoding="utf-8",
            )
            repository = root / "repository"
            frozen = root / "frozen"
            materializer = frozen / "benchmarks" / "corpus_materialization.py"
            materializer.parent.mkdir(parents=True)
            materializer.write_text("print('materialize')\n", encoding="utf-8")
            output = (
                data_root
                / "evidence"
                / "materializations"
                / "commit"
                / "human-short-read-diversity"
            )
            governance = {
                "data_root": data_root,
                "site_id": "lunarc",
                "slurm_account": "project",
                "slurm_partitions": ["compute"],
            }
            commit = "a" * 40

            with mock.patch.object(
                specification, "validate_repository", return_value=commit
            ):
                pipeline, credentials, slurm = specification.build_specification(
                    governance,
                    plan,
                    repository,
                    frozen,
                    output,
                    jobs=4,
                )

        self.assertEqual(pipeline["repository"]["commit"], commit)
        self.assertEqual(pipeline["corpus"]["sample_count"], 20)
        self.assertEqual(pipeline["scheduler"]["resources"]["cpus"], 4)
        self.assertEqual(pipeline["execution"]["scientific_binaries"], [])
        self.assertEqual(pipeline["acceptance"]["gzip_integrity"], "required")
        self.assertFalse(credentials["credentials_required"])
        self.assertIn("#SBATCH --account=project", slurm)
        self.assertIn("corpus_materialization.py", slurm)

    def test_specification_rejects_plan_outside_data_root(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            outside = root / "plan.json"
            outside.write_text("{}", encoding="utf-8")
            governance = {
                "data_root": root / "study" / "data",
                "site_id": "lunarc",
                "slurm_account": "project",
                "slurm_partitions": ["compute"],
            }

            with self.assertRaisesRegex(
                specification.SpecificationError, "escapes governed root"
            ):
                specification.build_specification(
                    governance,
                    outside,
                    root / "repository",
                    root / "frozen",
                    root / "study" / "data" / "evidence" / "materializations" / "x",
                    jobs=1,
                )


if __name__ == "__main__":
    unittest.main()
