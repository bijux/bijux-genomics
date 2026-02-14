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

root = Path(sys.argv[1])
bijux_dir = root / "containers/apptainer/bijux"
template = bijux_dir / "TEMPLATE.def.inc"
errors = []

if not template.exists():
    errors.append("missing template file containers/apptainer/bijux/TEMPLATE.def.inc")

for path in sorted(bijux_dir.glob("*.def")):
    head = "\n".join(path.read_text(encoding="utf-8").splitlines()[:20])
    if "BIJUX_TEMPLATE: v1" not in head:
        errors.append(f"{path.relative_to(root)}: missing BIJUX_TEMPLATE: v1 marker")

if errors:
    print("bijux-template-markers: failed", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("bijux-template-markers: OK")
PY
