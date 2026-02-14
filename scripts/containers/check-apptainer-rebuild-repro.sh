#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool="${1:-}"
if [[ -z "$tool" || "$tool" == "--help" || "$tool" == "-h" ]]; then
  cat <<'USAGE'
Usage: scripts/containers/check-apptainer-rebuild-repro.sh <tool-id>
Builds one Apptainer def twice and requires identical SIF sha256.
USAGE
  [[ -n "$tool" ]] && exit 0
  exit 2
fi

if [[ -f "$ROOT_DIR/containers/apptainer/bijux/${tool}.def" ]]; then
  def="$ROOT_DIR/containers/apptainer/bijux/${tool}.def"
elif [[ -f "$ROOT_DIR/containers/apptainer/non-bijux/${tool}.def" ]]; then
  def="$ROOT_DIR/containers/apptainer/non-bijux/${tool}.def"
else
  echo "apptainer rebuild repro: skip (no def for $tool)"
  exit 0
fi

require_cmd apptainer
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
run1="$TMP_ROOT/${tool}.repro1.sif"
run2="$TMP_ROOT/${tool}.repro2.sif"
log1="$TMP_ROOT/${tool}.repro1.log"
log2="$TMP_ROOT/${tool}.repro2.log"

apptainer build --force "$run1" "$def" >"$log1" 2>&1
apptainer build --force "$run2" "$def" >"$log2" 2>&1

h1="$(shasum -a 256 "$run1" | awk '{print $1}')"
h2="$(shasum -a 256 "$run2" | awk '{print $1}')"

if [[ "$h1" != "$h2" ]]; then
  echo "apptainer rebuild repro: SIF hash mismatch for $tool" >&2
  echo "- run1: $h1" >&2
  echo "- run2: $h2" >&2
  exit 1
fi

echo "apptainer rebuild repro: OK ($tool)"
