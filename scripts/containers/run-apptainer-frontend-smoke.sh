#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

POLICY_TOML="$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml"
PROOF_ROOT="${PROOF_ROOT:-$ROOT_DIR/artifacts/containers/hpc/frontend-smoke}"
UPDATE_VERSION_LOCK="${UPDATE_VERSION_LOCK:-1}"

[[ -f "$POLICY_TOML" ]] || { echo "missing $POLICY_TOML" >&2; exit 1; }

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
    raise SystemExit(f"refusing frontend smoke on compute node host: {hn}")
PY

mkdir -p "$PROOF_ROOT"
rm -f "$PROOF_ROOT"/*.json 2>/dev/null || true

ARTIFACT_DIR="$PROOF_ROOT" \
SMOKE_LEVEL=contract \
FRONTEND_PROOF_MODE=1 \
"$SCRIPT_DIR/smoke-apptainer.sh"

MANIFEST_DIR="$PROOF_ROOT" "$SCRIPT_DIR/summary.sh" --json "$PROOF_ROOT/summary.json" >/dev/null
"$SCRIPT_DIR/check-apptainer-frontend-smoke-proof.sh" "$PROOF_ROOT"

if [[ "$UPDATE_VERSION_LOCK" == "1" ]]; then
  "$SCRIPT_DIR/generate-version-lock.sh"
fi

echo "frontend apptainer smoke: OK"
