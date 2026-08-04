"""Tests for the governed real FASTQ read-length campaign."""

from __future__ import annotations

import gzip
import tempfile
import unittest
from pathlib import Path

from benchmarks.runs import fastq_profile_read_lengths as specifications


class FastqProfileReadLengthsSpecificationTests(unittest.TestCase):
    def test_tool_commands_match_the_governed_exact_row_dialects(self) -> None:
        paths = ["/input/sample_1.fastq.gz", "/input/sample_2.fastq.gz"]

        self.assertIn(
            "seqfu tab /input/sample_1.fastq.gz",
            specifications.tool_command("seqfu", paths),
        )
        self.assertTrue(specifications.tool_command("seqfu", paths).startswith("( "))
        self.assertIn(
            "seqfu tab /input/sample_2.fastq.gz",
            specifications.tool_command("seqfu", paths),
        )
        self.assertIn(
            "seqkit fx2tab --name --only-id --length --threads 4",
            specifications.tool_command("seqkit", paths),
        )

    def test_measured_command_uses_frozen_normalizer_and_exact_digest_gate(self) -> None:
        command = specifications.shell_command(
            "seqfu",
            [
                {
                    "sample_id": "ERR1",
                    "input_layout": "paired_end",
                    "mounted_fastq": ["/input/ERR1_1.fastq.gz", "/input/ERR1_2.fastq.gz"],
                    "expected_summary_mount": "/input/ERR1.summary.tsv",
                    "expected_histogram_mount": "/input/ERR1.histogram.tsv",
                    "expected_exact_sha256": "a" * 64,
                }
            ],
            1200,
            warmup=False,
        )

        self.assertEqual(command[:2], ["sh", "-c"])
        self.assertIn("sha256sum", command[2])
        self.assertIn(
            "/repository/benchmarks/execution/fastq_read_lengths.awk", command[2]
        )
        self.assertIn("mode=bundle", command[2])
        self.assertIn("mode=probe", command[2])
        self.assertIn("mode=metrics", command[2])
        self.assertIn(
            "diff -u /input/ERR1.summary.tsv", command[2]
        )
        self.assertIn("diff -u /input/ERR1.histogram.tsv", command[2])
        self.assertNotIn("do;", command[2])
        self.assertIn(
            '-v artifact="repetitions/repetition-${repetition}.ERR1.histogram.tsv"',
            command[2],
        )

    def test_independent_sample_evidence_derives_exact_summary_and_digest(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            study = root / "study"
            data = study / "data"
            data.mkdir(parents=True)
            fastq = data / "ERR1.fastq.gz"
            with gzip.open(fastq, "wt", encoding="utf-8") as stream:
                stream.write("@read-1\nACGT\n+\nFFFF\n@read-2\nAACGTA\n+\nFFFFFF\n")

            evidence = specifications.independent_sample_evidence(
                {
                    "sample_id": "ERR1",
                    "input_layout": "single_end",
                    "files": [{"governed_path": "study://data/ERR1.fastq.gz"}],
                },
                study,
            )

        self.assertEqual(evidence["read_count"], 2)
        self.assertEqual(evidence["minimum_read_length"], 4)
        self.assertEqual(evidence["maximum_read_length"], 6)
        self.assertEqual(evidence["distinct_read_lengths"], 2)
        self.assertEqual(
            evidence["summary_line"],
            "ERR1\tsingle_end\t1\t2\t4\t5\t5\t6\t2",
        )
        self.assertEqual(len(evidence["exact_rows_sha256"]), 64)

    def test_pipeline_freezes_interface_parameters_and_sample_outputs(self) -> None:
        interface = {
            "accepted_input_contracts": ["bijux.fastq.artifact.single_end.v1"],
            "output_contract": "bijux.benchmark.fastq.profile_read_lengths.bundle.v1",
            "normalized_result_contract": "benchmarks/contracts/schemas/result.json",
            "adapters": {
                tool: "bijux::profile_read_lengths" for tool in specifications.TOOLS
            },
            "downstream_probe": {
                "consumer_stage_id": "benchmark.fastq.profile_read_lengths.compare",
                "input_contract": "bijux.benchmark.fastq.profile_read_lengths.bundle.v1",
                "source": "benchmarks/execution/fastq_read_lengths.awk",
            },
        }
        pipeline = specifications.pipeline(
            "seqfu",
            {
                "results_root": Path("/study/results"),
                "frozen_repositories": Path("/study/frozen"),
                "account": "science",
                "partition": "compute",
            },
            "f" * 40,
            {"path": "/study/containers/seqfu.sif", "sha256": "a" * 64},
            [
                {
                    "id": "fastq_sample",
                    "kind": "data",
                    "path": "/study/data/sample.fastq.gz",
                    "mount_path": "/input/sample.fastq.gz",
                    "sha256": "b" * 64,
                }
            ],
            [
                {
                    "sample_id": "ERR1",
                    "input_layout": "single_end",
                    "mounted_fastq": ["/input/sample.fastq.gz"],
                    "expected_summary_mount": "/input/ERR1.summary.tsv",
                    "expected_histogram_mount": "/input/ERR1.histogram.tsv",
                    "expected_exact_sha256": "c" * 64,
                }
            ],
            1200,
            "2026-08-04T00:00:00+00:00",
            {
                "version": ["seqfu", "--version"],
                "help": ["seqfu", "--help"],
                "version_regex": "1[.]26[.]0",
            },
            {"schema_version": "bijux.benchmark.parameter_evidence.v1"},
            interface,
        )

        self.assertEqual(pipeline["run"]["id"], "run-2042")
        self.assertEqual(pipeline["run"]["kind"], "benchmark")
        self.assertEqual(pipeline["measurement"]["measured_repetitions"], 3)
        self.assertEqual(
            pipeline["interface"]["downstream_probe"]["stage_id"],
            "benchmark.fastq.profile_read_lengths.compare",
        )
        self.assertTrue(
            any(
                path.endswith("ERR1.histogram.tsv")
                for path in pipeline["outputs"]["required_files"]
            )
        )


if __name__ == "__main__":
    unittest.main()
