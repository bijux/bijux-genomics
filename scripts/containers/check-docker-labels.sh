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

root = Path(sys.argv[1])
required = [
    "org.opencontainers.image.title",
    "org.opencontainers.image.version",
    "org.opencontainers.image.source",
    "org.opencontainers.image.licenses",
]

errors = []
for path in sorted((root / "containers/docker/arm64").glob("Dockerfile.*")):
    text = path.read_text(encoding="utf-8")
    missing = [k for k in required if k not in text]
    if missing:
        errors.append(f"{path.relative_to(root)} missing labels: {', '.join(missing)}")

if errors:
    print("docker label policy check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("docker label policy: OK")
PY
