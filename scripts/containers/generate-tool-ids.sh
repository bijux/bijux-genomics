#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/TOOL_IDS.txt}"
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
tmp="$(mktemp "$TMP_ROOT/tool-ids.XXXXXX")"
trap 'rm -f "$tmp"' EXIT INT TERM

python3 - "$ROOT_DIR" > "$tmp" <<'PY'
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
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
ids = set()
for reg in regs:
    if not reg.exists():
        continue
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tool_id = str(row.get("id") or row.get("tool_id") or "").strip()
        if tool_id:
            ids.add(tool_id)

print("# GENERATED FILE - DO NOT EDIT")
print("# Regenerate with: scripts/containers/generate-tool-ids.sh")
for tool_id in sorted(ids):
    print(tool_id)
PY

mkdir -p "$(dirname "$OUT")"
mv "$tmp" "$OUT"
echo "generated $OUT"
