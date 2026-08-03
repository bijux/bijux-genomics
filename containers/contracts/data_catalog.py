"""Build a verified, path-stable index of governed scientific data records."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import re
import sys
from collections import Counter, defaultdict
from pathlib import Path
from typing import Any

CATALOG_SCHEMA = "bijux.data_catalog.v1"
INDEX_SCHEMA = "bijux.data_index.v1"
SHA256 = re.compile(r"[0-9a-f]{64}")
IDENTIFIER = re.compile(r"[a-z0-9][a-z0-9._-]*")
PATH_CONTRACTS = (
    ("path", "sha256", "size_bytes"),
    ("governed_path", "governed_sha256", "governed_size_bytes"),
    ("derived_path", "derived_sha256", "derived_size_bytes"),
)
TEMPORAL_FIELDS = (
    "retrieved_at_utc",
    "admitted_at_utc",
    "created_at_utc",
    "retrieved_from_existing_lunarc_holdings_at_utc",
)


class DataCatalogError(RuntimeError):
    """Raised when governed data cannot be identified without ambiguity."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise DataCatalogError(f"cannot read data catalog {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise DataCatalogError(f"data catalog must be an object: {path}")
    return value


def write_json(path: Path, value: dict[str, Any]) -> None:
    path.parent.mkdir(parents=True, exist_ok=True)
    partial = path.with_suffix(path.suffix + ".partial")
    partial.write_text(json.dumps(value, indent=2, sort_keys=True) + "\n")
    partial.replace(path)


def stable_data_path(path: Path, data_root: Path) -> str:
    try:
        relative = path.resolve().relative_to(data_root.resolve())
    except ValueError as exc:
        raise DataCatalogError(f"governed data path escapes {data_root}: {path}") from exc
    return f"study://data/{relative.as_posix()}"


def validate_timestamp(value: str, label: str) -> str:
    try:
        parsed = dt.datetime.fromisoformat(value.replace("Z", "+00:00"))
    except ValueError as exc:
        raise DataCatalogError(f"{label} must be an ISO-8601 timestamp") from exc
    if parsed.tzinfo is None or parsed.utcoffset() != dt.timedelta(0):
        raise DataCatalogError(f"{label} must use UTC")
    return value


def temporal_provenance(record: dict[str, Any]) -> tuple[str | None, str | None]:
    for field in TEMPORAL_FIELDS:
        value = record.get(field)
        if isinstance(value, str) and value:
            return field, validate_timestamp(value, field)
    acquisition = record.get("acquisition")
    if isinstance(acquisition, dict):
        for field in TEMPORAL_FIELDS:
            value = acquisition.get(field)
            if isinstance(value, str) and value:
                return f"acquisition.{field}", validate_timestamp(value, field)
    return None, None


def source_identity(record: dict[str, Any]) -> list[str]:
    identities = []
    for field in (
        "source",
        "source_url",
        "source_record",
        "source_page",
        "source_parent_id",
        "reference_parent_id",
    ):
        value = record.get(field)
        if isinstance(value, str) and value:
            identities.append(f"{field}:{value}")
    parents = record.get("parents")
    if isinstance(parents, list):
        identities.extend(
            f"parent:{value}" for value in parents if isinstance(value, str)
        )
    return identities


def basename_index(data_root: Path) -> dict[str, list[Path]]:
    index: dict[str, list[Path]] = defaultdict(list)
    for path in data_root.rglob("*"):
        if path.is_file() and "catalog" not in path.relative_to(data_root).parts[:1]:
            index[path.name].append(path)
    return index


def file_contract(
    value: dict[str, Any],
    label: str,
    data_root: Path,
    files_by_name: dict[str, list[Path]],
) -> dict[str, Any] | None:
    path: Path | None = None
    digest: Any = None
    size: Any = None
    source_fields: tuple[str, str, str] | None = None
    if all(key in value for key in ("name", "sha256", "size_bytes")) and not any(
        path_key in value for path_key, _, _ in PATH_CONTRACTS
    ):
        matches = files_by_name.get(str(value["name"]), [])
        if len(matches) != 1:
            raise DataCatalogError(
                f"bare data filename is not uniquely resolvable at {label}: {value['name']}"
            )
        path = matches[0]
        digest = value["sha256"]
        size = value["size_bytes"]
        source_fields = ("name", "sha256", "size_bytes")
    for path_key, digest_key, size_key in PATH_CONTRACTS:
        if path_key in value:
            legacy_parent = label.startswith("catalog.parents.")
            if path_key not in value or digest_key not in value:
                raise DataCatalogError(f"incomplete file identity at {label}")
            if size_key not in value and not legacy_parent:
                raise DataCatalogError(f"incomplete file identity at {label}")
            path = Path(str(value[path_key]))
            digest = value[digest_key]
            size = value.get(size_key)
            source_fields = tuple(
                key for key in (path_key, digest_key, size_key) if key in value
            )
            break
    if path is None:
        return None
    if not path.is_absolute():
        raise DataCatalogError(f"governed file path must be absolute at {label}: {path}")
    if size is None and label.startswith("catalog.parents."):
        size = path.stat().st_size if path.is_file() else None
    if not isinstance(size, int) or isinstance(size, bool) or size < 0:
        raise DataCatalogError(f"governed file size is invalid at {label}")
    if not isinstance(digest, str) or SHA256.fullmatch(digest) is None:
        raise DataCatalogError(f"governed file digest is invalid at {label}")
    if not path.is_file():
        raise DataCatalogError(f"governed data file is absent at {label}: {path}")
    if path.stat().st_size != size:
        raise DataCatalogError(f"governed data size drifted at {label}: {path}")
    observed = sha256_file(path)
    if observed != digest:
        raise DataCatalogError(f"governed data digest drifted at {label}: {path}")
    role = value.get("kind") or value.get("role") or label
    return {
        "path": stable_data_path(path, data_root),
        "sha256": digest,
        "size_bytes": size,
        "role": str(role),
        "legacy_fields": list(source_fields or ()),
    }


def governed_files(
    record: dict[str, Any], data_root: Path, files_by_name: dict[str, list[Path]]
) -> list[dict[str, Any]]:
    found: dict[str, dict[str, Any]] = {}

    def visit(value: Any, label: str) -> None:
        if isinstance(value, dict):
            candidate = file_contract(value, label, data_root, files_by_name)
            if candidate is not None:
                path = str(candidate["path"])
                previous = found.get(path)
                if previous is not None and (
                    previous["sha256"] != candidate["sha256"]
                    or previous["size_bytes"] != candidate["size_bytes"]
                ):
                    raise DataCatalogError(f"conflicting data identities for {path}")
                found[path] = candidate
            for key, child in value.items():
                if key.startswith("source_"):
                    continue
                visit(child, f"{label}.{key}" if label else key)
        elif isinstance(value, list):
            for index, child in enumerate(value):
                visit(child, f"{label}.{index}")

    visit(record, "catalog")
    return [found[path] for path in sorted(found)]


def normalize_record(
    catalog_path: Path,
    record: dict[str, Any],
    data_root: Path,
    files_by_name: dict[str, list[Path]],
) -> dict[str, Any]:
    record_id = record.get("id")
    if (
        record.get("schema_version") != CATALOG_SCHEMA
        or not isinstance(record_id, str)
        or IDENTIFIER.fullmatch(record_id) is None
        or catalog_path.stem != record_id
    ):
        raise DataCatalogError(f"catalog identity is invalid: {catalog_path}")
    for field in ("purpose", "access_classification", "redistribution_classification"):
        if not isinstance(record.get(field), str) or not record[field]:
            raise DataCatalogError(f"{record_id} lacks required {field}")
    limitations = record.get("known_limitations")
    if (
        not isinstance(limitations, list)
        or not limitations
        or any(not isinstance(item, str) or not item for item in limitations)
    ):
        raise DataCatalogError(f"{record_id} lacks explicit known limitations")
    files = governed_files(record, data_root, files_by_name)
    temporal_field, timestamp = temporal_provenance(record)
    sources = source_identity(record)
    missing = []
    if not files:
        missing.append("governed_files")
    if not sources:
        missing.append("source_identity")
    if timestamp is None:
        missing.append("temporal_provenance")
    return {
        "id": record_id,
        "catalog_path": stable_data_path(catalog_path, data_root),
        "catalog_sha256": sha256_file(catalog_path),
        "purpose": record["purpose"],
        "source_identities": sources,
        "access_classification": record["access_classification"],
        "redistribution_classification": record["redistribution_classification"],
        "scientific_name": record.get("scientific_name"),
        "reference_assembly": record.get("reference_assembly"),
        "temporal_provenance": {"field": temporal_field, "timestamp": timestamp},
        "files": files,
        "known_limitations": limitations,
        "integrity_status": "verified" if files else "unverified",
        "benchmark_eligibility": "eligible" if not missing else "incomplete",
        "missing_provenance": missing,
    }


def build_index(catalog_root: Path, data_root: Path) -> dict[str, Any]:
    catalog_root = catalog_root.resolve()
    data_root = data_root.resolve()
    try:
        catalog_root.relative_to(data_root)
    except ValueError as exc:
        raise DataCatalogError(
            "catalog root must remain under the governed data root"
        ) from exc
    files_by_name = basename_index(data_root)
    records = [
        normalize_record(path, load_json(path), data_root, files_by_name)
        for path in sorted(catalog_root.glob("*.json"))
    ]
    if not records:
        raise DataCatalogError(f"no data catalogs found under {catalog_root}")
    ids = [record["id"] for record in records]
    if len(ids) != len(set(ids)):
        raise DataCatalogError("data catalog IDs are not unique")
    eligibility = Counter(record["benchmark_eligibility"] for record in records)
    return {
        "schema_version": INDEX_SCHEMA,
        "created_at_utc": dt.datetime.now(dt.timezone.utc).isoformat(),
        "data_root": "study://data",
        "summary": {
            "record_count": len(records),
            "file_count": sum(len(record["files"]) for record in records),
            "size_bytes": sum(
                int(file["size_bytes"])
                for record in records
                for file in record["files"]
            ),
            "integrity_verified_count": sum(
                record["integrity_status"] == "verified" for record in records
            ),
            "benchmark_eligibility": dict(sorted(eligibility.items())),
        },
        "records": records,
    }


def render_report(index: dict[str, Any]) -> str:
    summary = index["summary"]
    lines = [
        "# Governed scientific data index",
        "",
        "This report proves file identity and exposes provenance gaps. It does not",
        "turn incomplete metadata into scientific acceptance.",
        "",
        f"- Catalog records: **{summary['record_count']}**",
        f"- Verified governed files: **{summary['file_count']}**",
        f"- Verified bytes: **{summary['size_bytes']}**",
        "- Benchmark eligibility: "
        + ", ".join(
            f"**{status} {count}**"
            for status, count in summary["benchmark_eligibility"].items()
        ),
        "",
        "| Data ID | Files | Integrity | Benchmark use | Missing provenance |",
        "|---|---:|---|---|---|",
    ]
    for record in index["records"]:
        missing = ", ".join(record["missing_provenance"]) or "—"
        lines.append(
            f"| `{record['id']}` | {len(record['files'])} | "
            f"{record['integrity_status']} | {record['benchmark_eligibility']} | "
            f"{missing} |"
        )
    return "\n".join(lines) + "\n"


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--catalog-root", type=Path, required=True)
    root.add_argument("--data-root", type=Path, required=True)
    root.add_argument("--output", type=Path, required=True)
    root.add_argument("--report", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        if args.output.exists() or args.report.exists():
            raise DataCatalogError("refusing to replace an existing governed data index")
        index = build_index(args.catalog_root, args.data_root)
        write_json(args.output, index)
        args.report.parent.mkdir(parents=True, exist_ok=True)
        args.report.write_text(render_report(index), encoding="utf-8")
        print(
            f"indexed {index['summary']['record_count']} data records and "
            f"{index['summary']['file_count']} governed files"
        )
        return 0
    except DataCatalogError as exc:
        print(f"governed data indexing failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
