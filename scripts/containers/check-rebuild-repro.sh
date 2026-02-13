#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool="${1:-}"
if [[ -z "$tool" || "$tool" == "--help" || "$tool" == "-h" ]]; then
  cat <<'EOF'
Usage: scripts/containers/check-rebuild-repro.sh <tool-id>
Rebuilds a single docker tool twice and compares:
- version output
- /opt/bijux/VERSION.json digest
EOF
  [[ -n "$tool" ]] && exit 0
  exit 2
fi

dockerfile="$ROOT_DIR/containers/docker/arm64/Dockerfile.${tool}"
if [[ ! -f "$dockerfile" ]]; then
  echo "rebuild-repro: skip (no dockerfile for $tool)"
  exit 0
fi

require_cmd docker
img1="bijux-repro/${tool}:run1"
img2="bijux-repro/${tool}:run2"

docker build --platform linux/arm64 -f "$dockerfile" -t "$img1" "$ROOT_DIR/containers/docker/arm64" >/dev/null
ver1="$(docker run --rm --entrypoint sh "$img1" -lc "$tool --version" 2>/dev/null | head -n1 || true)"
vf1="$(docker run --rm --entrypoint sh "$img1" -lc 'cat /opt/bijux/VERSION.json' 2>/dev/null | shasum -a 256 | awk '{print $1}')"

docker build --platform linux/arm64 -f "$dockerfile" -t "$img2" "$ROOT_DIR/containers/docker/arm64" >/dev/null
ver2="$(docker run --rm --entrypoint sh "$img2" -lc "$tool --version" 2>/dev/null | head -n1 || true)"
vf2="$(docker run --rm --entrypoint sh "$img2" -lc 'cat /opt/bijux/VERSION.json' 2>/dev/null | shasum -a 256 | awk '{print $1}')"

if [[ "$ver1" != "$ver2" ]]; then
  echo "rebuild-repro: version mismatch: '$ver1' vs '$ver2'" >&2
  exit 1
fi
if [[ "$vf1" != "$vf2" ]]; then
  echo "rebuild-repro: VERSION.json digest mismatch: '$vf1' vs '$vf2'" >&2
  exit 1
fi

echo "rebuild-repro: OK ($tool)"
