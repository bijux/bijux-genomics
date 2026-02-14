#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'

ROOT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)
FILE="$ROOT_DIR/configs/vcf/deprecations/knobs.toml"

usage() {
  cat <<USAGE
Usage: scripts/vcf/deprecate-knob.sh --stage <stage_id> --knob <name> --phase <warn|fail|remove> --replacement <name> --rationale <text>
USAGE
}

stage=""; knob=""; phase=""; replacement=""; rationale=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --stage) stage="$2"; shift 2 ;;
    --knob) knob="$2"; shift 2 ;;
    --phase) phase="$2"; shift 2 ;;
    --replacement) replacement="$2"; shift 2 ;;
    --rationale) rationale="$2"; shift 2 ;;
    -h|--help) usage; exit 0 ;;
    *) echo "unknown arg: $1" >&2; usage >&2; exit 2 ;;
  esac
done

[[ -n "$stage" && -n "$knob" && -n "$phase" && -n "$replacement" && -n "$rationale" ]] || { usage >&2; exit 2; }
[[ "$phase" == "warn" || "$phase" == "fail" || "$phase" == "remove" ]] || { echo "phase must be warn|fail|remove" >&2; exit 2; }

python3 - "$FILE" "$stage" "$knob" "$phase" "$replacement" "$rationale" <<'PY'
import sys
from pathlib import Path
path = Path(sys.argv[1])
stage, knob, phase, replacement, rationale = sys.argv[2:]
text = path.read_text(encoding="utf-8")
if f'stage_id = "{stage}"\nknob = "{knob}"' in text:
    raise SystemExit(f"deprecation already exists for {stage}:{knob}")
append = (
    "\n[[deprecation]]\n"
    f'stage_id = "{stage}"\n'
    f'knob = "{knob}"\n'
    f'phase = "{phase}"\n'
    f'replacement = "{replacement}"\n'
    f'rationale = "{rationale}"\n'
)
path.write_text(text.rstrip() + "\n" + append, encoding="utf-8")
print(f"added knob deprecation {stage}:{knob} ({phase})")
PY
