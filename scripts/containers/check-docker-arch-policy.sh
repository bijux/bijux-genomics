#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

amd64_dir="$ROOT_DIR/containers/docker/amd64"
policy_doc="$ROOT_DIR/containers/docker/multiarch-policy.md"
if [[ ! -f "$policy_doc" ]]; then
  echo "docker arch policy: missing containers/docker/multiarch-policy.md" >&2
  exit 1
fi
if ! rg -q "arm64" "$policy_doc"; then
  echo "docker arch policy: policy doc must mention arm64 support contract" >&2
  exit 1
fi
for marker in "build strategy" "publish strategy" "promotion criteria"; do
  if ! rg -qi "$marker" "$policy_doc"; then
    echo "docker arch policy: policy doc missing required multiarch marker: $marker" >&2
    exit 1
  fi
done
for marker in "cross-build" "buildx" "naming convention" "amd64"; do
  if ! rg -qi "$marker" "$policy_doc"; then
    echo "docker arch policy: policy doc missing required amd64-plan marker: $marker" >&2
    exit 1
  fi
done
if [[ -d "$amd64_dir" ]]; then
  if find "$amd64_dir" -type f -name 'Dockerfile.*' | grep -q .; then
    echo "docker arch policy: amd64 Dockerfiles detected under containers/docker/amd64" >&2
    echo "This repo currently ships docker/arm64 definitions only by contract." >&2
    exit 1
  fi
fi

echo "docker arch policy: OK (arm64-only)"
