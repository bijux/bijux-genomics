#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import re
import sys

root = Path(sys.argv[1])
errors = []

def scalar(text: str, key: str) -> str:
    quoted = re.search(rf'^{re.escape(key)}:\s*"([^"\n]+)"\s*$', text, flags=re.MULTILINE)
    if quoted:
        return quoted.group(1).strip()
    plain = re.search(rf'^{re.escape(key)}:\s*([^\n#]+)\s*$', text, flags=re.MULTILINE)
    return plain.group(1).strip() if plain else ""

for tool_path in sorted((root / "domain").glob("*/tools/*.yaml")):
    if tool_path.name == "_schema.yaml":
        continue
    text = tool_path.read_text(encoding="utf-8")
    status = scalar(text, "status")
    if status == "out_of_scope":
        continue
    tool_id = scalar(text, "tool_id")
    citation = scalar(text, "citation")
    # Existing domain schema uses `upstream` and `license`; treat them as
    # compatibility aliases for homepage and license-id gate semantics.
    homepage = scalar(text, "homepage") or scalar(text, "upstream")
    license_id = scalar(text, "license-id") or scalar(text, "license")

    if not tool_id:
        errors.append(f"{tool_path.relative_to(root)} missing tool_id")
    if not homepage:
        errors.append(f"{tool_path.relative_to(root)} missing homepage/upstream")
    if not citation:
        errors.append(f"{tool_path.relative_to(root)} missing citation")
    if not license_id:
        errors.append(f"{tool_path.relative_to(root)} missing license-id/license")

if errors:
    print("domain tool metadata check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("domain tool metadata: OK")
PY
