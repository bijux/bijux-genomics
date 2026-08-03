from __future__ import annotations

import importlib.util
import json
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path
from unittest.mock import patch

import yaml

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "mapping_summary_specs", BENCHMARKS / "mapping_summary_specs.py"
)
assert SPEC is not None and SPEC.loader is not None
specifications = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = specifications
SPEC.loader.exec_module(specifications)


class MappingSummarySpecificationTests(unittest.TestCase):
    def fixtures(self, root: Path) -> tuple[Path, Path, Path, Path]:
        study = root / "study"
        repository = root / "repository"
        frozen = root / "frozen"
        for path in (study / "containers", study / "data", study / "results", repository, frozen):
            path.mkdir(parents=True)
        governance = root / "governance.yml"
        governance.write_text(
            yaml.safe_dump(
                {
                    "schema_version": specifications.GOVERNANCE_SCHEMA,
                    "paths": {
                        "study_root": str(study),
                        "containers_root": str(study / "containers"),
                        "data_root": str(study / "data"),
                        "results_root": str(study / "results"),
                        "active_repository": str(repository),
                        "frozen_repositories": str(frozen),
                    },
                    "slurm": {"account": "research", "partitions": ["compute"]},
                }
            ),
            encoding="utf-8",
        )
        inventory = root / "images.json"
        inventory.write_text(
            json.dumps(
                {
                    "schema_version": specifications.IMAGE_SCHEMA,
                    "tools": [
                        {
                            "tool": tool,
                            "support_disposition": "retained",
                            "state": "current",
                            "selected_image": f"study://containers/apptainer/{tool}.sif",
                            "selected_image_sha256": digit * 64,
                        }
                        for tool, digit in (("picard", "a"), ("samtools", "b"))
                    ],
                }
            ),
            encoding="utf-8",
        )
        index = root / ("f" * 40) / "data-index.json"
        index.parent.mkdir()
        index.write_text(
            json.dumps(
                {
                    "schema_version": specifications.DATA_SCHEMA,
                    "records": [
                        {
                            "id": specifications.CORPUS_ID,
                            "benchmark_eligibility": "eligible",
                            "catalog_path": "study://data/catalog/corpus.json",
                            "catalog_sha256": "c" * 64,
                            "files": [
                                {
                                    "path": "study://data/corpora/sample.sorted.bam",
                                    "sha256": "d" * 64,
                                },
                                {
                                    "path": "study://data/corpora/sample.sorted.bam.bai",
                                    "sha256": "e" * 64,
                                },
                            ],
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        return governance, index, inventory, study

    def test_preparation_emits_complete_tool_contracts(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance, index, inventory, study = self.fixtures(root)
            output = root / "specifications"
            with patch.object(specifications, "current_commit", return_value="f" * 40):
                destination = specifications.prepare(
                    governance,
                    index,
                    inventory,
                    output,
                    "2026-08-03T00:00:00+00:00",
                )
            pipelines = {
                tool: yaml.safe_load(
                    (
                        destination / f"run-2004-{tool}" / "config-pipeline.yml"
                    ).read_text()
                )
                for tool in specifications.TOOLS
            }

        self.assertEqual(set(pipelines), set(specifications.TOOLS))
        for tool, pipeline in pipelines.items():
            self.assertEqual(pipeline["run"]["stage"], "bam.mapping_summary")
            self.assertEqual(pipeline["repository"]["commit"], "f" * 40)
            self.assertEqual(pipeline["execution"]["threads"], 1)
            self.assertEqual(len(pipeline["outputs"]["required_files"]), 13)
            self.assertEqual(
                pipeline["outputs"]["result_root"],
                str(
                    study
                    / "results/benchmarks/run-2004-bam-mapping-summary"
                    / tool
                ),
            )

    def test_commands_encode_one_warmup_and_three_measured_repetitions(self) -> None:
        roots = {
            "results_root": Path("/study/results"),
            "frozen_repositories": Path("/repository-runs"),
            "account": "research",
            "partition": "compute",
        }
        inputs = [
            {
                "id": "bam",
                "kind": "data",
                "path": "/study/data/sample.bam",
                "mount_path": "/input/ERR2112767.sorted.bam",
                "sha256": "a" * 64,
            }
        ]

        picard = specifications.pipeline(
            "picard",
            roots,
            "f" * 40,
            {"path": "/study/containers/picard.sif", "sha256": "b" * 64},
            inputs,
            "2026-08-03T00:00:00+00:00",
        )["commands"]["real"][2]
        samtools = specifications.pipeline(
            "samtools",
            roots,
            "f" * 40,
            {"path": "/study/containers/samtools.sif", "sha256": "c" * 64},
            inputs,
            "2026-08-03T00:00:00+00:00",
        )["commands"]["real"][2]

        self.assertIn("for repetition in 1 2 3", picard)
        self.assertIn("mkdir -p /scratch/picard-warmup", picard)
        self.assertIn('mkdir -p "/scratch/picard-${repetition}"', picard)
        self.assertIn('$1=="PAIR" || $1=="UNPAIRED"', picard)
        self.assertNotIn('$1=="FIRST_OF_PAIR"', picard)
        self.assertIn("samtools flagstat", samtools)
        self.assertIn('$4=="primary" && $5=="mapped"', samtools)
        self.assertIn('"warmup_repetitions":1', picard)
        self.assertIn('"measured_repetitions":3', samtools)
        for command in (picard, samtools):
            syntax = subprocess.run(
                ["sh", "-n", "-c", command],
                check=False,
                capture_output=True,
                text=True,
            )
            self.assertEqual(syntax.returncode, 0, syntax.stderr)

    def test_ineligible_corpus_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance, index, inventory, _ = self.fixtures(root)
            data = json.loads(index.read_text())
            data["records"][0]["benchmark_eligibility"] = "incomplete"
            index.write_text(json.dumps(data), encoding="utf-8")

            with (
                patch.object(specifications, "current_commit", return_value="f" * 40),
                self.assertRaisesRegex(
                    specifications.SpecificationError,
                    "not uniquely benchmark-eligible",
                ),
            ):
                specifications.prepare(governance, index, inventory, root / "output")

    def test_data_index_must_match_the_frozen_commit(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            governance, index, inventory, _ = self.fixtures(root)
            mismatched = root / "data-index.json"
            mismatched.write_bytes(index.read_bytes())

            with (
                patch.object(specifications, "current_commit", return_value="f" * 40),
                self.assertRaisesRegex(
                    specifications.SpecificationError,
                    "exact repository commit",
                ),
            ):
                specifications.prepare(
                    governance, mismatched, inventory, root / "output"
                )


if __name__ == "__main__":
    unittest.main()
