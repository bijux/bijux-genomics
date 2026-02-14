#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

SPEC="$ROOT_DIR/scripts/SUPPORTED.toml"
viol=()
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

while IFS= read -r rel; do
  [[ -n "$rel" ]] || continue
  [[ "$rel" == scripts/_lib/* ]] && continue
  abs="$ROOT_DIR/$rel"
  [[ -x "$abs" ]] || { viol+=("$rel not executable"); continue; }

  if ! "$abs" --help >/dev/null 2>&1; then
    viol+=("$rel: --help failed")
  fi
  if ! "$abs" --help 2>&1 | rg -q 'Usage:'; then
    viol+=("$rel: --help output missing 'Usage:'")
  fi
  if ! "$abs" --verbose --help >/dev/null 2>&1; then
    viol+=("$rel: --verbose --help failed")
  fi
  if ! "$abs" --dry-run --help >/dev/null 2>&1; then
    viol+=("$rel: --dry-run --help failed")
  fi
  set +e
  probe_dir="$(mktemp -d "$TMP_ROOT/script-interface-probe.XXXXXX")"
  (
    cd "$probe_dir"
    timeout 5s "$abs" --__bijux_invalid_flag__ >/dev/null 2>&1
  )
  rc=$?
  rm -rf "$probe_dir"
  set -e
  if capture_root_pollution; then
    viol+=("$rel: invalid-flag probe wrote --__bijux_invalid_flag__ at repo root")
  fi
done < <(awk '/^path = "/ {gsub(/^path = "/,""); gsub(/"$/,""); print}' "$SPEC")

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-script-interface: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-script-interface: OK"
