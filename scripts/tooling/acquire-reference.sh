#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

usage() {
  cat <<'USAGE'
Usage: scripts/tooling/acquire-reference.sh [--download] [--species <species-id>] [--build <build-id>] [--cache-root <dir>] [--verbose]

Acquires references from configs/runtime/reference_bank.toml.
Default mode is deterministic synthetic payload materialization for lock generation.
USAGE
}

download=0
verbose=0
species_filter=""
build_filter=""
cache_root="${ROOT_DIR}/artifacts/reference_store"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --download) download=1 ;;
    --species) species_filter="${2:-}"; shift ;;
    --build) build_filter="${2:-}"; shift ;;
    --cache-root) cache_root="${2:-}"; shift ;;
    --verbose) verbose=1 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

python3 - "$ROOT_DIR" "$cache_root" "$species_filter" "$build_filter" "$download" "$verbose" <<'PY'
from __future__ import annotations

import hashlib
import json
import os
from pathlib import Path
import sys
import urllib.request

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
cache_root = Path(sys.argv[2])
species_filter = sys.argv[3].strip()
build_filter = sys.argv[4].strip()
download = sys.argv[5] == "1"
verbose = sys.argv[6] == "1"

cfg = tomllib.loads((root / "configs/runtime/reference_bank.toml").read_text(encoding="utf-8"))
rows = cfg.get("reference", [])
if species_filter:
    rows = [row for row in rows if str(row.get("species_id", "")) == species_filter]
if build_filter:
    rows = [row for row in rows if str(row.get("build_id", "")) == build_filter]

acquire_log_root = root / "artifacts" / "containers" / "smoke" / "reference-acquire"
acquire_log_root.mkdir(parents=True, exist_ok=True)
lock_json = root / "configs/runtime/references/locks/lock.json"
lock_sha = root / "configs/runtime/references/locks/lock.json.sha256"


def now_utc() -> str:
    sde = os.environ.get("SOURCE_DATE_EPOCH")
    if sde:
        import datetime as dt

        return dt.datetime.fromtimestamp(int(sde), tz=dt.timezone.utc).isoformat().replace("+00:00", "Z")
    return "1970-01-01T00:00:00Z"


def sha256_path(path: Path) -> str:
    return hashlib.sha256(path.read_bytes()).hexdigest()


def run_index_tool(raw_fasta: Path, normalized_dir: Path, tool: str) -> dict[str, str]:
    manifest = {"tool": tool, "status": "synthetic", "output": ""}
    if tool == "samtools_faidx":
        out = normalized_dir / f"{raw_fasta.name}.fai"
        out.write_text(f"{raw_fasta.name}\t0\t0\t0\t0\n", encoding="utf-8")
        manifest["output"] = str(out)
    elif tool == "bwa_index":
        out = normalized_dir / f"{raw_fasta.name}.bwt"
        out.write_text("synthetic-bwa-index\n", encoding="utf-8")
        manifest["output"] = str(out)
    elif tool == "bowtie2_index":
        out = normalized_dir / f"{raw_fasta.name}.1.bt2"
        out.write_text("synthetic-bowtie2-index\n", encoding="utf-8")
        manifest["output"] = str(out)
    elif tool == "star_genome_generate":
        out = normalized_dir / "star" / "genomeParameters.txt"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text("synthetic-star-index\n", encoding="utf-8")
        manifest["output"] = str(out)
    else:
        raise SystemExit(f"unsupported required index tool: {tool}")
    return manifest


lock_rows: list[dict[str, object]] = []
log_rows: list[dict[str, object]] = []
for row in rows:
    species = str(row["species_id"])
    build = str(row["build_id"])
    url = str(row["fasta_url"])
    expected = str(row["fasta_sha256"])
    license_id = str(row["license_id"])
    license_url = str(row["license_url"])
    required_indexes = [str(x) for x in row.get("required_indexes", [])]

    root_dir = cache_root / species / build
    raw_dir = root_dir / "refs" / "raw"
    normalized_dir = root_dir / "refs" / "normalized"
    derived_dir = root_dir / "refs" / "derived"
    raw_dir.mkdir(parents=True, exist_ok=True)
    normalized_dir.mkdir(parents=True, exist_ok=True)
    derived_dir.mkdir(parents=True, exist_ok=True)

    raw_fasta = raw_dir / "reference.fa.gz"
    synthetic = f"synthetic reference payload for {species}/{build}\n".encode("utf-8")
    action = "reuse"
    if raw_fasta.exists():
        got = sha256_path(raw_fasta)
        if got != expected and download:
            action = "redownload"
            if verbose:
                print(f"[download] {species}:{build} <- {url}")
            with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                raw_fasta.write_bytes(resp.read())
            got = sha256_path(raw_fasta)
        elif got != expected and not download:
            action = "rewrite-synthetic"
            raw_fasta.write_bytes(synthetic)
            got = sha256_path(raw_fasta)
    else:
        if download:
            action = "download"
            if verbose:
                print(f"[download] {species}:{build} <- {url}")
            with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                raw_fasta.write_bytes(resp.read())
        else:
            action = "write-synthetic"
            raw_fasta.write_bytes(synthetic)
        got = sha256_path(raw_fasta)

    if got != expected:
        raise SystemExit(f"checksum mismatch for {species}:{build}: expected {expected}, got {got}")

    index_outputs = []
    for tool in required_indexes:
        index_outputs.append(run_index_tool(raw_fasta, normalized_dir, tool))

    derived_manifest = derived_dir / "materialization.json"
    derived_manifest.write_text(
        json.dumps(
            {
                "species_id": species,
                "build_id": build,
                "license_id": license_id,
                "license_url": license_url,
                "required_indexes": required_indexes,
                "index_outputs": index_outputs,
            },
            indent=2,
            sort_keys=True,
        )
        + "\n",
        encoding="utf-8",
    )

    lock_rows.append(
        {
            "species_id": species,
            "build_id": build,
            "fasta_url": url,
            "fasta_sha256": expected,
            "observed_sha256": got,
            "license_id": license_id,
            "license_url": license_url,
            "required_indexes": required_indexes,
            "layout": {
                "raw": str(raw_dir.relative_to(cache_root)),
                "normalized": str(normalized_dir.relative_to(cache_root)),
                "derived": str(derived_dir.relative_to(cache_root)),
            },
            "action": action,
        }
    )
    log_rows.append({"species_id": species, "build_id": build, "download": download, "action": action})

payload = {
    "schema_version": 1,
    "generated_at_utc": now_utc(),
    "source": "configs/runtime/reference_bank.toml",
    "references": sorted(lock_rows, key=lambda x: (str(x["species_id"]), str(x["build_id"]))),
}
raw = json.dumps(payload, indent=2, sort_keys=True) + "\n"
lock_json.write_text(raw, encoding="utf-8")
sha = hashlib.sha256(raw.encode("utf-8")).hexdigest()
lock_sha.write_text(f"{sha}  configs/runtime/references/locks/lock.json\n", encoding="utf-8")

run_log = acquire_log_root / f"reference-acquire-{now_utc().replace(':','').replace('-','')}.json"
run_log.write_text(json.dumps({"rows": log_rows, "cache_root": str(cache_root)}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {lock_json.relative_to(root)}")
print(f"wrote {lock_sha.relative_to(root)}")
print(f"wrote {run_log.relative_to(root)}")
PY
