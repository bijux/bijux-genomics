#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ -z "${CI:-}" ]]; then
  echo "version immutability: SKIP (CI-only gate)"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import subprocess
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
p = subprocess.run(["git", "-C", str(root), "rev-parse", "--verify", "HEAD^"], capture_output=True, text=True)
if p.returncode != 0:
    print("version immutability: SKIP (no previous commit)")
    raise SystemExit(0)
prev = p.stdout.strip()
show = subprocess.run(["git", "-C", str(root), "show", f"{prev}:containers/versions/versions.toml"], capture_output=True, text=True)
if show.returncode != 0:
    print("version immutability: SKIP (no previous versions.toml)")
    raise SystemExit(0)

prev_data = tomllib.loads(show.stdout)
now_data = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
errors = []
for tool, prev_row in prev_data.items():
    if not isinstance(prev_row, dict):
        continue
    now_row = now_data.get(tool)
    if not isinstance(now_row, dict):
        continue
    prev_status = str(prev_row.get("status", "production")).strip()
    now_status = str(now_row.get("status", prev_status)).strip()
    prev_ver = str(prev_row.get("version", "")).strip()
    now_ver = str(now_row.get("version", "")).strip()
    if prev_status == "production" and now_status == "production" and prev_ver and now_ver and prev_ver != now_ver:
        errors.append(f"{tool}: production version is immutable ({prev_ver} -> {now_ver})")

if errors:
    print("version immutability: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("version immutability: OK")
PY
