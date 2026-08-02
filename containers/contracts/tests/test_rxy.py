from __future__ import annotations

import importlib.util
from importlib.machinery import SourceFileLoader
import io
import json
import tempfile
import unittest
from pathlib import Path


SCRIPT = Path(__file__).parents[2] / "tools" / "rxy" / "rxy"


def load_rxy():
    spec = importlib.util.spec_from_loader("rxy", SourceFileLoader("rxy", str(SCRIPT)))
    if spec is None or spec.loader is None:
        raise RuntimeError("cannot load rxy source")
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


class RxyTests(unittest.TestCase):
    def run_idxstats(self, rows: str, *arguments: str) -> dict[str, object]:
        module = load_rxy()
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "idxstats.tsv"
            path.write_text(rows, encoding="utf-8")
            output = io.StringIO()
            result = module.main(["--idxstats", str(path), *arguments], stdout=output)
        self.assertEqual(result, 0)
        return json.loads(output.getvalue())

    def test_reports_female_from_upper_confidence_bound(self) -> None:
        report = self.run_idxstats("chr1\t1000\t5000\t0\nchrX\t1000\t1000\t0\nchrY\t1000\t0\t0\n")
        self.assertEqual(report["classification"], "female")
        self.assertEqual(report["y_fraction"], 0.0)
        self.assertTrue(report["sufficient_data"])

    def test_reports_male_from_lower_confidence_bound(self) -> None:
        report = self.run_idxstats("chrX\t1000\t800\t0\nchrY\t1000\t200\t0\n")
        self.assertEqual(report["classification"], "male")
        self.assertEqual(report["y_fraction"], 0.2)
        self.assertTrue(report["sufficient_data"])

    def test_supports_reference_specific_contig_names(self) -> None:
        report = self.run_idxstats(
            "NC_009175.3\t1000\t950\t0\nNC_009176.3\t1000\t50\t0\n",
            "--x-contig",
            "NC_009175.3",
            "--y-contig",
            "NC_009176.3",
        )
        self.assertEqual(report["sex_chromosome_reads"], 1000)
        self.assertEqual(report["classification"], "ambiguous")

    def test_rejects_malformed_idxstats(self) -> None:
        module = load_rxy()
        with tempfile.TemporaryDirectory() as temporary_directory:
            path = Path(temporary_directory) / "idxstats.tsv"
            path.write_text("chrX\t1000\tnot-a-count\t0\n", encoding="utf-8")
            with self.assertRaises(SystemExit):
                module.main(["--idxstats", str(path)], stdout=io.StringIO())


if __name__ == "__main__":
    unittest.main()
