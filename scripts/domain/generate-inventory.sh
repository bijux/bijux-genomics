#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_JSON="${1:-$ROOT_DIR/artifacts/domain/inventory.json}"
OUT_MD="${2:-$ROOT_DIR/artifacts/domain/inventory.md}"
ensure_artifacts_dir "$(dirname "$OUT_JSON")"
ensure_artifacts_dir "$(dirname "$OUT_MD")"
mkdir -p "$(dirname "$OUT_JSON")" "$(dirname "$OUT_MD")"

python3 - "$ROOT_DIR" "$OUT_JSON" "$OUT_MD" <<'PY'
from pathlib import Path
import json
import sys

root = Path(sys.argv[1])
out_json = Path(sys.argv[2])
out_md = Path(sys.argv[3])

rows = []
for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    dom = dom_dir.name
    stage_count = len([p for p in (dom_dir / "stages").glob("*.yaml") if p.name != "_schema.yaml"])
    tool_count = len([p for p in (dom_dir / "tools").glob("*.yaml") if p.name != "_schema.yaml"])
    fixture_stage_dirs = [p for p in (dom_dir / "fixtures").glob("*") if p.is_dir()]
    fixture_count = sum(1 for _ in (dom_dir / "fixtures").glob("*/*.txt"))
    rows.append(
        {
            "domain": dom,
            "stages": stage_count,
            "tools": tool_count,
            "fixture_stage_dirs": len(fixture_stage_dirs),
            "fixture_files": fixture_count,
            "has_artifacts_yaml": (dom_dir / "artifacts.yaml").exists(),
            "has_metrics_yaml": (dom_dir / "metrics.yaml").exists(),
            "has_default_settings_doc": (dom_dir / "docs" / "DEFAULT_SETTINGS.md").exists(),
        }
    )

payload = {"schema_version": "bijux.domain.inventory.v1", "domains": rows}
out_json.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

lines = [
    "# Domain Inventory",
    "",
    "| Domain | Stages | Tools | Fixture Stage Dirs | Fixture Files | artifacts.yaml | metrics.yaml | DEFAULT_SETTINGS.md |",
    "|---|---:|---:|---:|---:|:---:|:---:|:---:|",
]
for row in rows:
    lines.append(
        f"| {row['domain']} | {row['stages']} | {row['tools']} | {row['fixture_stage_dirs']} | {row['fixture_files']} | "
        f"{'yes' if row['has_artifacts_yaml'] else 'no'} | {'yes' if row['has_metrics_yaml'] else 'no'} | "
        f"{'yes' if row['has_default_settings_doc'] else 'no'} |"
    )
out_md.write_text("\n".join(lines) + "\n", encoding="utf-8")

print(f"generated {out_json}")
print(f"generated {out_md}")
PY
