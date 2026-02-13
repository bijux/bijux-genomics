#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: scripts/containers/release-gate.sh
USAGE
  exit 0
fi

cat <<'INFO'
container-release-gate: running required pre-release checks
INFO

"$SCRIPT_DIR/check-hpc-image-naming.sh"
"$SCRIPT_DIR/check-toolkit-bundles.sh"
"$SCRIPT_DIR/check-missing-images.sh"
"$SCRIPT_DIR/check-tool-container-coverage.sh"
"$SCRIPT_DIR/check-version-lock.sh"
"$SCRIPT_DIR/check-version-authority.sh"
"$SCRIPT_DIR/check-version-hash-pin.sh"
"$SCRIPT_DIR/check-lock-matches-built-output.sh"
"$SCRIPT_DIR/check-smoke-contract.sh"
"$SCRIPT_DIR/check-build-provenance.sh"
"$SCRIPT_DIR/check-network-disclosure.sh"
"$SCRIPT_DIR/check-sbom-artifacts.sh"
"$SCRIPT_DIR/check-vuln-hook.sh"
"$SCRIPT_DIR/container-doctor.sh" --strict

echo "container-release-gate: OK"
