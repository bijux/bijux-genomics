#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_TXT="$ROOT_DIR/artifacts/configs_inventory.txt"
OUT_MD="$ROOT_DIR/artifacts/inventory/configs.md"
mkdir -p "$(dirname "$OUT_TXT")" "$(dirname "$OUT_MD")"

{
  echo "# schema_version = 1"
  echo "# owner = bijux-dna-infra"
  find "$ROOT_DIR/configs" -type f | sed "s#^$ROOT_DIR/##" | sort | while read -r rel; do
    printf '%s\n' "$rel"
  done
} > "$OUT_TXT"

python3 - "$ROOT_DIR" "$OUT_MD" <<'PY'
import sys
from pathlib import Path

root = Path(sys.argv[1])
out = Path(sys.argv[2])

rows = []
for p in sorted((root / "configs").rglob("*")):
    if not p.is_file():
        continue
    rel = p.relative_to(root).as_posix()
    schema = "-"
    owner = "-"
    try:
        lines = p.read_text(encoding="utf-8").splitlines()[:8]
    except UnicodeDecodeError:
        lines = []
    for line in lines:
        s = line.strip()
        if s.startswith("# schema_version = "):
            schema = s.split("=", 1)[1].strip()
        if s.startswith("# owner = "):
            owner = s.split("=", 1)[1].strip()
    rows.append((rel, schema, owner))

with out.open("w", encoding="utf-8") as fh:
    fh.write("# Config Inventory\n\n")
    fh.write("| Path | Schema Version | Owner |\n")
    fh.write("|---|---:|---|\n")
    for rel, schema, owner in rows:
        fh.write(f"| `{rel}` | `{schema}` | `{owner}` |\n")

print(f"wrote {out}")
PY

echo "wrote $OUT_TXT"
