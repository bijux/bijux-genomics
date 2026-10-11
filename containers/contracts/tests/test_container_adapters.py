from __future__ import annotations

import importlib.machinery
import importlib.util
import json
import subprocess
import tempfile
import unittest
from pathlib import Path

TOOLS = Path(__file__).parents[2] / "tools"


def load_python_adapter(name: str):
    path = TOOLS / name / name
    loader = importlib.machinery.SourceFileLoader(f"container_{name}_adapter", str(path))
    spec = importlib.util.spec_from_loader(loader.name, loader)
    assert spec is not None
    module = importlib.util.module_from_spec(spec)
    loader.exec_module(module)
    return module


class IbdHapAdapterTests(unittest.TestCase):
    @classmethod
    def setUpClass(cls) -> None:
        cls.adapter = load_python_adapter("ibdhap")

    def test_uninformative_frequency_is_skipped_without_masking_invalid_data(self) -> None:
        self.assertIsNone(self.adapter.parse_frequency("AC=0;AF=0;AN=4", 12))
        self.assertEqual(self.adapter.parse_frequency("AC=1;AF=0.25;AN=4", 13), 0.25)
        with self.assertRaisesRegex(self.adapter.InputError, "AF outside"):
            self.adapter.parse_frequency("AF=1.2", 14)

    def test_vcf_selection_reports_excluded_frequency_rows(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            vcf = Path(directory) / "panel.vcf"
            vcf.write_text(
                "##fileformat=VCFv4.2\n"
                "#CHROM\tPOS\tID\tREF\tALT\tQUAL\tFILTER\tINFO\tFORMAT\tA\tB\n"
                "22\t100\trs0\tA\tG\t.\tPASS\tAF=0\tGT\t0|0\t0|0\n"
                "22\t110\trs1\tA\tG\t.\tPASS\tAF=0.1\tGT\t0|1\t0|0\n"
                "22\t120\trs2\tC\tT\t.\tPASS\tAF=0.2\tGT\t1|0\t0|1\n",
                encoding="utf-8",
            )

            records, selection = self.adapter.load_vcf(
                vcf,
                ("A", "B"),
                ("22", 90, 130),
                500,
            )

        self.assertEqual(len(records), 2)
        self.assertEqual(selection["skipped_uninformative_frequency_rows"], 1)
        self.assertEqual(selection["selected_marker_rows"], 2)

    def test_score_summary_reads_runtime_scores_and_records_selection(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            root = Path(directory)
            scores = root / "scores.tsv"
            scores.write_text(
                "marker 100 0 " + " ".join(["1"] + ["0"] * 14) + "\n"
                "marker 120 0 " + " ".join(["0"] * 14 + ["1"]) + "\n",
                encoding="utf-8",
            )
            summary = root / "summary.json"
            selection = {"selected_marker_rows": 2}

            self.adapter.summarize_scores(
                scores,
                summary,
                root / "source.vcf.gz",
                root / "map.gmap.gz",
                ("A", "B"),
                "22:90-130",
                selection,
            )

            record = json.loads(summary.read_text(encoding="utf-8"))

        self.assertEqual(record["score_rows"], 2)
        self.assertEqual(record["marker_selection"], selection)
        self.assertTrue(record["source_vcf"].endswith("source.vcf.gz"))


class GermlineAdapterTests(unittest.TestCase):
    def test_missing_native_inputs_fail_before_upstream_execution(self) -> None:
        wrapper = TOOLS / "germline" / "germline"
        with tempfile.TemporaryDirectory() as directory:
            completed = subprocess.run(
                [
                    "sh",
                    str(wrapper),
                    "-input",
                    "/governed-missing-panel.ped",
                    "/governed-missing-panel.map",
                    "-output",
                    str(Path(directory) / "germline"),
                ],
                check=False,
                capture_output=True,
                text=True,
            )

        self.assertEqual(completed.returncode, 2)
        self.assertIn("input PED does not exist", completed.stderr)


class PicardAdapterTests(unittest.TestCase):
    def run_version_probe(
        self, native_version: str
    ) -> subprocess.CompletedProcess[str]:
        wrapper = TOOLS / "picard" / "picard"
        with tempfile.TemporaryDirectory() as directory:
            binary_root = Path(directory)
            java = binary_root / "java"
            java.write_text(
                "#!/bin/sh\n"
                "printf 'ERROR StatusConsoleListener Could not determine local host name\\n' >&2\n"
                f"printf 'Version:{native_version}\\n' >&2\n"
                "exit 1\n",
                encoding="utf-8",
            )
            java.chmod(0o755)
            completed = subprocess.run(
                ["sh", str(wrapper), "--version"],
                check=False,
                capture_output=True,
                text=True,
                env={"PATH": f"{binary_root}:/usr/bin:/bin"},
            )
        return completed

    def test_version_probe_tolerates_unrelated_jvm_diagnostics(self) -> None:
        completed = self.run_version_probe("3.3.0")

        self.assertEqual(completed.returncode, 0)
        self.assertEqual(completed.stdout, "picard 3.3.0\n")
        self.assertEqual(completed.stderr, "")

    def test_version_probe_rejects_a_different_release(self) -> None:
        completed = self.run_version_probe("3.2.0")

        self.assertEqual(completed.returncode, 70)
        self.assertIn("unexpected Picard version probe", completed.stderr)


if __name__ == "__main__":
    unittest.main()
