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
regs = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
images = root / "configs/ci/tools/images.toml"
exempt = set()
if images.exists():
    data = tomllib.loads(images.read_text(encoding="utf-8"))
    tbl = data.get("smoke_exemptions", {})
    if isinstance(tbl, dict):
        exempt = {str(k) for k, v in tbl.items() if bool(v)}

errors = []
for reg in regs:
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        status = row.get("status")
        if status not in ("production", "supported"):
            if not (reg.name == "tool_registry_vcf_downstream.toml" and status == "planned"):
                continue
        if not row.get("container", False):
            continue
        tool_id = str(row.get("id") or row.get("tool_id") or "<unknown>")
        if tool_id in exempt:
            continue
        v = str(row.get("smoke_version_cmd", "")).strip()
        h = str(row.get("smoke_help_cmd", "")).strip()
        m = str(row.get("smoke_minimal_cmd", "")).strip()
        me = row.get("smoke_minimal_exit_code", None)
        if not v:
            errors.append(f"{reg}: {tool_id} missing smoke_version_cmd")
        if not h:
            errors.append(f"{reg}: {tool_id} missing smoke_help_cmd")
        if reg.name == "tool_registry_vcf_downstream.toml":
            if not m:
                errors.append(f"{reg}: {tool_id} missing smoke_minimal_cmd")
            if me is None or not isinstance(me, int):
                errors.append(f"{reg}: {tool_id} missing integer smoke_minimal_exit_code")

if errors:
    print("smoke contract check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("smoke contract: OK")
PY
