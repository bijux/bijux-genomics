#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
repo_root="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$repo_root"

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  echo "Usage: scripts/domain/validate.sh [--allow-non-artifacts]" >&2
  exit 0
fi

allow_non_artifacts=0
if [[ "${1:-}" == "--allow-non-artifacts" ]]; then
  allow_non_artifacts=1
  shift
fi

if [[ "$allow_non_artifacts" -ne 1 ]]; then
  require_artifact_env
fi

./scripts/domain/check-domain-layout.sh
./scripts/domain/check-domain-schema.sh
./scripts/domain/check-domain-index.sh
./scripts/domain/check-ssot-authority.sh
./scripts/domain/check-rust-stage-catalog-parity.sh
./scripts/domain/check-shared-tools.sh
./scripts/domain/check-tool-container-parity.sh
./scripts/domain/check-domain-tool-metadata.sh
./scripts/domain/check-planner-stage-coverage.sh
./scripts/domain/check-planner-fixture-coverage.sh
./scripts/domain/check-default-settings-docs.sh
./scripts/domain/check-fixture-contracts.sh
./scripts/domain/check-orphan-files.sh
./scripts/domain/check-doc-links.sh
./scripts/domain/check-external-tool-policy.sh
./scripts/domain/check-reference-bundle-lock.sh
./scripts/domain/check-inventory.sh

setup_artifact_env
cargo run -p bijux-dna-domain-compiler --bin domain_validate -- --domain-dir "$repo_root/domain"
