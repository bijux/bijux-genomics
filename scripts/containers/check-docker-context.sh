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
scan_files = list((root / "scripts").rglob("*.sh")) + list((root / "makefiles").glob("*.mk"))
errors = []
for path in scan_files:
    rel = path.relative_to(root)
    for i, line in enumerate(path.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        s = line.strip()
        if "docker build" not in s:
            continue
        # Reject implicit/explicit repo-root context usage.
        if re.search(r"\bdocker\s+build(\s+|$)", s):
            if re.search(r"\bdocker\s+build\b.*\s\.\s*$", s) or s.endswith("docker build") or s.endswith("docker build ."):
                errors.append(f"{rel}:{i}: docker build must not use repo-root context '.'")
            if "-f containers/docker/" in s and " containers/docker/" not in s:
                errors.append(f"{rel}:{i}: docker build should use containers/docker/<arch> as context")

if errors:
    print("docker context check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("docker context policy: OK")
PY
