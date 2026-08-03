"""Tests for the governed real FASTQ read-profile campaign."""

from __future__ import annotations

import unittest
from pathlib import Path

from benchmarks.runs import fastq_profile_reads as specifications


class FastqProfileReadsSpecificationTests(unittest.TestCase):
    def test_expected_admission_aggregates_paired_files_as_sequences(self) -> None:
        catalog = {
            "samples": [
                {"accession": "SINGLE", "layout": "se"},
                {"accession": "PAIRED", "layout": "pe"},
            ],
            "files": [
                {
                    "accession": "SINGLE",
                    "read_count": 7,
                    "base_count": 700,
                },
                {"accession": "PAIRED", "read_count": 5, "base_count": 500},
                {"accession": "PAIRED", "read_count": 5, "base_count": 510},
            ],
        }

        rows = specifications.expected_admission_rows(catalog)

        self.assertEqual(
            rows,
            [
                "PAIRED\tpaired_end\t2\t10\t1010",
                "SINGLE\tsingle_end\t1\t7\t700",
            ],
        )

    def test_tool_commands_match_the_governed_planner_dialects(self) -> None:
        paths = ["/input/sample_1.fastq.gz", "/input/sample_2.fastq.gz"]

        self.assertEqual(
            specifications.tool_command("seqfu", paths)[:6],
            ["seqfu", "stats", "--threads", "4", "--gc", paths[0]],
        )
        for tool in ("seqkit", "seqkit_stats"):
            self.assertEqual(
                specifications.tool_command(tool, paths)[:7],
                [tool, "stats", "-a", "-T", "-j", "4", paths[0]],
            )

    def test_measured_command_uses_frozen_normalizer_and_common_probe(self) -> None:
        command = specifications.shell_command(
            "seqfu", ["/input/sample.fastq.gz"], 1200, warmup=False
        )

        self.assertEqual(command[:2], ["sh", "-c"])
        self.assertIn("repetition-{repetition}", command[2])
        self.assertIn(
            "/repository/benchmarks/execution/fastq_profile_reads.awk", command[2]
        )
        self.assertIn("mode=normalize", command[2])
        self.assertIn("mode=bundle", command[2])
        self.assertIn("mode=probe", command[2])
        self.assertIn("diff -u /input/expected-profile.tsv", command[2])

    def test_pipeline_freezes_interface_parameters_and_repetitions(self) -> None:
        interface = {
            "accepted_input_contracts": ["bijux.fastq.artifact.single_end.v1"],
            "output_contract": "bijux.benchmark.fastq.profile_reads.bundle.v1",
            "normalized_result_contract": "benchmarks/contracts/schemas/result.json",
            "adapters": {tool: "bijux::profile_reads" for tool in specifications.TOOLS},
            "downstream_probe": {
                "consumer_stage_id": "benchmark.fastq.profile_reads.compare",
                "input_contract": "bijux.benchmark.fastq.profile_reads.bundle.v1",
                "source": "benchmarks/execution/fastq_profile_reads.awk",
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
            ["/input/sample.fastq.gz"],
            1200,
            "2026-08-03T20:00:00+00:00",
            {
                "version": ["seqfu", "--version"],
                "help": ["seqfu", "--help"],
                "version_regex": "1[.]26[.]0",
            },
            {"schema_version": "bijux.benchmark.parameter_evidence.v1"},
            interface,
        )

        self.assertEqual(pipeline["run"]["kind"], "benchmark")
        self.assertEqual(pipeline["measurement"]["measured_repetitions"], 3)
        self.assertEqual(pipeline["execution"]["threads"], 4)
        self.assertEqual(
            pipeline["interface"]["downstream_probe"]["stage_id"],
            "benchmark.fastq.profile_reads.compare",
        )
        self.assertEqual(len(pipeline["outputs"]["required_files"]), 21)


if __name__ == "__main__":
    unittest.main()
