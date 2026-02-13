#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

amd64_dir="$ROOT_DIR/containers/docker/amd64"
if [[ -d "$amd64_dir" ]]; then
  if find "$amd64_dir" -type f -name 'Dockerfile.*' | grep -q .; then
    echo "docker arch policy: amd64 Dockerfiles detected under containers/docker/amd64" >&2
    echo "This repo currently ships docker/arm64 definitions only by contract." >&2
    exit 1
  fi
fi

echo "docker arch policy: OK (arm64-only)"
