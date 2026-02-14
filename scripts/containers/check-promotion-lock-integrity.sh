#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ -z "${CI:-}" ]]; then
  echo "promotion lock integrity: SKIP (CI-only gate)"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
lock = json.loads((root / "containers/versions/lock.json").read_text(encoding="utf-8"))
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
reg_files = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]

lock_by_tool = {str(i.get("tool", "")).strip(): i for i in lock.get("items", [])}
prod = set()
for rp in reg_files:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if str(row.get("status", "")).strip() != "production":
            continue
        tool = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tool:
            continue
        prod.add(tool)

errors = []
for tool in sorted(prod):
    if tool not in lock_by_tool:
        errors.append(f"{tool}: production tool missing from lock.json")
        continue
    row = lock_by_tool[tool]
    lock_version = str(row.get("version", "")).strip()
    reg_version = str((versions.get(tool) or {}).get("version", "")).strip()
    if lock_version != reg_version:
        errors.append(f"{tool}: lock version '{lock_version}' != versions.toml '{reg_version}'")
    d_docker = str(row.get("resolved_image_digest", "")).strip()
    d_sif = str(row.get("resolved_sif_sha256", "")).strip()
    if not d_docker and not d_sif:
        errors.append(f"{tool}: promotion requires at least one locked artifact digest (docker/apptainer)")

if errors:
    print("promotion lock integrity: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("promotion lock integrity: OK")
PY
