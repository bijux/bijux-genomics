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

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

aliases_cfg = tomllib.loads(Path("configs/runtime/species_aliases.toml").read_text(encoding="utf-8"))
species_cfg = tomllib.loads(Path("configs/runtime/species.toml").read_text(encoding="utf-8"))

aliases = aliases_cfg.get("aliases", {})
default_builds = aliases_cfg.get("default_builds", {})
species_rows = species_cfg.get("species", [])

if not aliases:
    print("species-aliases: [aliases] table is empty", file=sys.stderr)
    sys.exit(1)

canonical = re.compile(r"^[A-Z][a-z]+ [a-z]+$")
bad = []
for alias, species in aliases.items():
    if alias != alias.lower():
        bad.append(f"alias {alias!r} must be lowercase")
    if not canonical.match(str(species)):
        bad.append(f"alias {alias!r} has non-canonical species id {species!r}; expected 'Genus species'")

authority_default_build = {}
authority_species = set()
for row in species_rows:
    sid = str(row.get("species_id", ""))
    bid = str(row.get("default_build_id", ""))
    if not sid or not bid:
        bad.append(f"species.toml row missing species_id/default_build_id: {row!r}")
        continue
    authority_default_build[sid] = bid
    authority_species.add(sid)

for species, build in default_builds.items():
    if species not in authority_default_build:
        bad.append(f"default_builds species {species!r} missing in species.toml authority")
        continue
    if str(build) != authority_default_build[species]:
        bad.append(
            f"default_builds mismatch for {species!r}: aliases={build!r}, species.toml={authority_default_build[species]!r}"
        )

for alias, species in aliases.items():
    if species not in authority_species:
        bad.append(f"alias {alias!r} points to undeclared species {species!r} in species.toml")

if bad:
    print("species-aliases: validation failed:", file=sys.stderr)
    for item in bad:
        print(f"  - {item}", file=sys.stderr)
    sys.exit(1)

print("species-aliases: OK")
PY
