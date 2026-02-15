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
planned = root / "containers/docs/PLANNED.md"
if not planned.exists():
    print("planned actionability: missing containers/docs/PLANNED.md", file=sys.stderr)
    raise SystemExit(1)

text = planned.read_text(encoding="utf-8")
errors = []
required_headers = ["| Tool |", "Owner"]
for h in required_headers:
    if h not in text:
        errors.append(f"PLANNED.md missing required column/header marker: {h}")

rows = []
in_table = False
for line in text.splitlines():
    if line.strip().startswith("| Tool ") and "Owner" in line:
        in_table = True
        continue
    if in_table and line.strip().startswith("|---"):
        continue
    if in_table and line.strip().startswith("|"):
        rows.append(line.strip())
    elif in_table and line.strip() == "":
        break

if not rows:
    errors.append("PLANNED.md has no actionable planned tool rows")

for row in rows:
    cols = [c.strip() for c in row.strip("|").split("|")]
    if len(cols) < 5:
        errors.append(f"PLANNED.md malformed row: {row}")
        continue
    tool = cols[0]
    owner = cols[4]
    if owner in {"", "-", "`-`", "`"`}:
        errors.append(f"{tool}: missing owner")

if errors:
    print("planned actionability: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print(f"planned actionability: OK ({len(rows)} planned rows)")
PY
