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
errors = []

for p in sorted((root / "containers/apptainer").rglob("*.def")):
    text = p.read_text(encoding="utf-8", errors="ignore")
    env = text.split("%environment", 1)[1].split("\n%", 1)[0] if "%environment" in text else ""
    if "TZ=UTC" not in env:
        errors.append(f"{p.relative_to(root)}: %environment must set TZ=UTC")
    if "LC_ALL=C" not in env:
        errors.append(f"{p.relative_to(root)}: %environment must set LC_ALL=C")

# Docker runtime determinism is enforced via smoke contract runner flags.
smoke_docker = (root / "scripts/containers/smoke-docker-arm64.sh").read_text(encoding="utf-8", errors="ignore")
for marker in ("-e TZ=UTC", "-e LC_ALL=C"):
    if marker not in smoke_docker:
        errors.append(f"scripts/containers/smoke-docker-arm64.sh must pass '{marker}' to docker run")

if errors:
    print("time/locale determinism: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("time/locale determinism: OK")
PY
