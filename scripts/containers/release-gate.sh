#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
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

cargo run -q -p bijux-dev-dna -- containers run check-hpc-image-naming
cargo run -q -p bijux-dev-dna -- containers run check-toolkit-bundles
cargo run -q -p bijux-dev-dna -- containers run check-missing-images
cargo run -q -p bijux-dev-dna -- containers run check-tool-container-coverage
cargo run -q -p bijux-dev-dna -- containers run check-version-lock
cargo run -q -p bijux-dev-dna -- containers run check-version-authority
cargo run -q -p bijux-dev-dna -- containers run check-version-hash-pin
"$SCRIPT_DIR/check-lock-matches-built-output.sh"
"$SCRIPT_DIR/check-smoke-contract.sh"
"$SCRIPT_DIR/check-vcf-imputation-toolchain.sh"
"$SCRIPT_DIR/check-smoke-inputs-policy.sh"
"$SCRIPT_DIR/check-tool-invocation-normalization.sh"
"$SCRIPT_DIR/check-smoke-contract-lock.sh"
"$SCRIPT_DIR/check-imputation-release-smoke.sh"
"$SCRIPT_DIR/check-imputation-cross-runtime-parity.sh"
cargo run -q -p bijux-dev-dna -- containers run check-qa-matrix-generated
"$SCRIPT_DIR/check-build-provenance.sh"
"$SCRIPT_DIR/check-digest-output-policy.sh"
cargo run -q -p bijux-dev-dna -- containers run check-network-disclosure
"$SCRIPT_DIR/check-imputation-network-policy.sh"
"$SCRIPT_DIR/check-runtime-downloads.sh"
"$SCRIPT_DIR/check-sbom-artifacts.sh"
"$SCRIPT_DIR/check-vuln-hook.sh"
"$SCRIPT_DIR/check-vuln-allowlist.sh"
cargo run -q -p bijux-dev-dna -- containers run check-license-index-generated
cargo run -q -p bijux-dev-dna -- containers run check-license-metadata
cargo run -q -p bijux-dev-dna -- containers run check-owners
cargo run -q -p bijux-dev-dna -- containers run check-tool-id-contract
cargo run -q -p bijux-dev-dna -- containers run check-tool-docs-generated
"$SCRIPT_DIR/check-time-locale-determinism.sh"
"$SCRIPT_DIR/check-imputation-runtime-constraints.sh"
"$SCRIPT_DIR/check-imputation-hardening.sh"
"$SCRIPT_DIR/check-runtime-tool-digest-recording.sh"
cargo run -q -p bijux-dev-dna -- containers run check-apptainer-cache-policy
"$SCRIPT_DIR/check-hpc-frontend-policy-enforcement.sh"
cargo run -q -p bijux-dev-dna -- containers run check-apptainer-frontend-version-output-lock
cargo run -q -p bijux-dev-dna -- containers run check-apptainer-frontend-smoke-proof
cargo run -q -p bijux-dev-dna -- containers run check-apptainer-frontend-reproducibility
cargo run -q -p bijux-dev-dna -- containers run check-apptainer-frontend-security
"$SCRIPT_DIR/check-release-checklist.sh"
"$SCRIPT_DIR/container-doctor.sh" --strict

echo "container-release-gate: OK"
