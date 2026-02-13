#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

FRONTEND_JSON="${1:-$ROOT_DIR/artifacts/containers/hpc/frontend-sif-digests.json}"
LOCAL_JSON="${2:-$ROOT_DIR/artifacts/containers/hpc/local-sif-digests.json}"
OUT_MD="${3:-$ROOT_DIR/artifacts/containers/hpc/frontend-local-diff.md}"

python3 - "$FRONTEND_JSON" "$LOCAL_JSON" "$OUT_MD" <<'PY'
from pathlib import Path
import json
import sys
f = json.loads(Path(sys.argv[1]).read_text(encoding="utf-8")) if Path(sys.argv[1]).exists() else {"items": []}
l = json.loads(Path(sys.argv[2]).read_text(encoding="utf-8")) if Path(sys.argv[2]).exists() else {"items": []}
out = Path(sys.argv[3])

fm = {str(i.get("tool", "")).strip(): str(i.get("sha256", "")).strip() for i in f.get("items", [])}
lm = {str(i.get("tool", "")).strip(): str(i.get("sha256", "")).strip() for i in l.get("items", [])}
shared = sorted(set(fm) & set(lm))
lines = [
    "# Frontend vs Local SIF Hash Diff",
    "",
    "| tool | frontend_sha256 | local_sha256 | match |",
    "|---|---|---|---|",
]
for t in shared:
    a = fm[t]; b = lm[t]
    lines.append(f"| `{t}` | `{a}` | `{b}` | `{'yes' if a == b else 'no'}` |")
missing_frontend = sorted(set(lm) - set(fm))
missing_local = sorted(set(fm) - set(lm))
if missing_frontend:
    lines += ["", "## Missing On Frontend", ""] + [f"- `{t}`" for t in missing_frontend]
if missing_local:
    lines += ["", "## Missing Locally", ""] + [f"- `{t}`" for t in missing_local]
mismatch = [t for t in shared if fm[t] != lm[t]]
if mismatch:
    lines += ["", "## Deterministic Causes To Document", "", "- base image digest drift", "- build timestamp embedded in image", "- tool download source changed", "- Apptainer/host version differences"]
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(out)
if mismatch:
    raise SystemExit(1)
PY
