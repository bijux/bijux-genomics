from __future__ import annotations

import importlib.util
import json
import tempfile
import unittest
from pathlib import Path

SCRIPT = Path(__file__).parents[1] / "data_catalog.py"
SPEC = importlib.util.spec_from_file_location("container_data_catalog", SCRIPT)
assert SPEC is not None and SPEC.loader is not None
catalog = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(catalog)


class DataCatalogTests(unittest.TestCase):
    def make_record(self, root: Path) -> tuple[Path, Path, Path]:
        data_root = root / "data"
        catalog_root = data_root / "catalog"
        corpus = data_root / "corpora" / "horse" / "sample.bam"
        corpus.parent.mkdir(parents=True)
        catalog_root.mkdir()
        corpus.write_bytes(b"governed-bam")
        record = catalog_root / "ancient-horse-bam.json"
        record.write_text(
            json.dumps(
                {
                    "schema_version": catalog.CATALOG_SCHEMA,
                    "id": "ancient-horse-bam",
                    "purpose": "real ancient horse alignment",
                    "source": "ENA",
                    "access_classification": "public",
                    "redistribution_classification": "accession-referenced",
                    "retrieved_at_utc": "2026-08-03T00:00:00Z",
                    "scientific_name": "Equus caballus",
                    "files": [
                        {
                            "path": str(corpus),
                            "sha256": catalog.sha256_file(corpus),
                            "size_bytes": corpus.stat().st_size,
                            "kind": "coordinate-sorted-bam",
                        }
                    ],
                    "known_limitations": [
                        "one sample does not represent a population"
                    ],
                }
            ),
            encoding="utf-8",
        )
        return data_root, catalog_root, record

    def test_index_verifies_files_and_publishes_stable_paths(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, catalog_root, _ = self.make_record(Path(directory))
            index = catalog.build_index(catalog_root, data_root)

        self.assertEqual(index["summary"]["record_count"], 1)
        self.assertEqual(index["summary"]["file_count"], 1)
        self.assertEqual(index["summary"]["benchmark_eligibility"], {"eligible": 1})
        self.assertEqual(
            index["records"][0]["files"][0]["path"],
            "study://data/corpora/horse/sample.bam",
        )

    def test_missing_temporal_provenance_is_visible_not_fabricated(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, catalog_root, record_path = self.make_record(Path(directory))
            record = json.loads(record_path.read_text())
            record.pop("retrieved_at_utc")
            record_path.write_text(json.dumps(record), encoding="utf-8")
            index = catalog.build_index(catalog_root, data_root)

        self.assertEqual(
            index["summary"]["benchmark_eligibility"], {"incomplete": 1}
        )
        self.assertEqual(
            index["records"][0]["missing_provenance"], ["temporal_provenance"]
        )

    def test_digest_drift_fails_indexing(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, catalog_root, record_path = self.make_record(Path(directory))
            record = json.loads(record_path.read_text())
            record["files"][0]["sha256"] = "0" * 64
            record_path.write_text(json.dumps(record), encoding="utf-8")

            with self.assertRaisesRegex(catalog.DataCatalogError, "digest drifted"):
                catalog.build_index(catalog_root, data_root)

    def test_bare_filename_must_resolve_uniquely(self) -> None:
        with tempfile.TemporaryDirectory() as directory:
            data_root, catalog_root, record_path = self.make_record(Path(directory))
            first = data_root / "corpora" / "horse" / "sample.bam"
            duplicate = data_root / "references" / "sample.bam"
            duplicate.parent.mkdir()
            duplicate.write_bytes(first.read_bytes())
            record = json.loads(record_path.read_text())
            record["files"] = [
                {
                    "name": "sample.bam",
                    "sha256": catalog.sha256_file(first),
                    "size_bytes": first.stat().st_size,
                }
            ]
            record_path.write_text(json.dumps(record), encoding="utf-8")

            with self.assertRaisesRegex(catalog.DataCatalogError, "not uniquely"):
                catalog.build_index(catalog_root, data_root)


if __name__ == "__main__":
    unittest.main()
