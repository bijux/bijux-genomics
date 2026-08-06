from __future__ import annotations

import gzip
import hashlib
import importlib.util
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
SPEC = importlib.util.spec_from_file_location(
    "corpus_materialization", BENCHMARKS / "corpus_materialization.py"
)
assert SPEC is not None and SPEC.loader is not None
materialization = importlib.util.module_from_spec(SPEC)
sys.modules[SPEC.name] = materialization
SPEC.loader.exec_module(materialization)


class CorpusMaterializationTests(unittest.TestCase):
    def fastq_bytes(self) -> bytes:
        payload = b"@read-one\nACGT\n+\nIIII\n@read-two\nAACCGG\n+\nIIIIII\n"
        output = io.BytesIO()
        with gzip.GzipFile(fileobj=output, mode="wb", mtime=0) as stream:
            stream.write(payload)
        return output.getvalue()

    def write_plan(self, data_root: Path, payload: bytes) -> Path:
        plan = data_root / "evidence" / "acquisition-plans" / "plan.json"
        plan.parent.mkdir(parents=True)
        plan.write_text(
            json.dumps(
                {
                    "schema_version": materialization.PLAN_SCHEMA,
                    "corpus_id": "corpus-01",
                    "portfolio_id": "human-short-read-diversity",
                    "species": "Homo sapiens",
                    "retrieved_at_utc": "2026-08-03T00:00:00+00:00",
                    "sample_count": 1,
                    "file_count": 1,
                    "total_size_bytes": len(payload),
                    "samples": [
                        {
                            "accession": "ERR000001",
                            "study_accession": "PRJEB1",
                            "sample_accession": "SAMEA1",
                            "experiment_accession": "ERX1",
                            "era": "ancient",
                            "layout": "se",
                            "size_band": "under_100mb",
                            "selection_reason": "real ancient single-end coverage",
                            "scientific_name": "Homo sapiens",
                            "library_source": "GENOMIC",
                            "library_strategy": "WGS",
                            "library_selection": "RANDOM",
                            "instrument_platform": "ILLUMINA",
                            "instrument_model": "Illumina HiSeq",
                            "archive_base_count": 10,
                            "archive_read_count": 2,
                            "first_public": "2020-01-01",
                            "last_updated": "2020-01-02",
                            "files": [
                                {
                                    "mate": None,
                                    "name": "ERR000001.fastq.gz",
                                    "source_url": "https://archive.example/ERR000001.fastq.gz",
                                    "source_md5": hashlib.md5(
                                        payload, usedforsecurity=False
                                    ).hexdigest(),
                                    "size_bytes": len(payload),
                                }
                            ],
                        }
                    ],
                }
            ),
            encoding="utf-8",
        )
        return plan

    def test_materialization_streams_validates_and_admits_fastq(self) -> None:
        payload = self.fastq_bytes()
        with tempfile.TemporaryDirectory() as directory:
            data_root = Path(directory) / "data"
            plan = self.write_plan(data_root, payload)

            catalog = materialization.materialize(
                plan,
                data_root,
                jobs=1,
                retries=0,
                timeout=10,
                opener=lambda _url, _timeout: io.BytesIO(payload),
            )

            target = Path(catalog["files"][0]["path"])
            self.assertEqual(catalog["files"][0]["read_count"], 2)
            self.assertEqual(catalog["files"][0]["base_count"], 10)
            self.assertEqual(catalog["files"][0]["materialization_status"], "downloaded")
            self.assertEqual(target.read_bytes(), payload)
            self.assertEqual(target.stat().st_mode & 0o777, 0o444)
            self.assertTrue(
                (data_root / "catalog" / "human-short-read-diversity.json").is_file()
            )

    def test_existing_file_must_match_source_identity(self) -> None:
        payload = self.fastq_bytes()
        with tempfile.TemporaryDirectory() as directory:
            data_root = Path(directory) / "data"
            plan = self.write_plan(data_root, payload)
            target = (
                data_root
                / "corpora"
                / "human-short-read-diversity"
                / "raw"
                / "ERR000001"
                / "ERR000001.fastq.gz"
            )
            target.parent.mkdir(parents=True)
            target.write_bytes(b"not-the-declared-source")

            with self.assertRaisesRegex(
                materialization.MaterializationError, "source identity mismatch"
            ):
                materialization.materialize(
                    plan,
                    data_root,
                    jobs=1,
                    retries=0,
                    timeout=10,
                    opener=lambda _url, _timeout: io.BytesIO(payload),
                )


if __name__ == "__main__":
    unittest.main()
