#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

ALLOWLIST="${ALLOWLIST:-$ROOT_DIR/configs/ci/tools/vuln_allowlist.toml}"

python3 - "$ALLOWLIST" <<'PY'
from pathlib import Path
from datetime import datetime, timezone
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

path = Path(sys.argv[1])
if not path.exists():
    print(f"vuln allowlist: missing {path}", file=sys.stderr)
    raise SystemExit(1)

data = tomllib.loads(path.read_text(encoding="utf-8"))
errors = []
seen = set()
now = datetime.now(timezone.utc)
for i, row in enumerate(data.get("allowlist", [])):
    cve = str(row.get("cve", "")).strip().upper()
    reason = str(row.get("reason", "")).strip()
    expires = str(row.get("expires_utc", "")).strip()
    if not cve or not re.fullmatch(r"CVE-\d{4}-\d{4,}", cve):
        errors.append(f"allowlist[{i}] invalid cve: {cve!r}")
        continue
    if cve in seen:
        errors.append(f"duplicate allowlisted cve: {cve}")
    seen.add(cve)
    if len(reason) < 12:
        errors.append(f"{cve}: reason/justification too short")
    if not expires:
        errors.append(f"{cve}: missing expires_utc")
        continue
    try:
        dt = datetime.fromisoformat(expires.replace("Z", "+00:00"))
    except ValueError:
        errors.append(f"{cve}: invalid expires_utc format: {expires}")
        continue
    if dt <= now:
        errors.append(f"{cve}: allowlist entry expired at {expires}")

if errors:
    print("vuln allowlist: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print(f"vuln allowlist: OK ({len(seen)} entries)")
PY
