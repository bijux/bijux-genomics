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
import json
import re
import sys
from pathlib import Path

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
for p in (panels_path, locks_path, lock_json_path, lock_sha_path):
    if not p.exists():
        errors.append(f"missing {p.relative_to(root)}")
if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
locks = tomllib.loads(locks_path.read_text(encoding="utf-8")).get("locks", {})
lock_json = json.loads(lock_json_path.read_text(encoding="utf-8"))
lock_rows = lock_json.get("panels", [])
lock_by_id = {str(row.get("id", "")): row for row in lock_rows if isinstance(row, dict)}

if not isinstance(panels, list) or not panels:
    errors.append("configs/vcf/panels/panels.toml must define at least one [[panel]]")
if not isinstance(locks, dict):
    errors.append("configs/vcf/panels/locks/panel_locks.toml [locks] table missing")
if lock_json.get("schema_version") != 2:
    errors.append("configs/vcf/panels/locks/lock.json schema_version must be 2")

sha_line = lock_sha_path.read_text(encoding="utf-8").strip().split()[0]
if not re.fullmatch(r"[0-9a-f]{64}", sha_line):
    errors.append("configs/vcf/panels/locks/lock.json.sha256 must start with 64-char sha256")

hex_re = re.compile(r"^[0-9a-f]{64}$")
for panel in panels:
    pid = str(panel.get("id", "")).strip()
    sid = str(panel.get("species_id", "")).strip()
    bid = str(panel.get("build_id", "")).strip()
    if not pid or not sid or not bid:
        errors.append("panel requires id/species_id/build_id")
        continue
    files = panel.get("files", [])
    if not isinstance(files, list) or not files:
        errors.append(f"panel {pid}: files list required")
    else:
        for f in files:
            for key in ("name", "path", "format", "url", "checksum_sha256"):
                if not str(f.get(key, "")).strip():
                    errors.append(f"panel {pid}: file missing {key}")
            sha = str(f.get("checksum_sha256", "")).strip()
            if sha and not hex_re.fullmatch(sha):
                errors.append(f"panel {pid}: file checksum must be 64-char hex")

    compat = panel.get("compatibility", {})
    if not isinstance(compat, dict):
        errors.append(f"panel {pid}: compatibility table required")
    else:
        if not isinstance(compat.get("tool_tags", []), list):
            errors.append(f"panel {pid}: compatibility.tool_tags list required")
        if not str(compat.get("glimpse_reference_format", "")).strip():
            errors.append(f"panel {pid}: compatibility.glimpse_reference_format required")

    lock = locks.get(pid, {})
    if not isinstance(lock, dict) or not lock:
        errors.append(f"panel {pid}: missing locks.{pid} in panel_locks.toml")
        continue
    if str(lock.get("species_id", "")).strip() != sid:
        errors.append(f"panel {pid}: species_id mismatch with lock")
    if str(lock.get("build_id", "")).strip() != bid:
        errors.append(f"panel {pid}: build_id mismatch with lock")
    lock_files = lock.get("files", [])
    if not isinstance(lock_files, list) or not lock_files:
        errors.append(f"panel {pid}: lock files list required")

    j = lock_by_id.get(pid, {})
    if not j:
        errors.append(f"panel {pid}: missing lock.json entry")
    else:
        if str(j.get("species_id", "")).strip() != sid:
            errors.append(f"panel {pid}: lock.json species_id mismatch")
        if str(j.get("build_id", "")).strip() != bid:
            errors.append(f"panel {pid}: lock.json build_id mismatch")
        if str(j.get("version", "")).strip() != str(panel.get("version", "")).strip():
            errors.append(f"panel {pid}: lock.json version mismatch")
        json_files = j.get("files", [])
        if not isinstance(json_files, list) or not json_files:
            errors.append(f"panel {pid}: lock.json files list required")
        else:
            by_name = {str(f.get("name", "")).strip(): f for f in json_files}
            for f in files if isinstance(files, list) else []:
                name = str(f.get("name", "")).strip()
                if not name:
                    continue
                jf = by_name.get(name)
                if not jf:
                    errors.append(f"panel {pid}: lock.json missing file {name}")
                    continue
                for field in ("path", "format", "url"):
                    if str(jf.get(field, "")).strip() != str(f.get(field, "")).strip():
                        errors.append(f"panel {pid}: file {name} {field} mismatch catalog vs lock.json")
                if str(jf.get("checksum_sha256", "")).strip() != str(f.get("checksum_sha256", "")).strip():
                    errors.append(f"panel {pid}: file {name} checksum mismatch catalog vs lock.json")

if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("panel-locks: OK")
PY
