#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
import re
import sys
from pathlib import Path
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
maps_path = root / "configs/vcf/maps/maps.toml"
locks_path = root / "configs/vcf/maps/locks/map_locks.toml"

errors = []
for p in (maps_path, locks_path):
    if not p.exists():
        errors.append(f"missing {p.relative_to(root)}")
if errors:
    print("map-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

maps = tomllib.loads(maps_path.read_text(encoding="utf-8")).get("map", [])
locks = tomllib.loads(locks_path.read_text(encoding="utf-8")).get("locks", {})
if not maps:
    errors.append("maps.toml must include at least one [[map]]")
if not isinstance(locks, dict):
    errors.append("map_locks.toml [locks] table missing")

hex_re = re.compile(r"^[0-9a-f]{64}$")
for m in maps:
    map_id = str(m.get("id", "")).strip()
    sid = str(m.get("species_id", "")).strip()
    bid = str(m.get("build_id", "")).strip()
    if not map_id or not sid or not bid:
        errors.append("map requires id/species_id/build_id")
        continue
    files = m.get("files", [])
    if not isinstance(files, list) or not files:
        errors.append(f"map {map_id}: files list required")
    else:
        for f in files:
            sha = str(f.get("checksum_sha256", "")).strip()
            if not hex_re.fullmatch(sha):
                errors.append(f"map {map_id}: checksum must be 64-char hex")
    lock = locks.get(map_id, {})
    if not isinstance(lock, dict) or not lock:
        errors.append(f"map {map_id}: missing locks.{map_id}")
        continue
    if str(lock.get("species_id", "")).strip() != sid:
        errors.append(f"map {map_id}: species mismatch with lock")
    if str(lock.get("build_id", "")).strip() != bid:
        errors.append(f"map {map_id}: build mismatch with lock")

if errors:
    print("map-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("map-locks: OK")
PY
