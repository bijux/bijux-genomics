#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

tool=""
stage=""
reason=""
removal_after=""
today="$(date -u +%Y-%m-%d)"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --tool) tool="${2:-}"; shift ;;
    --stage) stage="${2:-}"; shift ;;
    --reason) reason="${2:-}"; shift ;;
    --removal-after) removal_after="${2:-}"; shift ;;
    --help|-h)
      cat <<'EOF'
Usage: scripts/containers/demote.sh --tool <id> --stage <domain.stage> --reason <text> --removal-after <YYYY-MM-DD>
EOF
      exit 0
      ;;
    *) echo "unknown arg: $1" >&2; exit 2 ;;
  esac
  shift
done

[[ -n "$tool" && -n "$stage" && -n "$reason" && -n "$removal_after" ]] || { echo "missing required args" >&2; exit 2; }

python3 - "$ROOT_DIR" "$tool" "$stage" "$reason" "$today" "$removal_after" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1]); tool=sys.argv[2]; stage=sys.argv[3]; reason=sys.argv[4]
deprecated_since=sys.argv[5]; removal_after=sys.argv[6]

reg_files = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
found = False
for path in reg_files:
    text = path.read_text(encoding="utf-8")
    chunks = text.split("[[tools]]")
    out = [chunks[0]]
    for chunk in chunks[1:]:
        block = "[[tools]]" + chunk
        if f'id = "{tool}"' in block or f'tool_id = "{tool}"' in block:
            lines = block.splitlines()
            for i, line in enumerate(lines):
                if line.strip().startswith("status = "):
                    lines[i] = 'status = "experimental"'
                    found = True
                    break
            block = "\n".join(lines)
        out.append(block)
    path.write_text("".join(out), encoding="utf-8")
if not found:
    raise SystemExit(f"tool not found in production registries: {tool}")

dep = root / "configs/ci/registry/deprecations.toml"
text = dep.read_text(encoding="utf-8").rstrip() + "\n\n"
text += "[[deprecations]]\n"
text += f'tool_id = "{tool}"\n'
text += f'stage = "{stage}"\n'
text += f'deprecated_since = "{deprecated_since}"\n'
text += f'removal_after = "{removal_after}"\n'
text += f'rationale = "{reason}"\n'
dep.write_text(text, encoding="utf-8")
PY

"$SCRIPT_DIR/generate-version-lock.sh"
"$ROOT_DIR/scripts/domain/lock-registry.sh"
echo "demoted $tool -> experimental and appended deprecation entry"
