#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
OUT="$ROOT_DIR/docs/50-reference/VCF_DOWNSTREAM_COMPATIBILITY_MATRIX.md"
python3 - "$ROOT_DIR" "$OUT" <<'PY'
import sys
from pathlib import Path
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib  # type: ignore[no-redef]
root = Path(sys.argv[1]); out = Path(sys.argv[2])
panels = tomllib.loads((root / "configs/vcf/panels/panels.toml").read_text(encoding="utf-8")).get("panel", [])
reg = tomllib.loads((root / "configs/ci/registry/tool_registry_vcf_downstream.toml").read_text(encoding="utf-8")).get("tools", [])
rows = []
for p in panels:
    species = p.get("species_id", "")
    build = p.get("build_id", "")
    panel_id = p.get("id", "")
    tags = set((p.get("compatibility", {}) or {}).get("tool_tags", []))
    for t in reg:
        tool = t.get("id", "")
        stage_ids = ", ".join(t.get("stage_ids", []))
        if tool in tags:
            rows.append((species, build, panel_id, tool, stage_ids))
rows.sort()
lines = [
    "# VCF Downstream Compatibility Matrix",
    "",
    "Generated from `configs/vcf/panels/panels.toml` and `configs/ci/registry/tool_registry_vcf_downstream.toml`.",
    "",
    "| species_id | build_id | panel_id | tool_id | stage_ids |",
    "|---|---|---|---|---|",
]
for r in rows:
    lines.append(f"| {r[0]} | {r[1]} | {r[2]} | {r[3]} | {r[4]} |")
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"wrote {out}")
PY
