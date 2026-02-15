#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL

cfg="${ROOT_DIR}/configs/ci/clippy_allowlist.toml"
if [[ ! -f "$cfg" ]]; then
  echo "ERROR: missing clippy allowlist config: $cfg" >&2
  exit 1
fi

python3 - "$ROOT_DIR" "$cfg" <<'PY'
from __future__ import annotations
from pathlib import Path
from datetime import date
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
cfg = Path(sys.argv[2])
data = tomllib.loads(cfg.read_text(encoding="utf-8"))
entries = data.get("allow", [])
if not isinstance(entries, list):
    raise SystemExit("ERROR: [allow] entries must be an array of tables")

errors = []
for idx, entry in enumerate(entries, 1):
    path = entry.get("path")
    lint = entry.get("lint")
    expires_on = entry.get("expires_on")
    reason = entry.get("reason")
    if not all(isinstance(v, str) and v.strip() for v in (path, lint, expires_on, reason)):
        errors.append(f"entry #{idx}: path/lint/expires_on/reason are required non-empty strings")
        continue
    try:
        exp = date.fromisoformat(expires_on)
    except Exception:
        errors.append(f"entry #{idx}: invalid expires_on date: {expires_on}")
        continue
    if exp < date.today():
        errors.append(f"entry #{idx}: expired allow entry for {path} ({lint}) expired_on={expires_on}")
        continue
    f = root / path
    if not f.exists():
        errors.append(f"entry #{idx}: path does not exist: {path}")
        continue
    content = f.read_text(encoding="utf-8")
    pat = re.compile(r"#\[allow\(clippy::" + re.escape(lint) + r"\)\]")
    if not pat.search(content):
        errors.append(f"entry #{idx}: allow(clippy::{lint}) not found in {path}")

if errors:
    print("ERROR: clippy allowlist expiry check failed", file=sys.stderr)
    for err in errors:
        print(f"  - {err}", file=sys.stderr)
    raise SystemExit(1)
print("check-clippy-allowlist-expiry: OK")
PY
