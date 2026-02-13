#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

docker_dir="${1:-$ROOT_DIR/artifacts/containers/docker-arm64}"
apptainer_dir="${2:-$ROOT_DIR/artifacts/containers/apptainer}"

if [[ ! -d "$docker_dir" || ! -d "$apptainer_dir" ]]; then
  if [[ -n "${CI:-}" ]]; then
    echo "cross-runtime smoke: missing runtime dirs docker='$docker_dir' apptainer='$apptainer_dir'" >&2
    exit 1
  fi
  echo "cross-runtime smoke: SKIP (missing runtime dirs)"
  exit 0
fi

PYTHONPATH="$ROOT_DIR/scripts/tooling/python${PYTHONPATH:+:$PYTHONPATH}" \
  python3 -m bijux_dna_tools.compare_container_runtimes "$docker_dir" "$apptainer_dir"
