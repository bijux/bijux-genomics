#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import os
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
lock = root / "containers/versions/lock.json"
summary = root / "artifacts/containers/summary.json"
reg_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
]

if not lock.exists():
    print("lock-vs-built: missing containers/versions/lock.json", file=sys.stderr)
    raise SystemExit(1)
if not summary.exists():
    if "CI" in os.environ:
        print("lock-vs-built: missing artifacts/containers/summary.json", file=sys.stderr)
        raise SystemExit(1)
    print("lock-vs-built: SKIP (no artifacts/containers/summary.json)")
    raise SystemExit(0)

lock_data = json.loads(lock.read_text(encoding="utf-8"))
lock_tools = {item.get("tool") for item in lock_data.get("items", [])}
lock_row = {item.get("tool"): item for item in lock_data.get("items", []) if item.get("tool")}

prod = {}
for rp in reg_paths:
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for t in data.get("tools", []):
        if t.get("status") != "production":
            continue
        tool = str(t.get("id") or t.get("tool_id") or "").strip()
        if not tool or not t.get("container", False):
            continue
        prod[tool] = str(t.get("version", "")).strip()

summary_data = json.loads(summary.read_text(encoding="utf-8"))
manifest_by_tool = {}
for item in summary_data.get("items", []):
    tool = item.get("tool")
    mp = item.get("manifest")
    if not tool or not mp:
        continue
    p = Path(mp)
    if not p.exists():
        continue
    try:
        manifest_by_tool[tool] = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        continue

errors = []
strict_missing = "CI" in os.environ
for tool, expected_version in sorted(prod.items()):
    if tool not in lock_tools:
        errors.append(f"{tool}: missing from containers/versions/lock.json")
    manifest = manifest_by_tool.get(tool)
    if not manifest:
        if strict_missing:
            errors.append(f"{tool}: missing built manifest in artifacts/containers/summary.json")
        continue
    if manifest.get("status") != "ok":
        errors.append(f"{tool}: manifest status is not ok")
    declared_version = str(manifest.get("declared_version", "")).strip()
    if declared_version and expected_version and declared_version != expected_version:
        errors.append(f"{tool}: declared_version '{declared_version}' != registry version '{expected_version}'")
    locked_version = str(lock_row.get(tool, {}).get("version", "")).strip()
    if locked_version and declared_version and locked_version != declared_version:
        errors.append(f"{tool}: lock version '{locked_version}' != declared_version '{declared_version}'")
    version_output = str(manifest.get("version_output", "")).strip()
    if locked_version and locked_version not in {"0.0.0", "planned", "unknown"}:
        if not version_output:
            errors.append(f"{tool}: missing version_output for lock/version comparison")
        elif locked_version.lower() not in version_output.lower():
            errors.append(f"{tool}: version_output '{version_output}' does not contain lock version '{locked_version}'")
    digest = str(manifest.get("resolved_image_digest", "")).strip()
    if not digest:
        errors.append(f"{tool}: missing resolved_image_digest in manifest")
    lock_digest = str(lock_row.get(tool, {}).get("resolved_image_digest", "")).strip()
    if lock_digest and digest and lock_digest != digest:
        errors.append(f"{tool}: built digest '{digest}' does not match lock digest '{lock_digest}'")

if errors:
    print("lock-vs-built: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("lock-vs-built: OK")
PY
