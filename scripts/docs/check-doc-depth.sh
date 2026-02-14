#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
docs = root / "docs"

# Exclusions for generated/index/snapshot-like docs that are not narrative contracts.
exclude_suffixes = {
    "index.md",
    "DOCS_GRAPH.toml",
    "command_snapshot.txt",
    "release_help_snapshot.txt",
}

required = {
    "purpose": re.compile(r"^##\s+(Purpose|What)\s*$", re.IGNORECASE | re.MULTILINE),
    "scope": re.compile(r"^##\s+(Scope|Why)\s*$", re.IGNORECASE | re.MULTILINE),
    "non-goals": re.compile(r"^##\s+Non-goals\s*$", re.IGNORECASE | re.MULTILINE),
    "contracts": re.compile(r"^##\s+Contracts\s*$", re.IGNORECASE | re.MULTILINE),
}

violations = []
for p in sorted(docs.rglob("*.md")):
    name = p.name
    if name in exclude_suffixes or name.endswith(".generated.md"):
        continue
    if p.parts[-2:] == ("cli", "index.md"):
        continue
    text = p.read_text(encoding="utf-8")
    missing = [k for k, pat in required.items() if not pat.search(text)]
    if missing:
        violations.append((p.relative_to(root).as_posix(), missing))

if violations:
    print("doc-depth: missing required sections", file=sys.stderr)
    for rel, miss in violations:
        print(f"  - {rel}: {', '.join(miss)}", file=sys.stderr)
    sys.exit(1)

print("doc-depth: OK")
PY
