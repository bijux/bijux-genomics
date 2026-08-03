#!/usr/bin/env python3
"""Materialize and admit an immutable real FASTQ corpus acquisition plan."""

from __future__ import annotations

import argparse
import concurrent.futures
import datetime as dt
import gzip
import hashlib
import http.client
import json
import os
import re
import sys
import urllib.error
import urllib.request
from pathlib import Path
from typing import Any, BinaryIO, Callable

PLAN_SCHEMA = "bijux.benchmark.corpus_acquisition.v1"
CATALOG_SCHEMA = "bijux.data_catalog.v1"
CATALOG_ID = re.compile(r"[a-z0-9][a-z0-9-]*")
MD5 = re.compile(r"[0-9a-f]{32}")
OpenUrl = Callable[[str, int], BinaryIO]


class MaterializationError(RuntimeError):
    """Raised when source bytes cannot be admitted as the declared corpus."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_plan(path: Path) -> dict[str, Any]:
    try:
        plan = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise MaterializationError(f"cannot read acquisition plan {path}: {exc}") from exc
    if not isinstance(plan, dict) or plan.get("schema_version") != PLAN_SCHEMA:
        raise MaterializationError("acquisition plan schema is unsupported")
    portfolio_id = plan.get("portfolio_id")
    if not isinstance(portfolio_id, str) or CATALOG_ID.fullmatch(portfolio_id) is None:
        raise MaterializationError("acquisition plan has no valid portfolio identity")
    samples = plan.get("samples")
    if not isinstance(samples, list) or len(samples) != plan.get("sample_count"):
        raise MaterializationError("acquisition plan sample count disagrees with its records")
    accessions: set[str] = set()
    targets: set[tuple[str, str]] = set()
    file_count = 0
    total_size = 0
    for sample in samples:
        if not isinstance(sample, dict):
            raise MaterializationError("acquisition plan sample is not an object")
        accession = sample.get("accession")
        layout = sample.get("layout")
        files = sample.get("files")
        if (
            not isinstance(accession, str)
            or not accession
            or accession in accessions
            or layout not in {"se", "pe"}
            or not isinstance(files, list)
        ):
            raise MaterializationError("acquisition plan sample identity is invalid")
        accessions.add(accession)
        if len(files) != (1 if layout == "se" else 2):
            raise MaterializationError(f"acquisition file count disagrees for {accession}")
        for record in files:
            if not isinstance(record, dict):
                raise MaterializationError(f"acquisition file is invalid for {accession}")
            name = record.get("name")
            url = record.get("source_url")
            digest = record.get("source_md5")
            size = record.get("size_bytes")
            if (
                not isinstance(name, str)
                or Path(name).name != name
                or not name.endswith(".fastq.gz")
                or not isinstance(url, str)
                or not url.startswith("https://")
                or not isinstance(digest, str)
                or MD5.fullmatch(digest) is None
                or not isinstance(size, int)
                or isinstance(size, bool)
                or size <= 0
            ):
                raise MaterializationError(f"acquisition file identity is invalid for {accession}")
            identity = (accession, name)
            if identity in targets:
                raise MaterializationError(f"acquisition target repeats: {accession}/{name}")
            targets.add(identity)
            file_count += 1
            total_size += size
    if file_count != plan.get("file_count") or total_size != plan.get("total_size_bytes"):
        raise MaterializationError("acquisition plan aggregate file identity disagrees")
    return plan


def open_url(url: str, timeout: int) -> BinaryIO:
    request = urllib.request.Request(
        url,
        headers={"Accept": "application/octet-stream", "User-Agent": "bijux-genomics/1"},
    )
    return urllib.request.urlopen(request, timeout=timeout)


def observed_file(path: Path) -> tuple[int, str, str]:
    md5 = hashlib.md5(usedforsecurity=False)
    sha256 = hashlib.sha256()
    size = 0
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            size += len(chunk)
            md5.update(chunk)
            sha256.update(chunk)
    return size, md5.hexdigest(), sha256.hexdigest()


def verify_source_identity(path: Path, record: dict[str, Any]) -> str:
    size, md5, sha256 = observed_file(path)
    if size != record["size_bytes"] or md5 != record["source_md5"]:
        raise MaterializationError(
            f"source identity mismatch for {path}: observed {size} bytes and MD5 {md5}"
        )
    return sha256


def download_file(
    record: dict[str, Any],
    target: Path,
    retries: int,
    timeout: int,
    opener: OpenUrl,
) -> tuple[str, str]:
    if target.is_symlink():
        raise MaterializationError(f"corpus target must not be a symlink: {target}")
    if target.exists():
        return "verified_existing", verify_source_identity(target, record)
    target.parent.mkdir(parents=True, exist_ok=True)
    partial = target.with_name(f".{target.name}.partial-{os.getpid()}")
    if partial.exists() or partial.is_symlink():
        raise MaterializationError(f"materialization partial path already exists: {partial}")
    last_error: Exception | None = None
    for _ in range(retries + 1):
        md5 = hashlib.md5(usedforsecurity=False)
        sha256 = hashlib.sha256()
        size = 0
        try:
            with opener(str(record["source_url"]), timeout) as source, partial.open("xb") as sink:
                while True:
                    chunk = source.read(1024 * 1024)
                    if not chunk:
                        break
                    sink.write(chunk)
                    size += len(chunk)
                    md5.update(chunk)
                    sha256.update(chunk)
                sink.flush()
                os.fsync(sink.fileno())
            if size != record["size_bytes"] or md5.hexdigest() != record["source_md5"]:
                raise MaterializationError(
                    f"download identity mismatch for {target}: {size} bytes, MD5 {md5.hexdigest()}"
                )
            partial.replace(target)
            return "downloaded", sha256.hexdigest()
        except (
            MaterializationError,
            OSError,
            http.client.HTTPException,
            urllib.error.URLError,
        ) as exc:
            last_error = exc
            if partial.is_file():
                partial.unlink()
    raise MaterializationError(f"download failed for {target}: {last_error}")


def validate_fastq(path: Path) -> tuple[int, int]:
    reads = 0
    bases = 0
    try:
        with gzip.open(path, "rb") as stream:
            while True:
                header = stream.readline()
                if not header:
                    break
                sequence = stream.readline()
                separator = stream.readline()
                quality = stream.readline()
                if (
                    not sequence
                    or not separator
                    or not quality
                    or not header.startswith(b"@")
                    or not separator.startswith(b"+")
                    or len(sequence.rstrip(b"\r\n")) != len(quality.rstrip(b"\r\n"))
                ):
                    raise MaterializationError(f"invalid FASTQ record {reads + 1} in {path}")
                reads += 1
                bases += len(sequence.rstrip(b"\r\n"))
    except (OSError, EOFError) as exc:
        raise MaterializationError(f"gzip validation failed for {path}: {exc}") from exc
    if reads == 0:
        raise MaterializationError(f"FASTQ contains no records: {path}")
    return reads, bases


def materialize(
    plan_path: Path,
    data_root: Path,
    jobs: int,
    retries: int,
    timeout: int,
    opener: OpenUrl = open_url,
) -> dict[str, Any]:
    if jobs < 1 or retries < 0 or timeout < 1:
        raise MaterializationError("jobs, retries, and timeout are invalid")
    plan = load_plan(plan_path)
    data_root = data_root.resolve()
    try:
        plan_identity = plan_path.resolve().relative_to(data_root)
    except ValueError as exc:
        raise MaterializationError(
            "acquisition plan must remain under the governed data root"
        ) from exc
    corpus_root = data_root / "corpora" / str(plan["portfolio_id"])
    catalog_path = data_root / "catalog" / f"{plan['portfolio_id']}.json"
    if catalog_path.exists() or catalog_path.is_symlink():
        raise MaterializationError(f"refusing to replace admitted catalog: {catalog_path}")
    work = []
    for sample in plan["samples"]:
        for record in sample["files"]:
            target = corpus_root / "raw" / str(sample["accession"]) / str(record["name"])
            work.append((sample, record, target))
    downloaded: dict[Path, tuple[str, str]] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = {
            pool.submit(download_file, record, target, retries, timeout, opener): target
            for _, record, target in work
        }
        for future in concurrent.futures.as_completed(futures):
            target = futures[future]
            downloaded[target] = future.result()
    validations: dict[Path, tuple[int, int]] = {}
    with concurrent.futures.ThreadPoolExecutor(max_workers=jobs) as pool:
        futures = {pool.submit(validate_fastq, target): target for _, _, target in work}
        for future in concurrent.futures.as_completed(futures):
            target = futures[future]
            validations[target] = future.result()
    catalog_files = []
    catalog_samples = []
    for sample, record, target in work:
        status, sha256 = downloaded[target]
        reads, bases = validations[target]
        target.chmod(0o444)
        catalog_files.append(
            {
                "path": str(target),
                "sha256": sha256,
                "size_bytes": record["size_bytes"],
                "kind": "raw-fastq",
                "accession": sample["accession"],
                "study_accession": sample["study_accession"],
                "sample_accession": sample["sample_accession"],
                "experiment_accession": sample["experiment_accession"],
                "era": sample["era"],
                "layout": sample["layout"],
                "mate": record["mate"],
                "read_count": reads,
                "base_count": bases,
                "source_url": record["source_url"],
                "source_md5": record["source_md5"],
                "materialization_status": status,
            }
        )
    for sample in plan["samples"]:
        catalog_samples.append(
            {
                key: sample[key]
                for key in (
                    "accession",
                    "study_accession",
                    "sample_accession",
                    "experiment_accession",
                    "era",
                    "layout",
                    "size_band",
                    "selection_reason",
                    "scientific_name",
                    "library_source",
                    "library_strategy",
                    "library_selection",
                    "instrument_platform",
                    "instrument_model",
                    "archive_base_count",
                    "archive_read_count",
                    "first_public",
                    "last_updated",
                )
            }
        )
    catalog = {
        "schema_version": CATALOG_SCHEMA,
        "id": plan["portfolio_id"],
        "purpose": "real human short-read diversity corpus for comparative FASTQ campaigns",
        "source": "European Nucleotide Archive",
        "source_record": f"study://data/{plan_identity.as_posix()}",
        "source_record_sha256": sha256_file(plan_path),
        "retrieved_at_utc": plan["retrieved_at_utc"],
        "admitted_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "access_classification": "public",
        "redistribution_classification": "accession-referenced",
        "access_terms": (
            "ENA public data access terms apply; publication identifies source accessions."
        ),
        "scientific_name": plan["species"],
        "samples": catalog_samples,
        "files": catalog_files,
        "known_limitations": [
            "The cohort is curated for workload diversity and is not population-representative.",
            "Public archive metadata and sequencing context differ across studies and eras.",
            (
                "No independent sequence truth is available; this corpus supports "
                "observational and robustness comparisons only."
            ),
        ],
    }
    catalog_path.parent.mkdir(parents=True, exist_ok=True)
    partial_catalog = catalog_path.with_suffix(".json.partial")
    if partial_catalog.exists() or partial_catalog.is_symlink():
        raise MaterializationError(f"catalog partial path already exists: {partial_catalog}")
    partial_catalog.write_text(json.dumps(catalog, indent=2, sort_keys=True) + "\n")
    partial_catalog.replace(catalog_path)
    return catalog


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--plan", type=Path, required=True)
    value.add_argument("--data-root", type=Path, required=True)
    value.add_argument("--jobs", type=int, default=4)
    value.add_argument("--retries", type=int, default=2)
    value.add_argument("--timeout", type=int, default=120)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        catalog = materialize(
            args.plan, args.data_root, args.jobs, args.retries, args.timeout
        )
        print(
            f"admitted {catalog['id']} with {len(catalog['files'])} verified FASTQ files"
        )
        return 0
    except MaterializationError as exc:
        print(f"corpus materialization failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
