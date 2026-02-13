#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

bad_files=$(find "$ROOT_DIR/containers" -type f \( -name '*.digest' -o -name '*digests*.json' -o -name '*.sha256' \) \
  ! -path "$ROOT_DIR/containers/versions/*" || true)
if [[ -n "${bad_files:-}" ]]; then
  echo "digest output policy failed: generated digest artifacts must not live under containers/ tree" >&2
  printf '%s\n' "$bad_files" >&2
  exit 1
fi

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import re
import sys

root = Path(sys.argv[1])
errors = []

# Disallow floating "latest" refs in container docs/locks.
scan_docs = [root / "containers/docs", root / "containers", root / "docs/30-operations"]
latest_pat = re.compile(r":[Ll][Aa][Tt][Ee][Ss][Tt]\b")
for base in scan_docs:
    if not base.exists():
        continue
    for p in base.rglob("*.md"):
        for i, line in enumerate(p.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
            if latest_pat.search(line):
                errors.append(f"{p.relative_to(root)}:{i}: floating ':latest' reference is forbidden")

lock = root / "containers/versions/lock.json"
if lock.exists():
    data = json.loads(lock.read_text(encoding="utf-8"))
    for row in data.get("items", []):
        tool = str(row.get("tool", "")).strip()
        status = str(row.get("status", "")).strip()
        digest = str(row.get("resolved_image_digest", "")).strip()
        if status == "production" and digest and not digest.startswith("sha256:"):
            errors.append(f"lock.json: {tool} resolved_image_digest must be sha256:* when present")

if errors:
    print("digest output policy failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("digest refs policy: OK")
PY

echo "digest output policy: OK"
