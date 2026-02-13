#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])

major_docs = [
    "docs/10-architecture/CONTRACT_SPINE.md",
    "docs/10-architecture/CONTRACT_AUTHORITY.md",
    "docs/10-architecture/CONTRACT_AUTHORITY_LADDER.md",
    "docs/20-science/SCIENTIFIC_DEFAULTS.md",
    "docs/30-operations/CI.md",
    "docs/30-operations/CONTAINERS.md",
    "docs/30-operations/REPRODUCIBILITY.md",
    "docs/30-operations/PRODUCTION_GUARANTEES.md",
    "docs/50-reference/TOOL_ADMISSION.md",
]

required = {
    "purpose": re.compile(r"^##\s+Purpose\s*$", re.IGNORECASE | re.MULTILINE),
    "scope": re.compile(r"^##\s+Scope\s*$", re.IGNORECASE | re.MULTILINE),
    "contracts": re.compile(r"^##\s+Contracts\s*$", re.IGNORECASE | re.MULTILINE),
    "examples": re.compile(r"^##\s+Examples\s*$", re.IGNORECASE | re.MULTILINE),
    "failure modes": re.compile(r"^##\s+Failure modes\s*$", re.IGNORECASE | re.MULTILINE),
}

errors: list[str] = []
for rel in major_docs:
    p = root / rel
    if not p.exists():
        errors.append(f"{rel}: missing major doc file")
        continue
    text = p.read_text(encoding="utf-8")
    missing = [name for name, pat in required.items() if not pat.search(text)]
    if missing:
        errors.append(f"{rel}: missing sections: {', '.join(missing)}")

if errors:
    print("doc-major-depth: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("doc-major-depth: OK")
PY
