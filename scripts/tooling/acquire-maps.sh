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
Usage: scripts/tooling/acquire-maps.sh [--download] [--map <map-id>] [--cache-root <dir>] [--verbose]
USAGE
}

download=0
verbose=0
map_id=""
cache_root="${ROOT_DIR}/artifacts/vcf/reference_store/maps"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --help|-h) usage; exit 0 ;;
    --download) download=1 ;;
    --map) map_id="${2:-}"; shift ;;
    --cache-root) cache_root="${2:-}"; shift ;;
    --verbose) verbose=1 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
  shift
done

python3 - "$ROOT_DIR" "$cache_root" "$map_id" "$download" "$verbose" <<'PY'
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
map_filter = sys.argv[3].strip()
download = sys.argv[4] == "1"
verbose = sys.argv[5] == "1"

cfg = tomllib.loads((root / "configs/vcf/maps/maps.toml").read_text(encoding="utf-8"))
maps = cfg.get("map", [])
if map_filter:
    maps = [m for m in maps if str(m.get("id", "")) == map_filter]

acquire_log_root = root / "artifacts" / "containers" / "smoke" / "map-acquire"
acquire_log_root.mkdir(parents=True, exist_ok=True)

def now_utc() -> str:
    sde = os.environ.get("SOURCE_DATE_EPOCH")
    if sde:
        import datetime as dt
        return dt.datetime.fromtimestamp(int(sde), tz=dt.timezone.utc).isoformat().replace("+00:00", "Z")
    return "1970-01-01T00:00:00Z"

rows = []
for m in maps:
    mid = str(m["id"])
    sid = str(m["species_id"])
    bid = str(m["build_id"])
    files = m.get("files", [])
    base = cache_root / sid / bid / mid
    raw_dir = base / "raw"
    normalized_dir = base / "normalized"
    derived_dir = base / "derived"
    raw_dir.mkdir(parents=True, exist_ok=True)
    normalized_dir.mkdir(parents=True, exist_ok=True)
    derived_dir.mkdir(parents=True, exist_ok=True)

    observed = []
    for f in files:
        name = str(f["name"])
        rel = str(f["path"])
        url = str(f["url"])
        expected = str(f["checksum_sha256"])
        target = raw_dir / rel
        target.parent.mkdir(parents=True, exist_ok=True)
        synthetic = f"synthetic payload for {mid}/{name}\n".encode("utf-8")
        action = "reuse"
        if target.exists():
            got = hashlib.sha256(target.read_bytes()).hexdigest()
            if got != expected and download:
                action = "redownload"
                if verbose:
                    print(f"[download] {mid}:{name} <- {url}")
                with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                    target.write_bytes(resp.read())
                got = hashlib.sha256(target.read_bytes()).hexdigest()
            elif got != expected and not download:
                action = "rewrite-synthetic"
                target.write_bytes(synthetic)
                got = hashlib.sha256(target.read_bytes()).hexdigest()
        else:
            if download:
                action = "download"
                if verbose:
                    print(f"[download] {mid}:{name} <- {url}")
                with urllib.request.urlopen(url) as resp:  # nosec B310 - explicit governance path.
                    target.write_bytes(resp.read())
            else:
                action = "write-synthetic"
                target.write_bytes(synthetic)
            got = hashlib.sha256(target.read_bytes()).hexdigest()
        if got != expected:
            raise SystemExit(f"checksum mismatch for {mid}:{name}: expected {expected}, got {got}")
        observed.append({
            "name": name,
            "checksum_sha256": expected,
            "observed_sha256": got,
            "path": str(target.relative_to(cache_root)),
            "action": action,
        })

    derived_dir.joinpath("chunk_index.tsv").write_text("chunk\tregion\n0\tall\n", encoding="utf-8")
    rows.append({"map_id": mid, "species_id": sid, "build_id": bid, "files": observed})

run_log = acquire_log_root / f"map-acquire-{now_utc().replace(':','').replace('-','')}.json"
run_log.write_text(json.dumps({"rows": rows, "cache_root": str(cache_root)}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {run_log.relative_to(root)}")
PY
