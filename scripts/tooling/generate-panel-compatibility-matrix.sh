#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/50-reference/PANEL_COMPATIBILITY_MATRIX.md}"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
panels = tomllib.loads((root / "configs/vcf/panels/panels.toml").read_text(encoding="utf-8")).get("panel", [])
maps = tomllib.loads((root / "configs/vcf/maps/maps.toml").read_text(encoding="utf-8")).get("map", [])

lines = [
    "<!-- GENERATED FILE - DO NOT EDIT -->",
    "<!-- Regenerate with: scripts/tooling/generate-panel-compatibility-matrix.sh -->",
    "",
    "# PANEL_COMPATIBILITY_MATRIX",
    "",
    "## Purpose",
    "Defines generated compatibility coverage for species/build, panel/map pairs, and downstream tool backends.",
    "",
    "## Scope",
    "Derived from panel and map catalogs to document declared tool-tag compatibility.",
    "",
    "## Non-goals",
    "- Replacing stage-level validation or runtime compatibility checks.",
    "",
    "## Contracts",
    "- Matrix rows are generated from catalog authority and must not be hand-edited.",
    "- Missing species/build map entries must be represented explicitly as unsupported rows.",
    "",
    "| Species | Build | Panel ID | Map ID | Tool Backend | Supported | Notes |",
    "|---|---|---|---|---|---|---|",
]

maps_by_sb = {}
for m in maps:
    key = (m.get("species_id", ""), m.get("build_id", ""))
    maps_by_sb.setdefault(key, []).append(m)

for panel in sorted(panels, key=lambda p: (p.get("species_id",""), p.get("build_id",""), p.get("id",""))):
    species = panel.get("species_id", "")
    build = panel.get("build_id", "")
    panel_id = panel.get("id", "")
    compat = panel.get("compatibility", {}) if isinstance(panel.get("compatibility", {}), dict) else {}
    tools = compat.get("tool_tags", []) if isinstance(compat.get("tool_tags", []), list) else []
    maps_for = maps_by_sb.get((species, build), [])
    if not maps_for:
        lines.append(f"| `{species}` | `{build}` | `{panel_id}` | `-` | `-` | `no` | no map catalog for species/build |")
        continue
    for m in maps_for:
        map_id = m.get("id", "")
        mcompat = m.get("compatibility", {}) if isinstance(m.get("compatibility", {}), dict) else {}
        map_tools = set(mcompat.get("tool_tags", []) if isinstance(mcompat.get("tool_tags", []), list) else [])
        for t in sorted(set(tools) | map_tools):
            ok = t in tools and t in map_tools
            notes = []
            if t == "minimac4":
                notes.append("requires panel m3vcf")
            if t == "glimpse":
                notes.append(f"GLIMPSE format={compat.get('glimpse_reference_format','')}")
            note = "; ".join(n for n in notes if n) or "-"
            lines.append(
                f"| `{species}` | `{build}` | `{panel_id}` | `{map_id}` | `{t}` | `{'yes' if ok else 'no'}` | {note} |"
            )

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
