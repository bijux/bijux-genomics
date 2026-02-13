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
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
down = tomllib.loads((root / "configs/ci/registry/tool_registry_vcf_downstream.toml").read_text(encoding="utf-8"))
down_ids = {str(t.get("id") or t.get("tool_id") or "") for t in down.get("tools", [])}

errors = []
for tid in sorted(x for x in down_ids if x):
    entry = versions.get(tid)
    if not isinstance(entry, dict):
        errors.append(f"{tid}: missing versions entry")
        continue
    source = str(entry.get("source", ""))
    needs_hash = any(token in source for token in (".zip", ".tar.gz", "github.com", "faculty.washington.edu", "sourceforge.net"))
    if not needs_hash:
        continue
    source_sha = str(entry.get("source_sha256", "")).strip()
    pin = str(entry.get("pinned_commit", "")).strip()
    if not source_sha and not pin:
        errors.append(f"{tid}: missing source_sha256 or pinned_commit for source build")
    if source_sha and len(source_sha) != 64:
        errors.append(f"{tid}: source_sha256 must be 64 hex chars")
    if pin.lower() in {"pending", "unknown"}:
        errors.append(f"{tid}: pinned_commit must not be pending/unknown")

if errors:
    print("version hash pin check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("version hash pin: OK")
PY
