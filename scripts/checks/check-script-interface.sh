#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
viol_file="$(mktemp "${TMPDIR:-/tmp}/check-script-interface.viol.XXXXXX")"
trap 'rm -f "$viol_file"' EXIT
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
pollution_dir="$ROOT_DIR/artifacts/containers/smoke/pollution"
pollution_file="$ROOT_DIR/--__bijux_invalid_flag__"
timestamp="$(date -u +%Y%m%dT%H%M%SZ)"

usage() {
  cat <<'USAGE'
Usage: scripts/checks/check-script-interface.sh
USAGE
}

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  usage
  exit 0
fi

if [[ $# -gt 0 ]]; then
  echo "unknown argument: $1" >&2
  usage >&2
  exit 2
fi

capture_root_pollution() {
  if [[ -f "$pollution_file" ]]; then
    mkdir -p "$pollution_dir"
    mv "$pollution_file" "$pollution_dir/--__bijux_invalid_flag__.check-script-interface.${timestamp}.$$"
    return 0
  fi
  return 1
}

capture_root_pollution || true

run_help_probe() {
  local script_path="$1"
  shift
  timeout 8s "$script_path" "$@"
}

check_help_contract() {
  local rel="$1"
  [[ -n "$rel" ]] || return 0
  [[ "$rel" == scripts/_lib/* ]] && return 0
  abs="$ROOT_DIR/$rel"
  [[ -x "$abs" ]] || { printf '%s\n' "$rel not executable" >>"$viol_file"; return 0; }

  help_output=""
  if ! help_output="$(run_help_probe "$abs" --help 2>&1)"; then
    printf '%s\n' "$rel: --help failed" >>"$viol_file"
  fi
  if ! printf '%s\n' "$help_output" | rg -q 'Usage:'; then
    printf '%s\n' "$rel: --help output missing 'Usage:'" >>"$viol_file"
  fi
  if printf '%s\n' "$help_output" | rg -q -- '--verbose'; then
    if ! run_help_probe "$abs" --verbose --help >/dev/null 2>&1; then
      printf '%s\n' "$rel: --verbose --help failed" >>"$viol_file"
    fi
  fi
  if printf '%s\n' "$help_output" | rg -q -- '--dry-run'; then
    if ! run_help_probe "$abs" --dry-run --help >/dev/null 2>&1; then
      printf '%s\n' "$rel: --dry-run --help failed" >>"$viol_file"
    fi
  fi
}

jobs="${CHECK_SCRIPT_INTERFACE_JOBS:-8}"
export ROOT_DIR SPEC viol_file
export -f run_help_probe check_help_contract
awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC" \
  | xargs -P "$jobs" -I{} bash -lc 'check_help_contract "$@"' _ {}

while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  abs="$ROOT_DIR/$rel"
  [[ -x "$abs" ]] || continue
  set +e
  probe_dir="$(mktemp -d "$TMP_ROOT/script-interface-probe.XXXXXX")"
  (
    cd "$probe_dir"
    timeout 1s "$abs" --__bijux_invalid_flag__ >/dev/null 2>&1
  )
  rm -rf "$probe_dir"
  set -e
  if capture_root_pollution; then
    printf '%s\n' "$rel: invalid-flag probe wrote --__bijux_invalid_flag__ at repo root" >>"$viol_file"
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ -s "$viol_file" ]]; then
  echo "check-script-interface: violations found:" >&2
  sort -u "$viol_file" >&2
  exit 1
fi

echo "check-script-interface: OK"
