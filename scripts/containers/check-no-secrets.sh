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
scan = []
scan.extend((root / "containers/apptainer").rglob("*.def"))
scan.extend((root / "containers/docker").rglob("Dockerfile.*"))

patterns = [
    re.compile(r"AKIA[0-9A-Z]{16}"),  # aws access key id
    re.compile(r"(?i)(secret|token|password)\s*[:=]\s*['\"]?[A-Za-z0-9_\\-]{8,}"),
    re.compile(r"ghp_[A-Za-z0-9]{20,}"),  # github token
    re.compile(r"github_pat_[A-Za-z0-9_]{20,}"),
    re.compile(r"xox[baprs]-[A-Za-z0-9-]{10,}"),  # slack tokens
    re.compile(r"AIza[0-9A-Za-z\\-_]{35}"),  # google api key
    re.compile(r"(?i)aws_secret_access_key\s*[:=]\s*['\"]?[A-Za-z0-9/+=]{30,}"),
    re.compile(r"(?i)-----BEGIN (?:RSA|OPENSSH|EC) PRIVATE KEY-----"),
]

errors = []
for p in sorted(scan):
    text = p.read_text(encoding="utf-8", errors="ignore")
    for i, line in enumerate(text.splitlines(), start=1):
        s = line.strip()
        if not s or s.startswith("#"):
            continue
        for rx in patterns:
            if rx.search(line):
                errors.append(f"{p.relative_to(root)}:{i}: potential secret pattern matched")
                break

if errors:
    print("container secret scan: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("container secret scan: OK")
PY
