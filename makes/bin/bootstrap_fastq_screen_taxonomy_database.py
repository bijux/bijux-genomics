#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    artifact_bundle_size_bytes,
    default_screen_taxonomy_database_root,
    sha256_artifact_bundle,
    sha256_file,
)


REQUIRED_BACKEND_DIRS = ["kraken2", "krakenuniq", "centrifuge", "kaiju", "taxonomy"]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description=(
            "Validate and write governed lineage for the corpus-01 "
            "fastq.screen_taxonomy database bundle."
        )
    )
    parser.add_argument("--database-root", default="")
    parser.add_argument("--cache-root", default="")
    parser.add_argument("--database-catalog-id", default="taxonomy_reference")
    parser.add_argument("--database-artifact-id", default="taxonomy_db")
    parser.add_argument("--database-namespace", default="read_screening")
    parser.add_argument("--database-scope", default="read_screening")
    parser.add_argument(
        "--source-manifest",
        default="",
        help="Defaults to <database-root>/source/panel_manifest.json",
    )
    parser.add_argument(
        "--bootstrap-report",
        default="",
        help="Optional JSON report describing the build/bootstrap process.",
    )
    parser.add_argument(
        "--lineage-json",
        default="",
        help="Defaults to <database-root>/lineage.json",
    )
    return parser.parse_args()


def resolve_database_root(args: argparse.Namespace) -> Path:
    if args.database_root.strip():
        return Path(args.database_root).expanduser().resolve()
    if args.cache_root.strip():
        out_root = (
            Path(args.cache_root).expanduser().resolve()
            / "bijux-dna-results"
            / "corpus_01"
            / "fastq.screen_taxonomy"
            / "lunarc"
        )
        return default_screen_taxonomy_database_root(
            out_root,
            database_namespace=args.database_namespace,
            database_scope=args.database_scope,
            database_artifact_id=args.database_artifact_id,
        ).resolve()
    raise SystemExit("provide --database-root or --cache-root")


def resolve_source_manifest(database_root: Path, raw_value: str) -> Path:
    if raw_value.strip():
        return Path(raw_value).expanduser().resolve()
    return (database_root / "source" / "panel_manifest.json").resolve()


def resolve_lineage_json(database_root: Path, raw_value: str) -> Path:
    if raw_value.strip():
        return Path(raw_value).expanduser().resolve()
    return (database_root / "lineage.json").resolve()


def resolve_bootstrap_report(raw_value: str) -> Path | None:
    if not raw_value.strip():
        return None
    return Path(raw_value).expanduser().resolve()


def require_existing_dir(path: Path, label: str) -> None:
    if not path.is_dir():
        raise SystemExit(f"missing {label} directory: {path}")


def require_existing_file(path: Path, label: str) -> None:
    if not path.is_file():
        raise SystemExit(f"missing {label} file: {path}")


def build_lineage_payload(
    *,
    database_root: Path,
    source_manifest: Path,
    bootstrap_report: Path | None,
    database_catalog_id: str,
    database_artifact_id: str,
    database_namespace: str,
    database_scope: str,
) -> dict:
    require_existing_dir(database_root, "database-root")
    require_existing_file(source_manifest, "source manifest")

    panel_manifest = json.loads(source_manifest.read_text(encoding="utf-8"))
    panel_entries = panel_manifest.get("entries", [])
    if not isinstance(panel_entries, list) or not panel_entries:
        raise SystemExit(
            f"source manifest must contain a non-empty entries list: {source_manifest}"
        )

    backend_roots: list[dict] = []
    for backend in REQUIRED_BACKEND_DIRS:
        backend_root = database_root / backend
        require_existing_dir(backend_root, backend)
        backend_roots.append(
            {
                "backend": backend,
                "path": str(backend_root),
                "digest": sha256_artifact_bundle(backend_root),
                "size_bytes": artifact_bundle_size_bytes(backend_root),
            }
        )

    payload = {
        "schema_version": "bijux.fastq.screen_taxonomy.database_lineage.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "database_catalog_id": database_catalog_id,
        "database_artifact_id": database_artifact_id,
        "database_namespace": database_namespace,
        "database_scope": database_scope,
        "database_root": str(database_root),
        "database_digest": sha256_artifact_bundle(database_root),
        "database_size_bytes": artifact_bundle_size_bytes(database_root),
        "source_manifest": str(source_manifest),
        "source_manifest_digest": sha256_file(source_manifest),
        "source_entry_count": len(panel_entries),
        "source_entries": panel_entries,
        "backend_roots": backend_roots,
    }
    if bootstrap_report is not None:
        require_existing_file(bootstrap_report, "bootstrap report")
        payload["bootstrap_report"] = str(bootstrap_report)
        payload["bootstrap_report_digest"] = sha256_file(bootstrap_report)
    else:
        payload["bootstrap_report"] = None
        payload["bootstrap_report_digest"] = None
    return payload


def main() -> int:
    args = parse_args()
    database_root = resolve_database_root(args)
    source_manifest = resolve_source_manifest(database_root, args.source_manifest)
    bootstrap_report = resolve_bootstrap_report(args.bootstrap_report)
    lineage_json = resolve_lineage_json(database_root, args.lineage_json)
    payload = build_lineage_payload(
        database_root=database_root,
        source_manifest=source_manifest,
        bootstrap_report=bootstrap_report,
        database_catalog_id=args.database_catalog_id,
        database_artifact_id=args.database_artifact_id,
        database_namespace=args.database_namespace,
        database_scope=args.database_scope,
    )
    lineage_json.parent.mkdir(parents=True, exist_ok=True)
    lineage_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
