#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
FILE="$ROOT_DIR/configs/vcf/deprecations/panels.toml"

usage() {
  cat <<USAGE
Usage: scripts/tooling/deprecate-vcf-panel.sh --panel <panel_id> --phase <warn|fail|remove> --replacement <panel_id> --rationale <text>
USAGE
}

panel=""; phase=""; replacement=""; rationale=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --panel) panel="$2"; shift 2 ;;
    --phase) phase="$2"; shift 2 ;;
    --replacement) replacement="$2"; shift 2 ;;
    --rationale) rationale="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -n "$panel" && -n "$phase" && -n "$replacement" && -n "$rationale" ]] || { usage >&2; exit 2; }
[[ "$phase" == "warn" || "$phase" == "fail" || "$phase" == "remove" ]] || { echo "phase must be warn|fail|remove" >&2; exit 2; }

python3 - "$FILE" "$panel" "$phase" "$replacement" "$rationale" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1])
panel, phase, replacement, rationale = sys.argv[2:]
text = path.read_text(encoding="utf-8")
if f'panel_id = "{panel}"' in text:
    raise SystemExit(f"deprecation already exists for panel {panel}")
append = (
    "\n[[deprecation]]\n"
    f'panel_id = "{panel}"\n'
    f'phase = "{phase}"\n'
    f'replacement = "{replacement}"\n'
    f'rationale = "{rationale}"\n'
)
path.write_text(text.rstrip() + "\n" + append, encoding="utf-8")
print(f"added panel deprecation {panel} ({phase})")
PY
