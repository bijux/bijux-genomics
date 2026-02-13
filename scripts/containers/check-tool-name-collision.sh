#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
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
images = tomllib.loads((root / "configs/ci/tools/images.toml").read_text(encoding="utf-8"))
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))

tools = {}
for reg in regs:
    if not reg.exists():
        continue
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tid:
            continue
        tools[tid] = {
            "expected_bin": str(row.get("expected_bin") or "").strip(),
            "status": str(row.get("status") or "").strip(),
        }

errors = []

# Collision rule: if names differ only by a numeric suffix (e.g. plink/plink2),
# both IDs must be present as separate registry/image/version entries.
for tid in sorted(tools):
    m = re.match(r"^([a-z_]+?)(\d+)$", tid)
    if not m:
        continue
    base = m.group(1)
    if base not in tools:
        continue
    for candidate in (base, tid):
        if candidate not in images:
            errors.append(f"name-collision: missing images entry for '{candidate}'")
        if candidate not in versions:
            errors.append(f"name-collision: missing versions entry for '{candidate}'")

    base_bin = tools[base]["expected_bin"]
    suffixed_bin = tools[tid]["expected_bin"]
    if base_bin and suffixed_bin and base_bin == suffixed_bin:
        errors.append(
            f"name-collision: expected_bin must differ for '{base}' and '{tid}' (both '{base_bin}')"
        )

if errors:
    print("tool-name-collision: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("tool-name-collision: OK")
PY
