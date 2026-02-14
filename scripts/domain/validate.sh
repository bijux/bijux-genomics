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
  echo "Usage: scripts/domain/validate.sh [--allow-non-isolate]" >&2
  exit 0
fi

allow_non_isolate=0
if [[ "${1:-}" == "--allow-non-isolate" ]]; then
  allow_non_isolate=1
  shift
fi

if ! ./bin/require-isolate >/dev/null 2>&1; then
  if [[ "$allow_non_isolate" -ne 1 ]]; then
    echo "domain validate must run inside isolate; use --allow-non-isolate to override" >&2
    exit 2
  fi
fi

./scripts/domain/check-domain-layout.sh
./scripts/domain/check-domain-schema.sh
./scripts/domain/check-domain-index.sh
./scripts/domain/check-ssot-authority.sh
./scripts/domain/check-shared-tools.sh
./scripts/domain/check-tool-container-parity.sh
./scripts/domain/check-default-settings-docs.sh
./scripts/domain/check-fixture-contracts.sh
./scripts/domain/check-orphan-files.sh
./scripts/domain/check-doc-links.sh
./scripts/domain/check-external-tool-policy.sh
./scripts/domain/check-inventory.sh

cargo run -p bijux-dna-domain-compiler --bin domain_validate -- --domain-dir "$repo_root/domain"
