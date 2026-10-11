"""Tests for the exact FASTQ read-length benchmark boundary."""

from __future__ import annotations

import json
import subprocess
import tempfile
import tomllib
import unittest
from pathlib import Path


class FastqReadLengthContractTests(unittest.TestCase):
    root = Path(__file__).parents[2]
    normalizer = root / "benchmarks/execution/fastq_read_lengths.awk"

    def normalize(self, mode: str, rows: str) -> subprocess.CompletedProcess[str]:
        with tempfile.TemporaryDirectory() as directory:
            source = Path(directory) / "backend.tsv"
            source.write_text(rows, encoding="utf-8")
            return subprocess.run(
                [
                    "awk",
                    "-v",
                    f"mode={mode}",
                    "-v",
                    "sample_id=ERR1",
                    "-v",
                    "input_layout=paired_end",
                    "-f",
                    str(self.normalizer),
                    str(source),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

    def test_common_rows_produce_exact_summary_and_histogram(self) -> None:
        rows = "read-1\t90\nread-2\t92\nread-3\t92\nread-4\t98\n"

        summary = self.normalize("summary", rows)
        histogram = self.normalize("histogram", rows)

        self.assertEqual(summary.returncode, 0, summary.stderr)
        self.assertEqual(
            summary.stdout.splitlines()[1],
            "ERR1\tpaired_end\t2\t4\t90\t93\t92\t98\t3",
        )
        self.assertEqual(
            histogram.stdout.splitlines(),
            [
                "sample_id\tread_length\tcount",
                "ERR1\t90\t1",
                "ERR1\t92\t2",
                "ERR1\t98\t1",
            ],
        )

    def test_summary_dialect_is_rejected_as_non_observational(self) -> None:
        result = self.normalize("summary", "file\tnum_seqs\tmin_len\tmax_len\n")

        self.assertNotEqual(result.returncode, 0)
        self.assertIn("record_id<TAB>length", result.stderr)

    def test_interface_campaign_and_schema_close_the_same_tool_set(self) -> None:
        interface_catalog = tomllib.loads(
            (self.root / "benchmarks/contracts/stage-interfaces.toml").read_text()
        )
        campaign_catalog = tomllib.loads(
            (self.root / "benchmarks/contracts/stage-campaigns.toml").read_text()
        )
        schema = json.loads(
            (
                self.root
                / "benchmarks/contracts/schemas/fastq-profile-read-lengths-result.schema.json"
            ).read_text()
        )
        interface = next(
            item
            for item in interface_catalog["interfaces"]
            if item["stage_id"] == "fastq.profile_read_lengths"
        )
        campaign = next(
            item
            for item in campaign_catalog["campaigns"]
            if item["run_id"] == "run-2042"
        )

        self.assertEqual(interface["tool_ids"], ["seqfu", "seqkit"])
        self.assertEqual(campaign["tool_ids"], interface["tool_ids"])
        self.assertEqual(
            schema["properties"]["tool_id"]["enum"], interface["tool_ids"]
        )
        self.assertEqual(campaign["scheduler"], "slurm")
        self.assertTrue(campaign["container_only"])


if __name__ == "__main__":
    unittest.main()
