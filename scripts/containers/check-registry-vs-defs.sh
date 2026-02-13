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
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]

registry_ids = set()
for reg in regs:
    if not reg.exists():
        continue
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if tid:
            registry_ids.add(tid)

def_ids = set()
for d in (root / "containers/docker/arm64").glob("Dockerfile.*"):
    def_ids.add(d.name.split("Dockerfile.", 1)[1])
for d in (root / "containers/apptainer/bijux").glob("*.def"):
    def_ids.add(d.stem)
for d in (root / "containers/apptainer/non-bijux").glob("*.def"):
    def_ids.add(d.stem)

orphans = sorted(def_ids - registry_ids)
if orphans:
    print("registry-vs-defs: orphan container defs not present in registries:", file=sys.stderr)
    for tid in orphans:
        print(f"- {tid}", file=sys.stderr)
    raise SystemExit(1)

print("registry-vs-defs: OK")
PY
