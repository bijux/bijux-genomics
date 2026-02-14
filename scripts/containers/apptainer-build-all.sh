#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

POLICY_TOML="$ROOT_DIR/configs/ci/tools/hpc_frontend_build_policy.toml"
[[ -f "$POLICY_TOML" ]] || { echo "missing $POLICY_TOML" >&2; exit 1; }

host_name="$(hostname -f 2>/dev/null || hostname)"
python3 - "$POLICY_TOML" "$host_name" <<'PY'
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib
cfg = tomllib.load(open(sys.argv[1], "rb"))
hn = sys.argv[2]
compute_pat = str(cfg.get("compute_hostname_regex", "")).strip()
frontend_pat = str(cfg.get("frontend_hostname_regex", "")).strip()
if compute_pat and re.search(compute_pat, hn):
    raise SystemExit(f"refusing apptainer-build-all on compute node host: {hn}")
if frontend_pat and not re.search(frontend_pat, hn):
    raise SystemExit(f"refusing apptainer-build-all off frontend host: {hn}")
PY

ARTIFACT_DIR="${ARTIFACT_DIR:-$ROOT_DIR/artifacts/containers/hpc/frontend-smoke}"
mkdir -p "$ARTIFACT_DIR"

echo "apptainer-build-all: host=$host_name artifact_dir=$ARTIFACT_DIR"
ARTIFACT_DIR="$ARTIFACT_DIR" \
FRONTEND_PROOF_MODE=1 \
SMOKE_DISABLE_NETWORK=1 \
SMOKE_LEVEL=contract \
"$SCRIPT_DIR/smoke-apptainer.sh"

MANIFEST_DIR="$ARTIFACT_DIR" "$SCRIPT_DIR/summary.sh" --json "$ARTIFACT_DIR/summary.json" >/dev/null
"$SCRIPT_DIR/generate-version-lock.sh"
"$SCRIPT_DIR/check-smoke-contract-lock.sh"

echo "apptainer-build-all: OK"

