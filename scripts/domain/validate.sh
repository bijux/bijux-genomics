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

./scripts/domain/check-domain-layout.sh
./scripts/domain/check-domain-schema.sh
./scripts/domain/check-domain-index.sh

if ! ./bin/require-isolate >/dev/null 2>&1; then
  exec ./bin/isolate "$0" "$@"
fi

cargo run -p bijux-dna-domain-compiler --bin domain_validate -- --domain-dir "$repo_root/domain"
