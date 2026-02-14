#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_DIR="${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/doctor"
REPORT="${OUT_DIR}/report.json"
strict=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --strict) strict=1 ;;
    --help|-h)
      cat <<'USAGE'
Usage: scripts/containers/container-doctor.sh [--strict]
USAGE
      exit 0
      ;;
    *)
      echo "unknown arg: $1" >&2
      exit 2
      ;;
  esac
  shift
done

ensure_artifacts_dir "$OUT_DIR"
mkdir -p "$OUT_DIR"

run_check() {
  local id="$1"
  shift
  local status="ok"
  local out
  if out="$("$@" 2>&1)"; then
    status="ok"
  else
    status="fail"
  fi
  out="$(printf '%s' "$out" | tr '\n' ' ' | tr '\r' ' ' | sed -E 's/[[:space:]]+/ /g; s/^ //; s/ $//')"
  printf '{"id":"%s","status":"%s","detail":"%s"}\n' "$id" "$status" "$out"
  [[ "$status" == "ok" ]]
}

items=()
failed=0

for spec in \
  "missing_images $SCRIPT_DIR/check-missing-images.sh" \
  "lock_file_drift $SCRIPT_DIR/check-version-lock.sh" \
  "lock_vs_built $SCRIPT_DIR/check-lock-matches-built-output.sh" \
  "outdated_versions $SCRIPT_DIR/check-version-deprecations.sh" \
  "domain_parity $SCRIPT_DIR/check-tool-container-coverage.sh" \
  "registry_orphans $SCRIPT_DIR/check-registry-vs-defs.sh"; do
  id="${spec%% *}"
  cmd="${spec#* }"
  if line="$(run_check "$id" "$cmd")"; then
    items+=("$line")
  else
    items+=("$line")
    failed=1
  fi
done

{
  printf '{\n'
  printf '  "schema_version": "bijux.container.doctor.v1",\n'
  printf '  "strict": %s,\n' "$([[ "$strict" == "1" ]] && echo "true" || echo "false")"
  printf '  "items": [\n'
  for i in "${!items[@]}"; do
    if [[ "$i" -gt 0 ]]; then
      printf ',\n'
    fi
    printf '    %s' "${items[$i]}"
  done
  printf '\n  ]\n'
  printf '}\n'
} > "$REPORT"

echo "container-doctor: wrote $REPORT"
python3 - "$REPORT" <<'PY'
import json
import sys
d = json.load(open(sys.argv[1], "r", encoding="utf-8"))
for item in d["items"]:
    print(f'{item["id"]}: {item["status"]}')
PY

if [[ "$strict" == "1" && "$failed" == "1" ]]; then
  exit 1
fi
exit 0
