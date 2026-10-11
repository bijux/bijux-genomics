#!/usr/bin/env python3
"""Enrich an admitted corpus catalog from a newer immutable acquisition plan."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from pathlib import Path
from typing import Any

import corpus_materialization as materialization

SHA256 = re.compile(r"[0-9a-f]{64}")


class EnrichmentError(RuntimeError):
    """Raised when catalog context cannot be upgraded without changing raw identity."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise EnrichmentError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise EnrichmentError(f"JSON root must be an object: {path}")
    return value


def verify_catalog_files(catalog: dict[str, Any]) -> None:
    files = catalog.get("files")
    if not isinstance(files, list) or not files:
        raise EnrichmentError("catalog contains no governed files")
    for record in files:
        if not isinstance(record, dict):
            raise EnrichmentError("catalog file is invalid")
        path = Path(str(record.get("path", "")))
        expected_size = record.get("size_bytes")
        expected_sha256 = record.get("sha256")
        if (
            not path.is_absolute()
            or path.is_symlink()
            or not path.is_file()
            or not isinstance(expected_size, int)
            or isinstance(expected_size, bool)
            or expected_size <= 0
            or not isinstance(expected_sha256, str)
            or SHA256.fullmatch(expected_sha256) is None
        ):
            raise EnrichmentError(f"catalog file identity is invalid: {path}")
        if path.stat().st_size != expected_size:
            raise EnrichmentError(f"catalog file size drifted: {path}")
        if materialization.sha256_file(path) != expected_sha256:
            raise EnrichmentError(f"catalog file digest drifted: {path}")


def enriched_catalog(
    plan: dict[str, Any], catalog: dict[str, Any]
) -> dict[str, Any]:
    if catalog.get("id") != plan.get("portfolio_id"):
        raise EnrichmentError("catalog and acquisition plan identities disagree")
    catalog_files = catalog.get("files")
    if not isinstance(catalog_files, list):
        raise EnrichmentError("catalog files are invalid")
    plan_files: dict[tuple[str, str], tuple[dict[str, Any], dict[str, Any]]] = {}
    for sample in plan["samples"]:
        context = materialization.sample_context(sample)
        for record in sample["files"]:
            identity = (str(sample["accession"]), str(record["name"]))
            if identity in plan_files:
                raise EnrichmentError(
                    f"acquisition plan repeats a file identity: {identity}"
                )
            plan_files[identity] = (context, record)
    if len(plan_files) != len(catalog_files):
        raise EnrichmentError("catalog and acquisition plan file counts disagree")
    enriched_files = []
    observed: set[tuple[str, str]] = set()
    for catalog_file in catalog_files:
        if not isinstance(catalog_file, dict):
            raise EnrichmentError("catalog file is invalid")
        identity = (
            str(catalog_file.get("accession", "")),
            Path(str(catalog_file.get("path", ""))).name,
        )
        if identity in observed or identity not in plan_files:
            raise EnrichmentError(f"catalog file does not resolve to the plan: {identity}")
        observed.add(identity)
        context, source = plan_files[identity]
        if (
            catalog_file.get("source_md5") != source.get("source_md5")
            or catalog_file.get("size_bytes") != source.get("size_bytes")
            or catalog_file.get("source_url") != source.get("source_url")
        ):
            raise EnrichmentError(f"raw source identity drifted for {identity}")
        updated = dict(catalog_file)
        updated.update(
            {
                "study_accession": context["study_accession"],
                "sample_accession": context["sample_accession"],
                "experiment_accession": context["experiment_accession"],
                "era": context["era"],
            }
        )
        enriched_files.append(updated)
    updated_catalog = dict(catalog)
    updated_catalog["files"] = enriched_files
    updated_catalog["samples"] = [
        materialization.sample_context(sample) for sample in plan["samples"]
    ]
    return updated_catalog


def enrich(
    plan_path: Path,
    catalog_path: Path,
    expected_catalog_sha256: str,
    evidence_root: Path,
) -> Path:
    if SHA256.fullmatch(expected_catalog_sha256) is None:
        raise EnrichmentError("expected catalog SHA-256 is invalid")
    plan = materialization.load_plan(plan_path)
    observed_catalog_sha256 = materialization.sha256_file(catalog_path)
    if observed_catalog_sha256 != expected_catalog_sha256:
        raise EnrichmentError("catalog SHA-256 does not match the reviewed input")
    catalog = load_json(catalog_path)
    verify_catalog_files(catalog)
    updated = enriched_catalog(plan, catalog)
    data_root = catalog_path.resolve().parents[1]
    try:
        plan_identity = plan_path.resolve().relative_to(data_root)
    except ValueError as exc:
        raise EnrichmentError("acquisition plan escapes the governed data root") from exc
    updated["source_record"] = f"study://data/{plan_identity.as_posix()}"
    updated["source_record_sha256"] = materialization.sha256_file(plan_path)
    payload = json.dumps(updated, indent=2, sort_keys=True) + "\n"
    updated_sha256 = hashlib.sha256(payload.encode()).hexdigest()
    evidence = evidence_root / f"{observed_catalog_sha256}-{updated_sha256}"
    if evidence.exists() or evidence.is_symlink():
        raise EnrichmentError(f"refusing to replace enrichment evidence: {evidence}")
    evidence.mkdir(parents=True, mode=0o750)
    before = evidence / "catalog.before.json"
    after = evidence / "catalog.after.json"
    report = evidence / "enrichment.json"
    before.write_bytes(catalog_path.read_bytes())
    after.write_text(payload, encoding="utf-8")
    report.write_text(
        json.dumps(
            {
                "schema_version": "bijux.benchmark.corpus_catalog_enrichment.v1",
                "catalog_id": updated["id"],
                "before_sha256": observed_catalog_sha256,
                "after_sha256": updated_sha256,
                "plan_sha256": materialization.sha256_file(plan_path),
                "sample_count": len(updated["samples"]),
                "file_count": len(updated["files"]),
                "recovery": "restore catalog.before.json only after verifying before_sha256",
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )
    temporary = catalog_path.with_suffix(".json.enrichment-partial")
    if temporary.exists() or temporary.is_symlink():
        raise EnrichmentError(f"catalog enrichment partial already exists: {temporary}")
    temporary.write_text(payload, encoding="utf-8")
    temporary.replace(catalog_path)
    return report


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--plan", type=Path, required=True)
    value.add_argument("--catalog", type=Path, required=True)
    value.add_argument("--expected-catalog-sha256", required=True)
    value.add_argument("--evidence-root", type=Path, required=True)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        print(
            enrich(
                args.plan,
                args.catalog,
                args.expected_catalog_sha256,
                args.evidence_root,
            )
        )
        return 0
    except (EnrichmentError, materialization.MaterializationError) as exc:
        print(f"corpus catalog enrichment failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
