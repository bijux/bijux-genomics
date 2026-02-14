#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

REQ="$ROOT_DIR/configs/docs/requirements.txt"
LOCK="$ROOT_DIR/configs/docs/requirements.lock.txt"

[[ -f "$REQ" ]] || { echo "docs-req-lock: missing $REQ" >&2; exit 1; }
[[ -f "$LOCK" ]] || { echo "docs-req-lock: missing $LOCK" >&2; exit 1; }

tmp_root="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$tmp_root"
mkdir -p "$tmp_root"
tmp_req="$(mktemp "$tmp_root/tmp-docs-req.XXXXXX")"
tmp_lock="$(mktemp "$tmp_root/tmp-docs-lock.XXXXXX")"
trap 'rm -f "$tmp_req" "$tmp_lock"' EXIT

sed '/^\s*#/d;/^\s*$/d' "$REQ" | sort >"$tmp_req"
sed '/^\s*#/d;/^\s*$/d' "$LOCK" | sort >"$tmp_lock"

if ! diff -u "$tmp_req" "$tmp_lock" >/dev/null; then
  echo "docs-req-lock: requirements.lock.txt must match requirements.txt exactly (manual pip-compile style lock)" >&2
  diff -u "$tmp_req" "$tmp_lock" >&2 || true
  exit 1
fi

if rg -n '(^|[^=<>!~])[A-Za-z0-9_.-]+\s*(>=|<=|>|<|~=)' "$REQ" "$LOCK" >/dev/null 2>&1; then
  echo "docs-req-lock: floating/range versions are forbidden; use exact pins package==x.y.z" >&2
  exit 1
fi

if ! rg -n '^[A-Za-z0-9_.-]+==[0-9][A-Za-z0-9_.-]*$' "$REQ" >/dev/null 2>&1; then
  echo "docs-req-lock: requirements.txt must contain exact package==version pins only" >&2
  exit 1
fi

echo "docs-req-lock: OK"
