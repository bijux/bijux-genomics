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
    print("build-provenance: OK (no downstream registry)")
    raise SystemExit(0)

data = tomllib.loads(reg.read_text(encoding="utf-8"))
errors = []

for row in data.get("tools", []):
    if not isinstance(row, dict):
        continue
    if not bool(row.get("container", False)):
        continue
    tid = str(row.get("id") or row.get("tool_id") or "").strip()
    dockerfile = str(row.get("dockerfile") or "").strip()
    apptainer_def = str(row.get("apptainer_def") or "").strip()

    if dockerfile:
        path = root / dockerfile
        if not path.exists():
            errors.append(f"{tid}: missing dockerfile {dockerfile}")
        else:
            text = path.read_text(encoding="utf-8")
            if "/opt/bijux/VERSION.json" not in text:
                errors.append(f"{tid}: dockerfile missing provenance file write /opt/bijux/VERSION.json")
            if "bijux-tool-info" not in text:
                errors.append(f"{tid}: dockerfile missing bijux-tool-info self-report command")

    if apptainer_def:
        path = root / apptainer_def
        if not path.exists():
            errors.append(f"{tid}: missing apptainer def {apptainer_def}")
        else:
            text = path.read_text(encoding="utf-8")
            if "/opt/bijux/VERSION.json" not in text:
                errors.append(f"{tid}: apptainer def missing provenance file write /opt/bijux/VERSION.json")
            if "bijux-tool-info" not in text:
                errors.append(f"{tid}: apptainer def missing bijux-tool-info self-report command")

if errors:
    print("build-provenance: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("build-provenance: OK")
PY
