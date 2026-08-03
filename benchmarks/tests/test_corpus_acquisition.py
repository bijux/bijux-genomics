from __future__ import annotations

import datetime as dt
import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "corpus_acquisition", BENCHMARKS / "corpus_acquisition.py"
)
assert SPEC is not None and SPEC.loader is not None
corpus_acquisition = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = corpus_acquisition
SPEC.loader.exec_module(corpus_acquisition)


class CorpusAcquisitionTests(unittest.TestCase):
    def write_spec(self, root: Path) -> Path:
        path = root / "corpus.toml"
        path.write_text(
            'schema_version = "bijux.corpus_spec.v1"\n'
            'corpus_id = "real-panel"\n'
            'portfolio_id = "real-panel-diversity"\n'
            'species = "Homo sapiens"\n'
            "target_total = 2\n"
            "[[samples]]\n"
            'accession = "ERR000001"\n'
            'study_accession = "PRJEB1"\n'
            'era = "ancient"\n'
            'layout = "se"\n'
            'size_band = "small"\n'
            'reason = "ancient single-end coverage"\n'
            "[[samples]]\n"
            'accession = "ERR000002"\n'
            'study_accession = "PRJEB1"\n'
            'era = "modern"\n'
            'layout = "pe"\n'
            'size_band = "medium"\n'
            'reason = "modern paired-end coverage"\n',
            encoding="utf-8",
        )
        return path

    def test_resolver_locks_real_file_urls_digests_and_sizes(self) -> None:
        records = {
            "ERR000001": {
                "run_accession": "ERR000001",
                "study_accession": "PRJEB1",
                "sample_accession": "SAMEA1",
                "experiment_accession": "ERX1",
                "scientific_name": "Homo sapiens",
                "library_layout": "SINGLE",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "library_selection": "RANDOM",
                "instrument_platform": "ILLUMINA",
                "instrument_model": "Illumina HiSeq",
                "base_count": "1000",
                "read_count": "10",
                "fastq_ftp": "ftp.sra.ebi.ac.uk/one.fastq.gz",
                "fastq_md5": "a" * 32,
                "fastq_bytes": "101",
                "first_public": "2020-01-01",
                "last_updated": "2020-01-02",
            },
            "ERR000002": {
                "run_accession": "ERR000002",
                "study_accession": "PRJEB1",
                "sample_accession": "SAMEA2",
                "experiment_accession": "ERX2",
                "scientific_name": "Homo sapiens",
                "library_layout": "PAIRED",
                "library_source": "GENOMIC",
                "library_strategy": "WGS",
                "library_selection": "RANDOM",
                "instrument_platform": "ILLUMINA",
                "instrument_model": "Illumina HiSeq",
                "base_count": "2000",
                "read_count": "20",
                "fastq_ftp": "ftp.sra.ebi.ac.uk/two_1.fastq.gz;ftp.sra.ebi.ac.uk/two_2.fastq.gz",
                "fastq_md5": f"{'b' * 32};{'c' * 32}",
                "fastq_bytes": "202;303",
                "first_public": "2020-01-01",
                "last_updated": "2020-01-02",
            },
        }
        with tempfile.TemporaryDirectory() as directory:
            spec = self.write_spec(Path(directory))
            plan = corpus_acquisition.resolve_specification(
                spec,
                records.__getitem__,
                dt.datetime(2026, 8, 3, tzinfo=dt.timezone.utc),
            )

        self.assertEqual(plan["sample_count"], 2)
        self.assertEqual(plan["portfolio_id"], "real-panel-diversity")
        self.assertEqual(plan["file_count"], 3)
        self.assertEqual(plan["total_size_bytes"], 606)
        self.assertEqual(
            plan["samples"][1]["files"][1]["source_md5"], "c" * 32
        )
        self.assertEqual(plan["samples"][1]["files"][1]["mate"], 2)

    def test_layout_disagreement_is_rejected(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            spec = self.write_spec(Path(directory))

            with self.assertRaisesRegex(
                corpus_acquisition.AcquisitionError, "layout disagrees"
            ):
                corpus_acquisition.resolve_specification(
                    spec,
                    lambda accession: {
                        "run_accession": accession,
                        "study_accession": "PRJEB1",
                        "library_layout": "PAIRED",
                        "scientific_name": "Homo sapiens",
                    },
                    dt.datetime(2026, 8, 3, tzinfo=dt.timezone.utc),
                )


if __name__ == "__main__":
    unittest.main()
