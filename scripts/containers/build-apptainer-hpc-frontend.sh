#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

POLICY_TOML="$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml"
CACHE_POLICY_TOML="$ROOT_DIR/configs/ci/tools/apptainer_cache_policy.toml"
OUT_DIR="${OUT_DIR:-$ROOT_DIR/artifacts/containers/hpc}"
VM_OUT_DIR="${VM_OUT_DIR:-$HOME/apptainer-build}"
COPY_BACK_DIR="${COPY_BACK_DIR:-$ROOT_DIR/artifacts/containers/apptainer}"
COMPARE_WITH_LOCAL="${COMPARE_WITH_LOCAL:-1}"
UPDATE_VERSION_LOCK="${UPDATE_VERSION_LOCK:-1}"

[[ -f "$POLICY_TOML" ]] || { echo "missing $POLICY_TOML" >&2; exit 1; }
[[ -f "$CACHE_POLICY_TOML" ]] || { echo "missing $CACHE_POLICY_TOML" >&2; exit 1; }

host_name="$(hostname -f 2>/dev/null || hostname)"
python3 - "$POLICY_TOML" "$host_name" <<'PY'
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
cfg = tomllib.loads(open(sys.argv[1], "rb").read())
hn = sys.argv[2]
pat = str(cfg.get("compute_hostname_regex", "")).strip()
if pat and re.search(pat, hn):
    raise SystemExit(f"refusing build on compute node host: {hn}")
PY

# Frontend builds require pinned versions only.
"$SCRIPT_DIR/check-version-hash-pin.sh"

ensure_artifacts_dir "$OUT_DIR"
mkdir -p "$OUT_DIR"

# Build bijux + non-bijux defs on frontend node.
"$SCRIPT_DIR/build-apptainer-all.sh" \
  --defs-dir "$ROOT_DIR/containers/apptainer/bijux" \
  --vm-out "$VM_OUT_DIR" \
  --copy-back "$COPY_BACK_DIR"
"$SCRIPT_DIR/build-apptainer-all.sh" \
  --defs-dir "$ROOT_DIR/containers/apptainer/non-bijux" \
  --vm-out "$VM_OUT_DIR" \
  --copy-back "$COPY_BACK_DIR"

frontend_json="$OUT_DIR/frontend-sif-digests.json"
python3 - "$COPY_BACK_DIR/sif" "$frontend_json" "$host_name" <<'PY'
from pathlib import Path
import hashlib
import json
import sys

sif_dir = Path(sys.argv[1])
out = Path(sys.argv[2])
host = sys.argv[3]
rows = []
for p in sorted(sif_dir.glob("*.sif")):
    h = hashlib.sha256(p.read_bytes()).hexdigest()
    rows.append({"tool": p.stem, "sif_path": str(p), "sha256": h})
payload = {
    "schema_version": "bijux.hpc.frontend_sif_digests.v1",
    "host": host,
    "items": rows,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(out)
PY

if [[ "$UPDATE_VERSION_LOCK" == "1" ]]; then
  "$SCRIPT_DIR/generate-version-lock.sh" "$ROOT_DIR/containers/versions/lock.json"
fi

if [[ "$COMPARE_WITH_LOCAL" == "1" ]]; then
  "$SCRIPT_DIR/generate-local-apptainer-digests.sh" "$OUT_DIR/local-sif-digests.json"
  "$SCRIPT_DIR/compare-frontend-local-sif-hash.sh" \
    "$OUT_DIR/frontend-sif-digests.json" \
    "$OUT_DIR/local-sif-digests.json" \
    "$OUT_DIR/frontend-local-diff.md"
fi

echo "hpc frontend apptainer build: OK"
