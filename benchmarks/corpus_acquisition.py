#!/usr/bin/env python3
"""Resolve real ENA FASTQ corpus specifications into immutable download plans."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import urllib.parse
import urllib.request
from pathlib import Path
from typing import Any, Callable

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

PLAN_SCHEMA = "bijux.benchmark.corpus_acquisition.v1"
SPEC_SCHEMA = "bijux.corpus_spec.v1"
ENA_ENDPOINT = "https://www.ebi.ac.uk/ena/portal/api/filereport"
ENA_FIELDS = (
    "run_accession",
    "study_accession",
    "scientific_name",
    "library_layout",
    "instrument_platform",
    "fastq_ftp",
    "fastq_md5",
    "fastq_bytes",
    "first_public",
    "last_updated",
)
MD5 = re.compile(r"[0-9a-f]{32}")
ENA_LAYOUT = {"se": "SINGLE", "pe": "PAIRED"}


class AcquisitionError(RuntimeError):
    """Raised when public source metadata cannot identify exact corpus files."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_spec(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise AcquisitionError(f"cannot read corpus specification {path}: {exc}") from exc
    if not isinstance(value, dict) or value.get("schema_version") != SPEC_SCHEMA:
        raise AcquisitionError("corpus specification schema is unsupported")
    samples = value.get("samples")
    if not isinstance(samples, list) or not samples:
        raise AcquisitionError("corpus specification contains no samples")
    accessions = [sample.get("accession") for sample in samples if isinstance(sample, dict)]
    if len(accessions) != len(samples) or any(
        not isinstance(accession, str) or not accession for accession in accessions
    ):
        raise AcquisitionError("corpus specification has invalid sample accessions")
    if len(accessions) != len(set(accessions)):
        raise AcquisitionError("corpus specification repeats a sample accession")
    if value.get("target_total") != len(samples):
        raise AcquisitionError("corpus specification target_total disagrees with samples")
    return value


def ena_url(accession: str) -> str:
    query = urllib.parse.urlencode(
        {
            "accession": accession,
            "result": "read_run",
            "fields": ",".join(ENA_FIELDS),
            "format": "json",
        }
    )
    return f"{ENA_ENDPOINT}?{query}"


def fetch_ena_record(accession: str) -> dict[str, Any]:
    request = urllib.request.Request(
        ena_url(accession),
        headers={"Accept": "application/json", "User-Agent": "bijux-genomics/1"},
    )
    try:
        with urllib.request.urlopen(request, timeout=60) as response:
            payload = json.loads(response.read().decode("utf-8"))
    except (OSError, UnicodeDecodeError, json.JSONDecodeError) as exc:
        raise AcquisitionError(f"ENA metadata request failed for {accession}: {exc}") from exc
    if not isinstance(payload, list) or len(payload) != 1 or not isinstance(payload[0], dict):
        raise AcquisitionError(f"ENA returned an ambiguous run report for {accession}")
    return payload[0]


def split_field(record: dict[str, Any], key: str) -> list[str]:
    value = record.get(key)
    if not isinstance(value, str) or not value:
        raise AcquisitionError(f"ENA run report omits {key}")
    return value.split(";")


def normalized_files(record: dict[str, Any], layout: str) -> list[dict[str, Any]]:
    locations = split_field(record, "fastq_ftp")
    digests = split_field(record, "fastq_md5")
    byte_values = split_field(record, "fastq_bytes")
    if not (len(locations) == len(digests) == len(byte_values)):
        raise AcquisitionError("ENA FASTQ location, digest, and size counts disagree")
    expected = 1 if layout == "se" else 2
    if len(locations) != expected:
        raise AcquisitionError(
            f"ENA {layout} run exposes {len(locations)} FASTQ files instead of {expected}"
        )
    files = []
    for index, (location, digest, byte_value) in enumerate(
        zip(locations, digests, byte_values), start=1
    ):
        if MD5.fullmatch(digest) is None:
            raise AcquisitionError(f"ENA FASTQ MD5 is invalid: {digest}")
        try:
            size = int(byte_value)
        except ValueError as exc:
            raise AcquisitionError(f"ENA FASTQ byte count is invalid: {byte_value}") from exc
        if size <= 0:
            raise AcquisitionError("ENA FASTQ byte count must be positive")
        source_url = location if "://" in location else f"https://{location}"
        files.append(
            {
                "mate": index if layout == "pe" else None,
                "name": Path(urllib.parse.urlparse(source_url).path).name,
                "source_url": source_url,
                "source_md5": digest,
                "size_bytes": size,
            }
        )
    return files


def resolve_specification(
    spec_path: Path,
    fetch_record: Callable[[str], dict[str, Any]] = fetch_ena_record,
    retrieved_at: dt.datetime | None = None,
) -> dict[str, Any]:
    spec = load_spec(spec_path)
    timestamp = retrieved_at or dt.datetime.now(dt.timezone.utc)
    if timestamp.tzinfo is None or timestamp.utcoffset() != dt.timedelta(0):
        raise AcquisitionError("corpus retrieval timestamp must use UTC")
    resolved = []
    for sample in spec["samples"]:
        accession = str(sample["accession"])
        record = fetch_record(accession)
        if record.get("run_accession") != accession:
            raise AcquisitionError(f"ENA run identity disagrees for {accession}")
        if record.get("study_accession") != sample.get("study_accession"):
            raise AcquisitionError(f"ENA study identity disagrees for {accession}")
        layout = str(sample.get("layout", ""))
        if layout not in ENA_LAYOUT:
            raise AcquisitionError(f"corpus layout is unsupported for {accession}: {layout}")
        if record.get("library_layout") != ENA_LAYOUT[layout]:
            raise AcquisitionError(f"ENA library layout disagrees for {accession}")
        if record.get("scientific_name") != spec["species"]:
            raise AcquisitionError(f"ENA species identity disagrees for {accession}")
        resolved.append(
            {
                "accession": accession,
                "study_accession": sample["study_accession"],
                "era": sample["era"],
                "layout": layout,
                "size_band": sample["size_band"],
                "scientific_name": record.get("scientific_name"),
                "instrument_platform": record.get("instrument_platform"),
                "first_public": record.get("first_public"),
                "last_updated": record.get("last_updated"),
                "source_report_url": ena_url(accession),
                "files": normalized_files(record, layout),
            }
        )
    return {
        "schema_version": PLAN_SCHEMA,
        "corpus_id": spec["corpus_id"],
        "species": spec["species"],
        "retrieved_at_utc": timestamp.isoformat(),
        "source_authority": "European Nucleotide Archive",
        "source_spec": spec_path.as_posix(),
        "source_spec_sha256": sha256_file(spec_path),
        "sample_count": len(resolved),
        "file_count": sum(len(sample["files"]) for sample in resolved),
        "total_size_bytes": sum(
            file["size_bytes"] for sample in resolved for file in sample["files"]
        ),
        "samples": resolved,
    }


def parser() -> argparse.ArgumentParser:
    value = argparse.ArgumentParser(description=__doc__)
    value.add_argument("--spec", type=Path, required=True)
    value.add_argument("--output", type=Path, required=True)
    return value


def main() -> int:
    args = parser().parse_args()
    try:
        if args.output.exists() or args.output.is_symlink():
            raise AcquisitionError(f"refusing to replace acquisition plan: {args.output}")
        plan = resolve_specification(args.spec)
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_text(json.dumps(plan, indent=2, sort_keys=True) + "\n")
        print(
            f"resolved {plan['sample_count']} real samples, {plan['file_count']} files, "
            f"{plan['total_size_bytes']} bytes"
        )
        return 0
    except AcquisitionError as exc:
        print(f"corpus acquisition failed: {exc}")
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
