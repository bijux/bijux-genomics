#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

out="${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/vuln_scan_report.json"
"$SCRIPT_DIR/vuln-scan-hook.sh" "${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/sbom" "$out" >/dev/null
if [[ ! -f "$out" ]]; then
  echo "vuln hook: missing report $out" >&2
  exit 1
fi
echo "vuln hook: OK"
