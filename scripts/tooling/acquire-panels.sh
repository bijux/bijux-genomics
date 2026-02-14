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
Usage: scripts/tooling/acquire-panels.sh [--download] [--panel <panel-id>] [--cache-root <dir>] [--verbose]

This is the only allowed network path for panel acquisition.
Default mode writes/refreshes lock metadata without downloading payloads.
USAGE
}

download=0
verbose=0
panel_id=""
cache_root="${ROOT_DIR}/artifacts/vcf/reference_store/panels"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --download) download=1 ;;
    --panel) panel_id="${2:-}"; shift ;;
    --cache-root) cache_root="${2:-}"; shift ;;
    --verbose) verbose=1 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

python3 - "$ROOT_DIR" "$cache_root" "$panel_id" "$download" "$verbose" <<'PY'
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
panel_filter = sys.argv[3].strip()
download = sys.argv[4] == "1"
verbose = sys.argv[5] == "1"

cfg = tomllib.loads((root / "configs/vcf/panels/panels.toml").read_text(encoding="utf-8"))
panels = cfg.get("panel", [])
if panel_filter:
    panels = [p for p in panels if str(p.get("id", "")) == panel_filter]

acquire_log_root = root / "artifacts" / "containers" / "smoke" / "panel-acquire"
acquire_log_root.mkdir(parents=True, exist_ok=True)
lock_json = root / "configs/vcf/panels/locks/lock.json"
lock_sha = root / "configs/vcf/panels/locks/lock.json.sha256"

def now_utc() -> str:
    sde = os.environ.get("SOURCE_DATE_EPOCH")
    if sde:
        import datetime as dt
        return dt.datetime.fromtimestamp(int(sde), tz=dt.timezone.utc).isoformat().replace("+00:00", "Z")
    return "1970-01-01T00:00:00Z"

log_rows = []
lock_rows = []
for panel in panels:
    pid = str(panel["id"])
    sid = str(panel["species_id"])
    bid = str(panel["build_id"])
    version = str(panel["version"])
    files = panel.get("files", [])

    panel_root = cache_root / sid / bid / pid
    raw_dir = panel_root / "raw"
    normalized_dir = panel_root / "normalized"
    derived_dir = panel_root / "derived"
    raw_dir.mkdir(parents=True, exist_ok=True)
    normalized_dir.mkdir(parents=True, exist_ok=True)
    derived_dir.mkdir(parents=True, exist_ok=True)

    manifest_files = []
    for f in files:
        name = str(f["name"])
        rel_path = str(f["path"])
        url = str(f["url"])
        expected = str(f["checksum_sha256"])
        dest = raw_dir / rel_path
        dest.parent.mkdir(parents=True, exist_ok=True)
        synthetic = f"synthetic payload for {pid}/{name}\n".encode("utf-8")
        action = "reuse"
        if dest.exists():
            got = hashlib.sha256(dest.read_bytes()).hexdigest()
            if got != expected and download:
                action = "redownload"
                if verbose:
                    print(f"[download] {pid}:{name} <- {url}")
                with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                    dest.write_bytes(resp.read())
                got = hashlib.sha256(dest.read_bytes()).hexdigest()
            elif got != expected and not download:
                action = "rewrite-synthetic"
                dest.write_bytes(synthetic)
                got = hashlib.sha256(dest.read_bytes()).hexdigest()
        else:
            if download:
                action = "download"
                if verbose:
                    print(f"[download] {pid}:{name} <- {url}")
                with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                    dest.write_bytes(resp.read())
            else:
                action = "write-synthetic"
                dest.write_bytes(synthetic)
            got = hashlib.sha256(dest.read_bytes()).hexdigest()

        if got != expected:
            raise SystemExit(f"checksum mismatch for {pid}:{name}: expected {expected}, got {got}")
        manifest_files.append({
            "name": name,
            "path": rel_path,
            "materialized_path": str(dest.relative_to(cache_root)),
            "url": url,
            "checksum_sha256": expected,
            "observed_sha256": got,
            "format": str(f["format"]),
            "action": action,
        })

    overlap_stub = derived_dir / "overlap.tsv"
    overlap_stub.write_text("chr\toverlap_sites\toverlap_fraction\nall\t0\t0.0\n", encoding="utf-8")
    index_stub = normalized_dir / "panel.vcf.gz.tbi"
    if not index_stub.exists():
        index_stub.write_text("tabix-index-placeholder\n", encoding="utf-8")

    lock_rows.append({
        "id": pid,
        "species_id": sid,
        "build_id": bid,
        "version": version,
        "license": str(panel.get("license", "")),
        "citation": str(panel.get("citation", "")),
        "files": manifest_files,
        "storage_layout": {
            "raw": str(raw_dir.relative_to(cache_root)),
            "normalized": str(normalized_dir.relative_to(cache_root)),
            "derived": str(derived_dir.relative_to(cache_root)),
        },
    })
    log_rows.append({
        "panel_id": pid,
        "species_id": sid,
        "build_id": bid,
        "download": download,
        "file_count": len(manifest_files),
    })

payload = {
    "schema_version": 2,
    "generated_at_utc": now_utc(),
    "source": "configs/vcf/panels/panels.toml",
    "panels": sorted(lock_rows, key=lambda x: x["id"]),
}
raw = json.dumps(payload, indent=2, sort_keys=True) + "\n"
lock_json.write_text(raw, encoding="utf-8")
sha = hashlib.sha256(raw.encode("utf-8")).hexdigest()
lock_sha.write_text(f"{sha}  configs/vcf/panels/locks/lock.json\n", encoding="utf-8")

run_log = acquire_log_root / f"panel-acquire-{now_utc().replace(':','').replace('-','')}.json"
run_log.write_text(json.dumps({"rows": log_rows, "cache_root": str(cache_root)}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {lock_json.relative_to(root)}")
print(f"wrote {lock_sha.relative_to(root)}")
print(f"wrote {run_log.relative_to(root)}")
PY
