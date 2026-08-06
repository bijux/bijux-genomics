from __future__ import annotations

import hashlib
import importlib.util
import json
import sys
import tempfile
import unittest
from pathlib import Path

BENCHMARKS = Path(__file__).parents[1]
ACQUISITION_SPEC = importlib.util.spec_from_file_location(
    "corpus_acquisition", BENCHMARKS / "corpus_acquisition.py"
)
assert ACQUISITION_SPEC is not None and ACQUISITION_SPEC.loader is not None
acquisition = importlib.util.module_from_spec(ACQUISITION_SPEC)
sys.modules[ACQUISITION_SPEC.name] = acquisition
ACQUISITION_SPEC.loader.exec_module(acquisition)
ENRICHMENT_SPEC = importlib.util.spec_from_file_location(
    "corpus_catalog_enrichment", BENCHMARKS / "corpus_catalog_enrichment.py"
)
assert ENRICHMENT_SPEC is not None and ENRICHMENT_SPEC.loader is not None
enrichment = importlib.util.module_from_spec(ENRICHMENT_SPEC)
ENRICHMENT_SPEC.loader.exec_module(enrichment)


class CorpusCatalogEnrichmentTests(unittest.TestCase):
    def context(self) -> dict[str, object]:
        return {
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
            "archive_base_count": 4,
            "archive_read_count": 1,
            "first_public": "2020-01-01",
            "last_updated": "2020-01-02",
        }

    def records(self, root: Path) -> tuple[Path, Path, Path, bytes]:
        data_root = root / "data"
        raw = (
            data_root
            / "corpora"
            / "human-short-read-diversity"
            / "raw"
            / "ERR000001"
            / "ERR000001.fastq.gz"
        )
        raw.parent.mkdir(parents=True)
        raw_bytes = b"immutable-real-fastq-bytes"
        raw.write_bytes(raw_bytes)
        source_md5 = hashlib.md5(raw_bytes, usedforsecurity=False).hexdigest()
        source_url = "https://archive.example/ERR000001.fastq.gz"
        plan_path = data_root / "evidence" / "acquisition-plans" / "enriched.json"
        plan_path.parent.mkdir(parents=True)
        context = self.context()
        plan = {
            "schema_version": acquisition.PLAN_SCHEMA,
            "corpus_id": "corpus-01",
            "portfolio_id": "human-short-read-diversity",
            "species": "Homo sapiens",
            "retrieved_at_utc": "2026-08-03T00:00:00+00:00",
            "sample_count": 1,
            "file_count": 1,
            "total_size_bytes": len(raw_bytes),
            "samples": [
                {
                    **context,
                    "files": [
                        {
                            "mate": None,
                            "name": raw.name,
                            "source_url": source_url,
                            "source_md5": source_md5,
                            "size_bytes": len(raw_bytes),
                        }
                    ],
                }
            ],
        }
        plan_path.write_text(json.dumps(plan), encoding="utf-8")
        catalog_path = data_root / "catalog" / "human-short-read-diversity.json"
        catalog_path.parent.mkdir()
        catalog = {
            "schema_version": "bijux.data_catalog.v1",
            "id": "human-short-read-diversity",
            "purpose": "real human short-read diversity corpus",
            "source": "European Nucleotide Archive",
            "source_record": "study://data/evidence/acquisition-plans/original.json",
            "source_record_sha256": "0" * 64,
            "retrieved_at_utc": "2026-08-03T00:00:00+00:00",
            "access_classification": "public",
            "redistribution_classification": "accession-referenced",
            "known_limitations": ["not population-representative"],
            "files": [
                {
                    "path": str(raw),
                    "sha256": hashlib.sha256(raw_bytes).hexdigest(),
                    "size_bytes": len(raw_bytes),
                    "accession": "ERR000001",
                    "source_url": source_url,
                    "source_md5": source_md5,
                }
            ],
        }
        catalog_path.write_text(json.dumps(catalog, sort_keys=True) + "\n", encoding="utf-8")
        return data_root, plan_path, catalog_path, raw_bytes

    def test_enrichment_preserves_raw_identity_and_recovery_evidence(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, plan_path, catalog_path, raw_bytes = self.records(Path(directory))
            before_bytes = catalog_path.read_bytes()
            before_sha256 = hashlib.sha256(before_bytes).hexdigest()

            report_path = enrichment.enrich(
                plan_path,
                catalog_path,
                before_sha256,
                data_root / "evidence" / "catalog-enrichments",
            )

            catalog = json.loads(catalog_path.read_text())
            report = json.loads(report_path.read_text())
            evidence = report_path.parent
            self.assertEqual((evidence / "catalog.before.json").read_bytes(), before_bytes)
            self.assertEqual(
                (evidence / "catalog.after.json").read_bytes(), catalog_path.read_bytes()
            )
            self.assertEqual(report["before_sha256"], before_sha256)
            self.assertEqual(report["after_sha256"], acquisition.sha256_file(catalog_path))
            self.assertEqual(report["plan_sha256"], acquisition.sha256_file(plan_path))
            self.assertEqual(catalog["samples"][0]["sample_accession"], "SAMEA1")
            self.assertEqual(catalog["files"][0]["experiment_accession"], "ERX1")
            self.assertEqual(catalog["files"][0]["era"], "ancient")
            self.assertEqual(
                catalog["source_record"],
                "study://data/evidence/acquisition-plans/enriched.json",
            )
            raw = Path(catalog["files"][0]["path"])
            self.assertEqual(raw.read_bytes(), raw_bytes)
            self.assertEqual(
                hashlib.sha256(raw.read_bytes()).hexdigest(),
                catalog["files"][0]["sha256"],
            )

    def test_enrichment_requires_the_reviewed_catalog_hash(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, plan_path, catalog_path, _ = self.records(Path(directory))

            with self.assertRaisesRegex(enrichment.EnrichmentError, "reviewed input"):
                enrichment.enrich(
                    plan_path,
                    catalog_path,
                    "0" * 64,
                    data_root / "evidence" / "catalog-enrichments",
                )

            self.assertFalse((data_root / "evidence" / "catalog-enrichments").exists())

    def test_enrichment_rejects_raw_source_drift(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, plan_path, catalog_path, _ = self.records(Path(directory))
            plan = json.loads(plan_path.read_text())
            plan["samples"][0]["files"][0]["source_url"] = "https://archive.example/drift.fastq.gz"
            plan_path.write_text(json.dumps(plan), encoding="utf-8")

            with self.assertRaisesRegex(enrichment.EnrichmentError, "raw source identity drifted"):
                enrichment.enrich(
                    plan_path,
                    catalog_path,
                    acquisition.sha256_file(catalog_path),
                    data_root / "evidence" / "catalog-enrichments",
                )

    def test_enrichment_rehashes_governed_raw_files(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, plan_path, catalog_path, _ = self.records(Path(directory))
            catalog = json.loads(catalog_path.read_text())
            Path(catalog["files"][0]["path"]).write_bytes(b"changed-raw-bytes")

            with self.assertRaisesRegex(enrichment.EnrichmentError, "drifted"):
                enrichment.enrich(
                    plan_path,
                    catalog_path,
                    acquisition.sha256_file(catalog_path),
                    data_root / "evidence" / "catalog-enrichments",
                )

            self.assertFalse((data_root / "evidence" / "catalog-enrichments").exists())


if __name__ == "__main__":
    unittest.main()
