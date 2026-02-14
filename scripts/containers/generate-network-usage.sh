#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/artifacts/containers/network_usage.json}"
ensure_artifacts_dir "$(dirname "$OUT")"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import json
import re
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
patterns = [r"\bcurl\b", r"\bwget\b", r"\bgit clone\b", r"\bapt-get\s+update\b"]
rx = re.compile("|".join(patterns), re.IGNORECASE)
items = []
for p in sorted((root / "containers").rglob("*")):
    if not p.is_file():
        continue
    if not (p.suffix == ".def" or p.name.startswith("Dockerfile.")):
        continue
    text = p.read_text(encoding="utf-8", errors="ignore")
    hits = [line.strip() for line in text.splitlines() if rx.search(line)]
    items.append({
        "path": str(p.relative_to(root)),
        "network_required": bool(hits),
        "commands": hits[:20],
    })

payload = {
    "schema_version": "bijux.container.network_usage.v1",
    "items": items,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
