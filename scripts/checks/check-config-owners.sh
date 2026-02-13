#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OWNERS="$ROOT_DIR/configs/OWNERS.toml"
TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
RULES_TMP="$(mktemp "$TMP_ROOT/tmp-config-owners.XXXXXX")"
trap 'rm -f "$RULES_TMP"' EXIT

awk '
  BEGIN {in_rule=0; prefix=""; owner=""}
  /^[[:space:]]*\[\[rule\]\][[:space:]]*$/ {
    if (in_rule && prefix != "" && owner != "") print prefix "\t" owner
    in_rule=1; prefix=""; owner=""; next
  }
  {
    if (!in_rule) next
    line=$0
    sub(/#.*/, "", line)
    gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
    if (line == "") next
    if (line ~ /^prefix[[:space:]]*=/) {
      p=substr(line, index(line, "=")+1)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", p)
      gsub(/^"|"$/, "", p)
      prefix=p
    }
    if (line ~ /^owner[[:space:]]*=/) {
      o=substr(line, index(line, "=")+1)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", o)
      gsub(/^"|"$/, "", o)
      owner=o
    }
  }
  END {
    if (in_rule && prefix != "" && owner != "") print prefix "\t" owner
  }
' "$OWNERS" > "$RULES_TMP"

if [[ ! -s "$RULES_TMP" ]]; then
  echo "config-owners: configs/OWNERS.toml has no valid [[rule]] entries" >&2
  exit 1
fi

failed=0
while IFS= read -r f; do
  [[ -n "$f" ]] || continue
  rel="${f#$ROOT_DIR/}"
  matches=0
  while IFS= read -r rule; do
    prefix="${rule%%$'\t'*}"
    if [[ "$rel" == "$prefix"* ]]; then
      matches=$((matches + 1))
    fi
  done < "$RULES_TMP"

  if [[ $matches -ne 1 ]]; then
    echo "config-owners: $rel matched $matches owner rules (expected 1)" >&2
    failed=1
  fi
done < <(find "$ROOT_DIR/configs" -type f | sort)

if [[ $failed -ne 0 ]]; then
  exit 1
fi

echo "config-owners: OK"
