#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

CFG="$ROOT_DIR/configs/bench/knobs.toml"

get_default_value() {
  local key="$1"
  awk -v want="$key" '
    BEGIN {in_defaults=0}
    /^[[:space:]]*\[defaults\][[:space:]]*$/ {in_defaults=1; next}
    /^[[:space:]]*\[[^]]+\][[:space:]]*$/ {if (in_defaults) exit}
    {
      if (!in_defaults) next
      line=$0
      sub(/#.*/, "", line)
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", line)
      if (line == "") next
      split(line, kv, "=")
      k=kv[1]
      gsub(/^[[:space:]]+|[[:space:]]+$/, "", k)
      if (k == want) {
        v=substr(line, index(line, "=")+1)
        gsub(/^[[:space:]]+|[[:space:]]+$/, "", v)
        gsub(/^"|"$/, "", v)
        print v
        exit
      }
    }
  ' "$CFG"
}

require_key() {
  local key="$1"
  local val
  val="$(get_default_value "$key")"
  [[ -n "$val" ]] || { echo "bench-knobs: missing key in [defaults]: $key" >&2; exit 1; }
  printf '%s' "$val"
}

warmup_policy="$(require_key warmup_policy)"
repetitions="$(require_key repetitions)"
capture_cpu="$(require_key capture_cpu)"
capture_memory="$(require_key capture_memory)"
capture_io="$(require_key capture_io)"

case "$warmup_policy" in
  none|once|per-benchmark) ;;
  *) echo "bench-knobs: warmup_policy must be one of none|once|per-benchmark" >&2; exit 1 ;;
esac

if ! [[ "$repetitions" =~ ^[0-9]+$ ]]; then
  echo "bench-knobs: repetitions must be an integer" >&2
  exit 1
fi
if (( repetitions < 1 || repetitions > 100 )); then
  echo "bench-knobs: repetitions must be within [1, 100]" >&2
  exit 1
fi

for v in "$capture_cpu" "$capture_memory" "$capture_io"; do
  case "$v" in
    true|false) ;;
    *) echo "bench-knobs: capture toggles must be true|false" >&2; exit 1 ;;
  esac
done

echo "bench-knobs: OK"
