#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/docs/TOOL_NAME_MAP.md}"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
regs = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
rows = {}
for rp in regs:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tid:
            continue
        rows[tid] = {
            "expected_bin": str(row.get("expected_bin", "")).strip() or tid,
            "status": str(row.get("status", "")).strip(),
        }

lines = [
    "<!-- GENERATED FILE - DO NOT EDIT -->",
    "<!-- Regenerate with: scripts/containers/generate-tool-name-map.sh -->",
    "",
    "# Tool Name Mapping",
    "",
    "| Tool ID | Expected Binary | Status |",
    "|---|---|---|",
]
for tid in sorted(rows):
    r = rows[tid]
    lines.append(f"| `{tid}` | `{r['expected_bin']}` | `{r['status']}` |")
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
