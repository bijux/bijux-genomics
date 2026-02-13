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

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
doc = (root / "containers/NETWORK_USAGE.md").read_text(encoding="utf-8")
tool_ids = []
for raw in (root / "containers/TOOL_IDS.txt").read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    tool_ids.append(line.split("\t", 1)[0])

errors = []
runtime_network_true = []
for tid in sorted(tool_ids):
    meta = root / "containers/network" / f"{tid}.network.toml"
    if not meta.exists():
        errors.append(f"missing per-tool network metadata: {meta.relative_to(root)}")
        continue
    data = tomllib.loads(meta.read_text(encoding="utf-8"))
    for key in ("tool_id", "runtime_network", "build_network", "notes"):
        if key not in data:
            errors.append(f"{meta.relative_to(root)} missing key '{key}'")
    if str(data.get("tool_id", "")).strip() != tid:
        errors.append(f"{meta.relative_to(root)} tool_id mismatch")
    if bool(data.get("runtime_network", False)):
        runtime_network_true.append(tid)

for tid in runtime_network_true:
    if re.search(rf"`{re.escape(tid)}`", doc) is None:
        errors.append(f"containers/NETWORK_USAGE.md must list runtime-network tool `{tid}`")

if errors:
    print("network disclosure metadata check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("network disclosure metadata: OK")
PY

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
