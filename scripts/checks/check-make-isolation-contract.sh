#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

mk="$ROOT_DIR/makefiles/cargo.mk"

viol=()

# Public gates must require isolate explicitly.
for t in fmt lint audit test coverage doctor; do
  block="$(awk -v target="$t" '
    $0 ~ "^"target":" {in_t=1; print; next}
    in_t && $0 ~ /^[^ \t].*:/ {exit}
    in_t {print}
  ' "$mk")"
  if ! grep -q './bin/require-isolate >/dev/null' <<<"$block"; then
    viol+=("make target '$t' must call ./bin/require-isolate >/dev/null")
  fi
done

# ARTIFACTS_DIR must be per invocation and isolate-aware under artifacts/isolate/<target>/<runid>.
art_line="$(rg -n '^ARTIFACTS_DIR \?=' "$mk" || true)"
if [[ -z "$art_line" ]]; then
  viol+=("makefiles/cargo.mk must define ARTIFACTS_DIR ?=")
else
  if ! grep -q 'MAKECMDGOALS' <<<"$art_line"; then
    viol+=("ARTIFACTS_DIR must include MAKECMDGOALS for per-target uniqueness")
  fi
  if ! grep -q 'ISO_ROOT' <<<"$art_line"; then
    viol+=("ARTIFACTS_DIR must include ISO_ROOT override for isolated outputs")
  fi
  if ! grep -q 'artifacts/isolate/' <<<"$art_line"; then
    viol+=("ARTIFACTS_DIR must be rooted under artifacts/isolate/")
  fi
  if ! grep -q 'ISO_RUN_ID' <<<"$art_line"; then
    viol+=("ARTIFACTS_DIR must include ISO_RUN_ID segment for run uniqueness")
  fi
fi

# _test-triage must use ARTIFACTS_DIR instead of hardcoded artifacts/.
triage_block="$(awk '
  /^_test-triage:/ {in_t=1; print; next}
  in_t && /^[^ \t].*:/ {exit}
  in_t {print}
' "$mk")"
if grep -q 'artifacts/' <<<"$triage_block"; then
  viol+=("_test-triage must not hardcode artifacts/ paths; use \$(ARTIFACTS_DIR)")
fi

if [[ ${#viol[@]} -gt 0 ]]; then
  echo "check-make-isolation-contract: violations found:" >&2
  printf '%s\n' "${viol[@]}" >&2
  exit 1
fi

echo "check-make-isolation-contract: OK"
