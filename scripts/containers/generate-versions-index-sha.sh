#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/versions/index.sha256}"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import hashlib
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
versions_dir = root / "containers/versions"
rows = []
for p in sorted(versions_dir.glob("*")):
    if not p.is_file():
        continue
    if p.name == "index.sha256":
        continue
    digest = hashlib.sha256(p.read_bytes()).hexdigest()
    rows.append((p.name, digest))

payload = "\n".join(f"{d}  {n}" for n, d in rows) + "\n"
out.write_text(payload, encoding="utf-8")
print(f"generated {out}")
PY
