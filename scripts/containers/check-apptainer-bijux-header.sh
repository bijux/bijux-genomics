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
expected = [
    "# Container definition license: GPL-3.0.",
    "# This container definition is part of bijux-dna.",
    "# The bijux-dna software source code is licensed under Apache-2.0.",
    "# Copyright (C) 2026 Bijan Mousavi",
]

errors = []
for path in sorted((root / "containers/apptainer/bijux").glob("*.def")):
    lines = path.read_text(encoding="utf-8").splitlines()
    head = lines[:4]
    if head != expected:
        errors.append(str(path.relative_to(root)))

if errors:
    print("apptainer bijux header check failed (first 4 lines must match policy):", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("apptainer bijux headers: OK")
PY
