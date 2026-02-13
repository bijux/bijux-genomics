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
external = set(tomllib.loads((root / "configs/domain/external_tools.toml").read_text(encoding="utf-8")).get("non_container_tools", {}).keys())
errors = []

for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    fx_root = dom_dir / "fixtures"
    readme = fx_root / "README.md"
    if not readme.exists():
        errors.append(f"{readme.relative_to(root)} missing")
    for fx in sorted(fx_root.glob("*/*.txt")):
        text = fx.read_text(encoding="utf-8").strip()
        if "=" not in text:
            # legacy compact fixture format: "<stage> <tool> ..."
            parts = text.split()
            if len(parts) < 2:
                errors.append(f"{fx.relative_to(root)}: invalid fixture format")
                continue
            tool = parts[1].strip()
            if tool in external:
                continue
            # force migration for shipped tools
            errors.append(f"{fx.relative_to(root)}: legacy fixture format; use key=value contract fields")
            continue
        kv = {}
        for line in text.splitlines():
            if not line.strip():
                continue
            if "=" not in line:
                continue
            k, v = line.split("=", 1)
            kv[k.strip()] = v.strip()
        for key in ("tool", "stage", "args", "expected_outputs"):
            if key not in kv:
                errors.append(f"{fx.relative_to(root)}: missing required key '{key}'")
        # stage path consistency
        stage_dir = fx.parent.name
        if kv.get("stage") and kv["stage"] != stage_dir:
            errors.append(f"{fx.relative_to(root)}: stage mismatch ({kv['stage']} != {stage_dir})")

if errors:
    print("fixture contracts check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("fixture contracts: OK")
PY
