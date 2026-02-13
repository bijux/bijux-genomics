#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
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
reg_files = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
req_files = [
    root / "configs/ci/tools/required_tools.toml",
    root / "configs/ci/tools/required_tools_vcf.toml",
    root / "configs/ci/tools/required_tools_vcf_downstream.toml",
]
known: set[str] = set()
for p in reg_files:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if tid:
            known.add(tid)

errors: list[str] = []
for p in req_files:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    for tid in data.get("required_tools", []):
        if str(tid) not in known:
            errors.append(f"{p.relative_to(root)}: required_tools entry '{tid}' has no registry definition")

if errors:
    print("registry-required-tools-parity: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("registry-required-tools-parity: OK")
PY
