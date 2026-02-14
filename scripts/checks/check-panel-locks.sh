#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-panel-locks.sh
USAGE
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
import hashlib
import json
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
panels_path = root / "configs/vcf/panels/panels.toml"
locks_path = root / "configs/vcf/panels/locks/panel_locks.toml"
lock_json_path = root / "configs/vcf/panels/locks/lock.json"
lock_sha_path = root / "configs/vcf/panels/locks/lock.json.sha256"

errors = []
if not panels_path.exists():
    errors.append(f"missing {panels_path.relative_to(root)}")
if not locks_path.exists():
    errors.append(f"missing {locks_path.relative_to(root)}")
if not lock_json_path.exists():
    errors.append(f"missing {lock_json_path.relative_to(root)}")
if not lock_sha_path.exists():
    errors.append(f"missing {lock_sha_path.relative_to(root)}")
if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
locks = tomllib.loads(locks_path.read_text(encoding="utf-8")).get("locks", {})
lock_json = json.loads(lock_json_path.read_text(encoding="utf-8"))
lock_items = lock_json.get("panels", [])
lock_by_id = {str(item.get("id", "")).strip(): item for item in lock_items if isinstance(item, dict)}

if not isinstance(panels, list) or not panels:
    errors.append("configs/vcf/panels/panels.toml must define at least one [[panel]] entry")

if lock_json.get("schema_version") != 1:
    errors.append("configs/vcf/panels/locks/lock.json: schema_version must be 1")
if str(lock_json.get("source", "")).strip() != "configs/vcf/panels/panels.toml":
    errors.append("configs/vcf/panels/locks/lock.json: source must be configs/vcf/panels/panels.toml")

raw_lock = lock_json_path.read_bytes()
sha = hashlib.sha256(raw_lock).hexdigest()
sha_line = lock_sha_path.read_text(encoding="utf-8").strip()
expected_sha_line = f"{sha}  configs/vcf/panels/locks/lock.json"
if sha_line != expected_sha_line:
    errors.append("configs/vcf/panels/locks/lock.json.sha256 does not match lock.json content")

re_sha = re.compile(r"^sha256:[0-9a-f]{64}$")

for p in panels:
    pid = str(p.get("id", "")).strip()
    if not pid:
        errors.append("panel entry missing id")
        continue
    version = str(p.get("version", "")).strip()
    url = str(p.get("url", "")).strip()
    checksum = str(p.get("checksum_sha256", "")).strip()
    if not version:
        errors.append(f"panel {pid}: missing version")
    if not url:
        errors.append(f"panel {pid}: missing url")
    if url and not re.match(r"^https?://", url):
        errors.append(f"panel {pid}: url must be http(s)")
    if url and re.search(r"(/|@)(latest|main|master)($|[/?#])", url):
        errors.append(f"panel {pid}: floating URL token detected ({url})")
    if not checksum:
        errors.append(f"panel {pid}: missing checksum_sha256")
    elif not re_sha.match(checksum):
        errors.append(f"panel {pid}: checksum_sha256 must match sha256:<64-hex>")
    for meta_field in ("population_set", "genome_build", "variant_set_compatibility", "citation"):
        mv = str(p.get(meta_field, "")).strip()
        if not mv:
            errors.append(f"panel {pid}: missing {meta_field}")

    lock = locks.get(pid, {}) if isinstance(locks, dict) else {}
    if not lock:
        errors.append(f"panel {pid}: missing lock entry in panel_locks.toml")
        continue
    for field in ("version", "url", "checksum_sha256"):
        lv = str(lock.get(field, "")).strip()
        if not lv:
            errors.append(f"panel {pid}: lock missing {field}")
    if version and str(lock.get("version", "")).strip() != version:
        errors.append(f"panel {pid}: version mismatch between panels.toml and panel_locks.toml")
    if url and str(lock.get("url", "")).strip() != url:
        errors.append(f"panel {pid}: url mismatch between panels.toml and panel_locks.toml")
    if checksum and str(lock.get("checksum_sha256", "")).strip() != checksum:
        errors.append(f"panel {pid}: checksum mismatch between panels.toml and panel_locks.toml")

    jlock = lock_by_id.get(pid, {})
    if not jlock:
        errors.append(f"panel {pid}: missing lock entry in lock.json")
        continue
    for field in ("url", "version", "sha256", "date", "license", "citation"):
        if not str(jlock.get(field, "")).strip():
            errors.append(f"panel {pid}: lock.json missing {field}")
    if checksum and str(jlock.get("sha256", "")).strip() != checksum:
        errors.append(f"panel {pid}: checksum mismatch between panels.toml and lock.json")
    if url and str(jlock.get("url", "")).strip() != url:
        errors.append(f"panel {pid}: url mismatch between panels.toml and lock.json")
    if version and str(jlock.get("version", "")).strip() != version:
        errors.append(f"panel {pid}: version mismatch between panels.toml and lock.json")

    derived = jlock.get("derived_artifacts", [])
    if not isinstance(derived, list) or not derived:
        errors.append(f"panel {pid}: lock.json derived_artifacts must be a non-empty list")
    else:
        has_index = any(str(x).endswith(".tbi") for x in derived)
        has_chunks = any(str(x).endswith(".chunks.tsv") for x in derived)
        if not has_index:
            errors.append(f"panel {pid}: lock.json derived_artifacts must include a .tbi index")
        if not has_chunks:
            errors.append(f"panel {pid}: lock.json derived_artifacts must include a .chunks.tsv manifest")

    derived_checksums = jlock.get("derived_checksums_sha256", {})
    if not isinstance(derived_checksums, dict) or not derived_checksums:
        errors.append(f"panel {pid}: lock.json must include derived_checksums_sha256 table")
    else:
        for name, value in derived_checksums.items():
            if not re_sha.match(str(value)):
                errors.append(f"panel {pid}: invalid derived checksum for {name}")

if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("panel-locks: OK")
PY
