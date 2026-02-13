#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
reg = root / "configs/ci/registry/tool_registry_vcf_downstream.toml"
if not reg.exists():
    print("bijux-template-markers: OK (no downstream registry)")
    raise SystemExit(0)

data = tomllib.loads(reg.read_text(encoding="utf-8"))
errors = []
for row in data.get("tools", []):
    if not isinstance(row, dict):
        continue
    apptainer_def = str(row.get("apptainer_def") or "").strip()
    if not apptainer_def.startswith("containers/apptainer/bijux/"):
        continue
    path = root / apptainer_def
    if not path.exists():
        errors.append(f"missing bijux def: {apptainer_def}")
        continue
    head = "\n".join(path.read_text(encoding="utf-8").splitlines()[:8])
    if "BIJUX_TEMPLATE: v1" not in head:
        errors.append(f"{apptainer_def}: missing BIJUX_TEMPLATE: v1 marker")

if errors:
    print("bijux-template-markers: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("bijux-template-markers: OK")
PY
