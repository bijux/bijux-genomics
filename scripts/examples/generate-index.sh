#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/examples/index.yaml}"
if [[ "${1:-}" == "--help" ]]; then
  echo "Usage: scripts/examples/generate-index.sh [output-path]"
  exit 0
fi

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
examples_root = root / "examples"

rows = []
for ex_toml in sorted(examples_root.glob("**/example.toml")):
    ex_dir = ex_toml.parent
    rel = ex_dir.relative_to(root).as_posix()
    if rel.startswith("examples/_template"):
        continue
    data = tomllib.loads(ex_toml.read_text(encoding="utf-8"))
    ex_id = str(data.get("id", ex_dir.name))
    domain = str(data.get("domain", "unknown"))
    corpus = str(data.get("corpus_required", "none"))
    outputs = data.get("expected_outputs", [])
    if not isinstance(outputs, list):
        outputs = []
    rows.append((ex_id, domain, corpus, [str(x) for x in outputs], rel))

lines = [
    "# GENERATED FILE - DO NOT EDIT",
    "# Regenerate with: scripts/examples/generate-index.sh",
    "examples:",
]
for ex_id, domain, corpus, outputs, rel in rows:
    lines.append(f"  - id: {ex_id}")
    lines.append(f"    domain: {domain}")
    lines.append(f"    corpus_required: {corpus}")
    lines.append("    expected_outputs:")
    if outputs:
        for o in outputs:
            lines.append(f"      - {o}")
    else:
        lines.append("      - none")
    lines.append(f"    path: {rel}")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
