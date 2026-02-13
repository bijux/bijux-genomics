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

root = Path(sys.argv[1])
idx = root / "configs/logging/index.md"
knobs = root / "configs/logging/runtime.toml"
errors = []

if not idx.exists():
    errors.append("configs/logging/index.md missing")
else:
    t = idx.read_text(encoding="utf-8")
    for required in ("RUST_LOG", "BIJUX_LOG_FORMAT", "trace_id", "run_id"):
        if required not in t:
            errors.append(f"configs/logging/index.md must document '{required}'")
    if "runtime.toml" not in t:
        errors.append("configs/logging/index.md must reference runtime.toml")

if not knobs.exists():
    errors.append("configs/logging/runtime.toml missing")
else:
    k = knobs.read_text(encoding="utf-8")
    for key in ("default_level", "default_format", "json_fields"):
        if not re.search(rf"^{key}\s*=", k, flags=re.MULTILINE):
            errors.append(f"configs/logging/runtime.toml missing key '{key}'")

if errors:
    print("logging-contract: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("logging-contract: OK")
PY
