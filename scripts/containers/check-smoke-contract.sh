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
contract_doc = root / "containers/docs/SMOKE_CONTRACT.md"
if not contract_doc.exists():
    print("smoke contract check failed: missing containers/docs/SMOKE_CONTRACT.md", file=sys.stderr)
    raise SystemExit(1)
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
        m = str(row.get("smoke_minimal_cmd", "")).strip() or f"{tool_id} --help"
        me = row.get("smoke_minimal_exit_code", 0)
        he = row.get("smoke_help_exit_code", 0)
        neg = str(row.get("smoke_negative_cmd", "")).strip() or f"{tool_id} --__bijux_invalid_flag__"
        ne = row.get("smoke_negative_exit_code", 2)
        npat = str(row.get("smoke_negative_expected_pattern", "")).strip() or "invalid|unknown|error|usage"
        expected_bin = str(row.get("expected_bin", "")).strip()
        if not v:
            errors.append(f"{reg}: {tool_id} missing smoke_version_cmd")
        if not h:
            errors.append(f"{reg}: {tool_id} missing smoke_help_cmd")
        if not isinstance(he, int):
            errors.append(f"{reg}: {tool_id} smoke_help_exit_code must be integer")
        elif he != 0:
            errors.append(f"{reg}: {tool_id} smoke_help_exit_code must be 0")
        if not expected_bin:
            errors.append(f"{reg}: {tool_id} missing expected_bin tool binary contract")
        # Per-tool smoke spec contract:
        # help/version/minimal + one expected-failure command.
        if not m:
            errors.append(f"{reg}: {tool_id} resolved smoke_minimal_cmd is empty")
        if not isinstance(me, int):
            errors.append(f"{reg}: {tool_id} smoke_minimal_exit_code must be integer")
        if not neg:
            errors.append(f"{reg}: {tool_id} resolved smoke_negative_cmd is empty")
        if not isinstance(ne, int):
            errors.append(f"{reg}: {tool_id} smoke_negative_exit_code must be integer")
        if not npat:
            errors.append(f"{reg}: {tool_id} resolved smoke_negative_expected_pattern is empty")

if errors:
    print("smoke contract check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("smoke contract: OK")
PY
