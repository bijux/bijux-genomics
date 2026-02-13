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
license_dir = root / "containers/licenses"
errors = []
for path in sorted((root / "containers/apptainer").rglob("*.def")):
    tool = path.stem
    meta = license_dir / f"{tool}.license.toml"
    if not meta.exists():
        errors.append(f"missing {meta.relative_to(root)}")
        continue
    data = tomllib.loads(meta.read_text(encoding="utf-8"))
    for key in ("tool_id", "container_kind", "spdx", "upstream_url", "redistribution_note", "citation", "version"):
        if not str(data.get(key, "")).strip():
            errors.append(f"{meta.relative_to(root)} missing key: {key}")
    if data.get("tool_id") != tool:
        errors.append(f"{meta.relative_to(root)} tool_id mismatch")
    up = str(data.get("upstream_url", ""))
    if not up.startswith(("http://", "https://")):
        errors.append(f"{meta.relative_to(root)} upstream_url must be URL")

if errors:
    print("license metadata check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("license metadata: OK")
PY
