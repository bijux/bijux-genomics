#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

stage_ids_file="$tmp/stage_ids.txt"
tool_ids_file="$tmp/tool_ids.txt"

rg -N --no-filename '^stage_id:\s*"' domain/fastq/stages domain/bam/stages \
  | sed -E 's/^stage_id:\s*"([^"]+)".*/\1/' \
  | sort -u > "$stage_ids_file"

rg -N --no-filename '^tool_id:\s*"' domain/fastq/tools domain/bam/tools \
  | sed -E 's/^tool_id:\s*"([^"]+)".*/\1/' \
  | sort -u > "$tool_ids_file"

unknown_stages=()
while IFS= read -r token; do
  [[ -z "$token" ]] && continue
  if ! grep -Fxq "$token" "$stage_ids_file"; then
    unknown_stages+=("$token")
  fi
done < <(
  rg -oN --no-filename '`(fastq|bam)\.[a-z0-9_]+' docs \
    | sed -E 's/^`//g' \
    | sort -u
)

unknown_tools=()
while IFS= read -r token; do
  [[ -z "$token" ]] && continue
  [[ "$token" == *"*" ]] && continue
  if ! grep -Fxq "$token" "$tool_ids_file"; then
    unknown_tools+=("$token")
  fi
done < <(
  rg -oN --no-filename '`tool:[a-z0-9][a-z0-9._-]*`' docs \
    | sed -E 's/^`tool:|`$//g' \
    | sort -u
)

if ((${#unknown_stages[@]} > 0)) || ((${#unknown_tools[@]} > 0)); then
  echo "docs reference unknown stage/tool ids"
  if ((${#unknown_stages[@]} > 0)); then
    echo "unknown stages:"
    printf '  %s\n' "${unknown_stages[@]}"
  fi
  if ((${#unknown_tools[@]} > 0)); then
    echo "unknown tools:"
    printf '  %s\n' "${unknown_tools[@]}"
  fi
  exit 1
fi

echo "docs stage/tool references validated"
