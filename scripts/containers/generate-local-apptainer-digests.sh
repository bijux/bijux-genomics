#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SIF_DIR="${SIF_DIR:-$ROOT_DIR/artifacts/containers/apptainer/sif}"
OUT="${1:-$ROOT_DIR/artifacts/containers/hpc/local-sif-digests.json}"

python3 - "$SIF_DIR" "$OUT" <<'PY'
from pathlib import Path
import hashlib
import json
import sys
sif_dir = Path(sys.argv[1])
out = Path(sys.argv[2])
rows = []
if sif_dir.exists():
    for p in sorted(sif_dir.glob("*.sif")):
        rows.append({
            "tool": p.stem,
            "sif_path": str(p),
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest(),
        })
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps({
    "schema_version": "bijux.local.sif_digests.v1",
    "items": rows,
}, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
PY
