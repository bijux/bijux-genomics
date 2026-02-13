#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/checks/check-audit-allowlist.sh
USAGE
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import datetime as dt
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
path = root / "audit-allowlist.toml"
if not path.exists():
    print("audit-allowlist: missing audit-allowlist.toml", file=sys.stderr)
    raise SystemExit(1)

data = tomllib.loads(path.read_text(encoding="utf-8"))
rows = data.get("advisory", [])
if not isinstance(rows, list):
    print("audit-allowlist: advisory entries must be [[advisory]] array", file=sys.stderr)
    raise SystemExit(1)

errors = []
today = dt.date.today()
for i, row in enumerate(rows):
    tag = f"entry[{i}]"
    adv = str(row.get("id", "")).strip()
    why = str(row.get("why", "")).strip()
    expiry = str(row.get("expiry", "")).strip()
    owner = str(row.get("owner", "")).strip()
    link = str(row.get("link", "")).strip()

    if not re.match(r"^RUSTSEC-\d{4}-\d{4}$", adv):
        errors.append(f"{tag}: id must match RUSTSEC-YYYY-NNNN")
    if not why:
        errors.append(f"{tag}: missing why")
    if not owner:
        errors.append(f"{tag}: missing owner")
    if not link.startswith(("http://", "https://")):
        errors.append(f"{tag}: link must be http(s)")
    if not expiry:
        errors.append(f"{tag}: missing expiry")
    else:
        try:
            exp = dt.date.fromisoformat(expiry)
            if exp < today:
                errors.append(f"{tag}: expiry has passed ({expiry})")
        except Exception:
            errors.append(f"{tag}: expiry must be YYYY-MM-DD")

if errors:
    print("audit-allowlist: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("audit-allowlist: OK")
PY
