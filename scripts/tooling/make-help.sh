#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" || "${1:-}" == "--dry-run" || "${1:-}" == "--verbose" ]]; then
  cat <<'USAGE'
Usage: scripts/tooling/make-help.sh [--internal]
USAGE
  exit 0
fi

show_internal=0
if [[ "${1:-}" == "--internal" ]]; then
  show_internal=1
fi

python3 - "$ROOT_DIR" "$show_internal" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
show_internal = sys.argv[2] == "1"
readme = (root / "makefiles/README.md").read_text(encoding="utf-8").splitlines()

public = []
in_public = False
for line in readme:
    if line.strip() == "Public targets (stable contract):":
        in_public = True
        continue
    if in_public and line.startswith("- `") and "`" in line[3:]:
        target = line.split("`")[1]
        public.append(target)
        continue
    if in_public and line.strip() and not line.startswith("- "):
        break

print("Public make targets:\n")
for target in public:
    print(f"  {target:<22} from makefiles/README.md")

if show_internal:
    mk = (root / "makefiles/cargo.mk").read_text(encoding="utf-8").splitlines()
    internal = []
    for row in mk:
        m = re.match(r"^([_a-zA-Z0-9-]+):\s*##\s*(.+)$", row)
        if not m:
            continue
        name, desc = m.group(1), m.group(2)
        if name.startswith("_") or name in {"domain-validate", "examples-validate"}:
            internal.append((name, desc))
    if internal:
        print("\nInternal make targets:\n")
        for name, desc in internal:
            print(f"  {name:<22} {desc}")

print("\nSee makefiles/README.md for the public surface contract.")
PY
