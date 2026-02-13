#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ -z "${CI:-}" ]]; then
  echo "apptainer version label sync: SKIP (CI-only gate)"
  exit 0
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
errors = []

for path in sorted((root / "containers/apptainer").rglob("*.def")):
    tool = path.stem
    text = path.read_text(encoding="utf-8", errors="ignore")
    reg = versions.get(tool)
    if not isinstance(reg, dict):
        errors.append(f"{path.relative_to(root)}: missing versions.toml entry")
        continue
    expected = str(reg.get("version", "")).strip()
    m = re.search(r"org\.opencontainers\.image\.version\s+([^\n\r]+)", text)
    if not m:
        errors.append(f"{path.relative_to(root)}: missing org.opencontainers.image.version label")
        continue
    label_value = m.group(1).strip().strip('"\'')
    placeholder = label_value in {"${TOOL_VERSION}", "$TOOL_VERSION", "unknown", "planned", "latest-pinned"} or label_value.endswith("-planned")
    if not placeholder and label_value != expected:
        errors.append(f"{path.relative_to(root)}: label version '{label_value}' != versions.toml '{expected}'")

if errors:
    print("apptainer version label sync: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("apptainer version label sync: OK")
PY
