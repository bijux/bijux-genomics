#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - <<'PY'
from pathlib import Path
import re
import sys

cfg = Path("configs/runtime/species_aliases.toml")
text = cfg.read_text(encoding="utf-8")

in_aliases = False
seen = {}
dups = []
aliases = {}
for line in text.splitlines():
    s = line.strip()
    if not s or s.startswith("#"):
        continue
    if s.startswith("["):
        in_aliases = (s == "[aliases]")
        continue
    if not in_aliases:
        continue
    if "=" not in s:
        continue
    key, val = [p.strip() for p in s.split("=", 1)]
    key = key.strip('"')
    if key in seen:
        dups.append(key)
    seen[key] = True
    val = val.strip().strip('"')
    aliases[key] = val

if dups:
    print("species-aliases: duplicate aliases found:", ", ".join(sorted(set(dups))), file=sys.stderr)
    sys.exit(1)

if not aliases:
    print("species-aliases: [aliases] table is empty", file=sys.stderr)
    sys.exit(1)

canonical = re.compile(r"^[A-Z][a-z]+ [a-z]+$")
bad = []
for alias, species in aliases.items():
    if alias != alias.lower():
        bad.append(f"alias {alias!r} must be lowercase")
    if not canonical.match(species):
        bad.append(f"alias {alias!r} has non-canonical species id {species!r}; expected 'Genus species'")

if bad:
    print("species-aliases: canonical validation failed:", file=sys.stderr)
    for item in bad:
        print(f"  - {item}", file=sys.stderr)
    sys.exit(1)

print("species-aliases: OK")
PY
