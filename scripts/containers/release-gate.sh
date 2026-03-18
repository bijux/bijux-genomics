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
cargo run -q -p bijux-dev-dna -- containers run check-smoke-contract
cargo run -q -p bijux-dev-dna -- containers run check-vcf-imputation-toolchain
cargo run -q -p bijux-dev-dna -- containers run check-smoke-inputs-policy
cargo run -q -p bijux-dev-dna -- containers run check-tool-invocation-normalization
cargo run -q -p bijux-dev-dna -- containers run check-smoke-contract-lock
cargo run -q -p bijux-dev-dna -- containers run check-imputation-release-smoke
cargo run -q -p bijux-dev-dna -- containers run check-imputation-cross-runtime-parity
cargo run -q -p bijux-dev-dna -- containers run check-qa-matrix-generated
"$SCRIPT_DIR/check-build-provenance.sh"
"$SCRIPT_DIR/check-digest-output-policy.sh"
cargo run -q -p bijux-dev-dna -- containers run check-network-disclosure
cargo run -q -p bijux-dev-dna -- containers run check-imputation-network-policy
cargo run -q -p bijux-dev-dna -- containers run check-runtime-downloads
cargo run -q -p bijux-dev-dna -- containers run check-sbom-artifacts
cargo run -q -p bijux-dev-dna -- containers run check-vuln-hook
cargo run -q -p bijux-dev-dna -- containers run check-vuln-allowlist
cargo run -q -p bijux-dev-dna -- containers run check-license-index-generated
cargo run -q -p bijux-dev-dna -- containers run check-license-metadata
cargo run -q -p bijux-dev-dna -- containers run check-owners
cargo run -q -p bijux-dev-dna -- containers run check-tool-id-contract
cargo run -q -p bijux-dev-dna -- containers run check-tool-docs-generated
cargo run -q -p bijux-dev-dna -- containers run check-time-locale-determinism
cargo run -q -p bijux-dev-dna -- containers run check-imputation-runtime-constraints
cargo run -q -p bijux-dev-dna -- containers run check-imputation-hardening
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
