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
baseline="${ROOT_DIR}/configs/ci/clippy_allowlist_baseline.toml"

python3 - "$cfg" "$baseline" <<'PY'
from __future__ import annotations
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

cfg = tomllib.loads(Path(sys.argv[1]).read_text(encoding="utf-8"))
base = tomllib.loads(Path(sys.argv[2]).read_text(encoding="utf-8"))

allow = cfg.get("allow", [])
base_entries = base.get("entry", [])
max_entries = int(base.get("max_entries", len(base_entries)))

allow_keys = {(x.get("path"), x.get("lint")) for x in allow if isinstance(x, dict)}
base_keys = {(x.get("path"), x.get("lint")) for x in base_entries if isinstance(x, dict)}

errors = []
if len(allow) > max_entries:
    errors.append(f"allowlist grew: {len(allow)} > max_entries={max_entries}")
extra = sorted(k for k in allow_keys if k not in base_keys)
if extra:
    errors.append("new allowlist entries are forbidden:")
    for path, lint in extra:
        errors.append(f"  - {path} :: {lint}")

if errors:
    print("check-clippy-allowlist-growth: FAILED", file=sys.stderr)
    for e in errors:
        print(e, file=sys.stderr)
    raise SystemExit(1)

print("check-clippy-allowlist-growth: OK")
PY
