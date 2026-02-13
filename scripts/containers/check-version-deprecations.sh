#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
from datetime import date
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
deps_path = root / "containers/versions/deprecations.toml"
lock = json.loads((root / "containers/versions/lock.json").read_text(encoding="utf-8"))
lock_tools = {row.get("tool") for row in lock.get("items", [])}
today = date.today()

errors = []
if deps_path.exists():
    deps = tomllib.loads(deps_path.read_text(encoding="utf-8"))
    for row in deps.get("deprecation", []):
        tool = str(row.get("tool_id", "")).strip()
        version = str(row.get("version", "")).strip()
        ds = str(row.get("deprecated_since", "")).strip()
        ra = str(row.get("removal_after", "")).strip()
        mode = str(row.get("compatibility_mode", "")).strip()
        if not tool or not version:
            errors.append("deprecation row missing tool_id/version")
            continue
        if tool not in versions:
            errors.append(f"{tool}: deprecation refers to unknown tool")
        else:
            current = str(versions[tool].get("version", "")).strip()
            if current != version:
                errors.append(f"{tool}: deprecation version '{version}' does not match versions.toml '{current}'")
        if tool not in lock_tools:
            errors.append(f"{tool}: missing from lock.json, breaks reproducibility")
        try:
            d1 = date.fromisoformat(ds)
            d2 = date.fromisoformat(ra)
            if d2 <= d1:
                errors.append(f"{tool}: removal_after must be after deprecated_since")
            if mode == "allowed" and today > d2:
                errors.append(f"{tool}: compatibility_mode=allowed expired after {ra}")
        except Exception:
            errors.append(f"{tool}: invalid dates in deprecations.toml")
        if mode not in {"allowed", "blocked"}:
            errors.append(f"{tool}: compatibility_mode must be allowed|blocked")

if errors:
    print("version deprecations: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("version deprecations: OK")
PY
