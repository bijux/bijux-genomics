#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

POLICY="${ROOT_DIR}/configs/ci/tools/smoke_inputs_policy.toml"
[[ -f "$POLICY" ]] || { echo "smoke-inputs policy: missing $POLICY" >&2; exit 1; }

python3 - "$ROOT_DIR" "$POLICY" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
policy = Path(sys.argv[2])
data = tomllib.loads(policy.read_text(encoding="utf-8"))
entries = data.get("tool_inputs", {})
errors = []

for tool, row in sorted(entries.items()):
    if not isinstance(row, dict):
        errors.append(f"{tool}: policy row must be table")
        continue
    rel = str(row.get("path", "")).strip()
    if not rel:
        errors.append(f"{tool}: missing path")
        continue
    p = root / rel
    if not p.exists():
        errors.append(f"{tool}: missing input file {rel}")
        continue
    if not p.is_file():
        errors.append(f"{tool}: input path is not a file {rel}")
        continue
    if p.stat().st_size == 0:
        errors.append(f"{tool}: input file is empty {rel}")

if errors:
    print("smoke-inputs policy: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print(f"smoke-inputs policy: OK ({len(entries)} entries)")
PY

