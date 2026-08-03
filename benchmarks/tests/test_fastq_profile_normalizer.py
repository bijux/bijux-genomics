"""Tests for the tool-independent FASTQ profiling evidence adapter."""

from __future__ import annotations

import json
import subprocess
import tempfile
import unittest
from pathlib import Path

PROFILE_HEADER = (
    "sample_id\tinput_layout\tfile_count\tread_count\tbase_count\t"
    "minimum_read_length\tmaximum_read_length"
)


class FastqProfileNormalizerTests(unittest.TestCase):
    script = (Path(__file__).parents[1] / "execution/fastq_profile_reads.awk").resolve()

    def normalize(self, tool: str, content: str) -> str:
        with tempfile.TemporaryDirectory() as directory:
            raw = Path(directory) / "raw.tsv"
            raw.write_text(content, encoding="utf-8")
            completed = subprocess.run(
                [
                    "awk",
                    "-v",
                    "mode=normalize",
                    "-v",
                    f"tool={tool}",
                    "-f",
                    str(self.script),
                    str(raw),
                ],
                check=True,
                capture_output=True,
                text=True,
            )
        rows = sorted(completed.stdout.splitlines())
        return PROFILE_HEADER + "\n" + "\n".join(rows) + "\n"

    def evidence(self, mode: str, profile: str, *variables: str) -> dict[str, object]:
        with tempfile.TemporaryDirectory() as directory:
            normalized = Path(directory) / "profile.tsv"
            normalized.write_text(profile, encoding="utf-8")
            command = ["awk", "-v", f"mode={mode}"]
            for variable in variables:
                command.extend(("-v", variable))
            command.extend(("-f", str(self.script), str(normalized)))
            completed = subprocess.run(
                command,
                check=True,
                capture_output=True,
                text=True,
            )
        return json.loads(completed.stdout)

    def test_seqfu_and_seqkit_normalize_to_the_same_large_count_profile(self) -> None:
        seqfu = self.normalize(
            "seqfu",
            "File\t#Seq\tTotal bp\tAvg\tN50\tN75\tN90\tauN\tMin\tMax\t%GC\n"
            "/input/PAIR_1.fastq.gz\t3000000000\t4000000000\t1\t1\t1\t1\t1\t20\t127\t50\n"
            "/input/PAIR_2.fastq.gz\t3000000000\t4100000000\t1\t1\t1\t1\t1\t18\t126\t50\n",
        )
        seqkit = self.normalize(
            "seqkit",
            "file\tformat\ttype\tnum_seqs\tsum_len\tmin_len\tavg_len\tmax_len\n"
            "/input/PAIR_1.fastq.gz\tFASTQ\tDNA\t3000000000\t4000000000\t20\t1\t127\n"
            "/input/PAIR_2.fastq.gz\tFASTQ\tDNA\t3000000000\t4100000000\t18\t1\t126\n",
        )

        self.assertEqual(seqfu, seqkit)
        self.assertIn("PAIR\tpaired_end\t2\t6000000000\t8100000000\t18\t127", seqfu)

    def test_bundle_and_probe_consume_the_same_normalized_profile(self) -> None:
        profile = (
            PROFILE_HEADER + "\nSAMPLE\tsingle_end\t1\t235860\t17621963\t20\t127\n"
        )

        bundle = self.evidence("bundle", profile, "tool=seqfu")
        probe = self.evidence("probe", profile)
        metrics = self.evidence(
            "metrics", profile, "input_bytes=11610657", "output_bytes=512"
        )

        self.assertEqual(bundle["results"][0]["read_count"], 235860)
        self.assertEqual(
            probe["consumer_stage_id"], "benchmark.fastq.profile_reads.compare"
        )
        self.assertEqual(probe["read_count"], 235860)
        self.assertEqual(metrics["admission_mismatch_count"], 0)
        self.assertEqual(metrics["input_bytes"], 11610657)


if __name__ == "__main__":
    unittest.main()
