#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool=""
to_status=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool) tool="${2:-}"; shift ;;
    --to) to_status="${2:-}"; shift ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/containers/promote.sh --tool <id> --to <experimental|production>
EOF
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

[[ -n "$tool" ]] || { echo "--tool required" >&2; exit 2; }
[[ "$to_status" == "experimental" || "$to_status" == "production" ]] || { echo "--to must be experimental|production" >&2; exit 2; }

python3 - "$ROOT_DIR" "$tool" <<'PY'
from pathlib import Path
import json
import sys
root = Path(sys.argv[1]); tool = sys.argv[2]
lock = root / "containers/versions/lock.json"
data = json.loads(lock.read_text(encoding="utf-8"))
known = {str(i.get("tool", "")) for i in data.get("items", [])}
if tool not in known:
    raise SystemExit(f"tool '{tool}' not present in containers/versions/lock.json; ad-hoc promotion is forbidden")
PY

python3 - "$ROOT_DIR" "$tool" "$to_status" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
tool = sys.argv[2]
to_status = sys.argv[3]
files = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]

updated = False
for path in files:
    text = path.read_text(encoding="utf-8")
    chunks = text.split("[[tools]]")
    head = chunks[0]
    out = [head]
    for chunk in chunks[1:]:
        block = "[[tools]]" + chunk
        if f'id = "{tool}"' in block or f'tool_id = "{tool}"' in block:
            lines = block.splitlines()
            for i, line in enumerate(lines):
                if line.strip().startswith("status = "):
                    lines[i] = f'status = "{to_status}"'
                    updated = True
                    break
            block = "\n".join(lines)
        out.append(block)
    path.write_text("".join(out), encoding="utf-8")

if not updated:
    raise SystemExit(f"tool not found: {tool}")
PY

python3 - "$ROOT_DIR" "$tool" "$to_status" <<'PY'
from pathlib import Path
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
tool = sys.argv[2]
to_status = sys.argv[3]
vp = root / "containers/versions/versions.toml"
data = tomllib.loads(vp.read_text(encoding="utf-8"))
if tool not in data:
    raise SystemExit(f"missing versions entry for {tool}")
entry = data[tool]
entry["status"] = to_status
lines = []
for tid in sorted(data.keys()):
    lines.append(f"[{tid}]")
    for k, v in data[tid].items():
        if isinstance(v, bool):
            vv = "true" if v else "false"
        else:
            vv = f"\"{v}\""
        lines.append(f"{k} = {vv}")
    lines.append("")
vp.write_text("\n".join(lines).rstrip() + "\n", encoding="utf-8")
PY

"$SCRIPT_DIR/generate-version-lock.sh"
"$ROOT_DIR/scripts/domain/lock-registry.sh"
echo "promoted $tool -> $to_status"
