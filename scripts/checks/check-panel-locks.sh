#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
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

errors = []
if not panels_path.exists():
    errors.append(f"missing {panels_path.relative_to(root)}")
if not locks_path.exists():
    errors.append(f"missing {locks_path.relative_to(root)}")
if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

panels = tomllib.loads(panels_path.read_text(encoding="utf-8")).get("panel", [])
locks = tomllib.loads(locks_path.read_text(encoding="utf-8")).get("locks", {})

if not isinstance(panels, list) or not panels:
    errors.append("configs/vcf/panels/panels.toml must define at least one [[panel]] entry")

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
    if not checksum:
        errors.append(f"panel {pid}: missing checksum_sha256")
    elif not re.match(r"^sha256:[0-9a-f]{64}$", checksum):
        errors.append(f"panel {pid}: checksum_sha256 must match sha256:<64-hex>")

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

if errors:
    print("panel-locks: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("panel-locks: OK")
PY
