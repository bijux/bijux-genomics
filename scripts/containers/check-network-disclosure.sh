#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

report="${ISO_ROOT:-$ROOT_DIR/artifacts}/containers/network_usage.json"
"$SCRIPT_DIR/generate-network-usage.sh" "$report" >/dev/null

if [[ ! -f "$ROOT_DIR/containers/NETWORK_USAGE.md" ]]; then
  echo "missing containers/NETWORK_USAGE.md" >&2
  exit 1
fi

if [[ "${1:-}" == "--offline" || "${BIJUX_OFFLINE:-0}" == "1" ]]; then
  python3 - "$report" <<'PY'
import json
import sys
from pathlib import Path

rep = Path(sys.argv[1])
data = json.loads(rep.read_text(encoding="utf-8"))
bad = [row["path"] for row in data.get("items", []) if row.get("network_required")]
if bad:
    print("offline mode blocked: network-required container recipes detected:", file=sys.stderr)
    for p in bad:
        print(f"- {p}", file=sys.stderr)
    raise SystemExit(1)
print("network disclosure/offline: OK")
PY
else
  echo "network disclosure: OK"
fi
