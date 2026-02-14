#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
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
bundles = tomllib.loads((root / "configs/ci/tools/toolkit_bundles.toml").read_text(encoding="utf-8")).get("bundles", {})
images = tomllib.loads((root / "configs/ci/tools/images.toml").read_text(encoding="utf-8"))
appt = {p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")} | {p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def")}
dock = {p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*")}
errors = []

for bid, spec in sorted(bundles.items()):
    tools = spec.get("tools", [])
    if not isinstance(tools, list) or not tools:
        errors.append(f"{bid}: empty tools list")
        continue
    any_buildable = False
    for t in tools:
        meta = images.get(t, {})
        status = str(meta.get("status", "")).strip()
        if t in appt or t in dock:
            any_buildable = True
        elif status not in {"planned"}:
            errors.append(f"{bid}: tool '{t}' is not buildable (no docker/apptainer def)")
    if not any_buildable:
        errors.append(f"{bid}: no buildable tools in bundle")

if errors:
    print("toolkit bundle buildable: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("toolkit bundle buildable: OK")
PY
