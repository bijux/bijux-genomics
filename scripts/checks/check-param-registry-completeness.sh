#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
param_files = [
    root / "configs/ci/params/param_registry.toml",
    root / "configs/ci/params/param_registry_vcf.toml",
    root / "configs/ci/params/param_registry_downstream.toml",
]
known_stage_ids: set[str] = set()
for p in param_files:
    d = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in d.get("entries", []) + d.get("params", []):
        sid = str(row.get("stage_id", "")).strip()
        if sid:
            known_stage_ids.add(sid)

errors: list[str] = []
for f in sorted(root.glob("domain/*/stages/*.yaml")):
    if f.name.startswith("_"):
        continue
    text = f.read_text(encoding="utf-8")
    sm = re.search(r'^status:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    status = sm.group(1).strip() if sm else ""
    if status not in {"production", "supported"}:
        continue
    m = re.search(r'^stage_id:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    if not m:
        errors.append(f"{f.relative_to(root)}: missing stage_id")
        continue
    sid = m.group(1).strip()
    if sid not in known_stage_ids:
        errors.append(f"{f.relative_to(root)}: stage_id '{sid}' missing from param_registry*.toml")

if errors:
    print("param-registry-completeness: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("param-registry-completeness: OK")
PY
