#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/artifacts/domain/inventory.json}"
ensure_artifacts_dir "$(dirname "$OUT")"
mkdir -p "$(dirname "$OUT")"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import json
import re
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])

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
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
