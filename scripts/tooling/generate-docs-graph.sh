#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/docs/DOCS_GRAPH.toml}"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
docs = root / "docs"

lines = []
lines.append("# GENERATED FILE - DO NOT EDIT")
lines.append("# Regenerate with: scripts/tooling/generate-docs-graph.sh")
lines.append("")

for dirpath in sorted([docs] + [p for p in docs.rglob("*") if p.is_dir()]):
    idx = dirpath / "index.md"
    if not idx.exists():
        continue
    from_rel = idx.relative_to(root).as_posix()
    child_paths = []
    for f in sorted(dirpath.glob("*.md")):
        if f.name == "index.md":
            continue
        child_paths.append(f.relative_to(root).as_posix())
    for sub in sorted(p for p in dirpath.iterdir() if p.is_dir()):
        sub_idx = sub / "index.md"
        if sub_idx.exists():
            child_paths.append(sub_idx.relative_to(root).as_posix())

    lines.append("[[edge]]")
    lines.append(f'from = "{from_rel}"')
    lines.append("children = [")
    for c in child_paths:
        lines.append(f'  "{c}",')
    lines.append("]")
    lines.append("")

out.write_text("\n".join(lines), encoding="utf-8")
print(f"generated {out}")
PY
