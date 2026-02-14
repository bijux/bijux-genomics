#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
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
regs = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
errors = []

def first_token(cmd: str) -> str:
    cmd = (cmd or "").strip()
    if not cmd:
        return ""
    return re.split(r"\s+", cmd, maxsplit=1)[0]

for reg in regs:
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        runtimes = row.get("runtimes", [])
        if "apptainer" not in runtimes and "docker" not in runtimes:
            continue
        tool = str(row.get("id") or row.get("tool_id") or "").strip()
        expected_bin = str(row.get("expected_bin", "")).strip()
        if not tool:
            continue
        if not expected_bin:
            errors.append(f"{reg.name}:{tool}: missing expected_bin")
            continue
        for field in ("smoke_version_cmd", "smoke_help_cmd"):
            cmd = str(row.get(field, "")).strip()
            if not cmd:
                errors.append(f"{reg.name}:{tool}: missing {field}")
                continue
            tok = first_token(cmd)
            if tok != expected_bin:
                errors.append(
                    f"{reg.name}:{tool}: {field} must start with expected_bin '{expected_bin}', got '{tok}'"
                )

if errors:
    print("tool invocation normalization: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("tool invocation normalization: OK")
PY

