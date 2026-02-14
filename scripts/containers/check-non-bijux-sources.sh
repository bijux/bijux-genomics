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
non_bijux_dir = root / "containers/apptainer/non-bijux"
sources_doc = non_bijux_dir / "NON_BIJUX_SOURCES.md"

if not sources_doc.exists():
    print(f"missing required provenance index: {sources_doc}", file=sys.stderr)
    raise SystemExit(1)

defs = sorted(p.stem for p in non_bijux_dir.glob("*.def"))
text = sources_doc.read_text(encoding="utf-8")
rows = {}
for line in text.splitlines():
    m = re.match(
        r"\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*(.+?)\s*\|\s*(\S+)\s*\|\s*`([^`]+)`\s*\|\s*`([^`]+)`\s*\|\s*(.+?)\s*\|",
        line,
    )
    if not m:
        continue
    tool_id, def_path, why_non_bijux, upstream, license_field, checksum, patching_rules = m.groups()
    rows[tool_id] = (def_path, why_non_bijux, upstream, license_field, checksum, patching_rules)

errors = []
for tool_id in defs:
    if tool_id not in rows:
        errors.append(f"{tool_id}: missing row in NON_BIJUX_SOURCES.md")
        continue
    def_path, why_non_bijux, upstream, license_field, checksum, patching_rules = rows[tool_id]
    expected_path = f"containers/apptainer/non-bijux/{tool_id}.def"
    if def_path != expected_path:
        errors.append(f"{tool_id}: def path mismatch, expected {expected_path}, got {def_path}")
    if not upstream.startswith(("http://", "https://")):
        errors.append(f"{tool_id}: upstream_source must be URL")
    if not why_non_bijux.strip():
        errors.append(f"{tool_id}: why_non_bijux must be non-empty")
    if not license_field.strip():
        errors.append(f"{tool_id}: upstream_license must be non-empty")
    if not patching_rules.strip():
        errors.append(f"{tool_id}: patching_rules must be non-empty")
    if not checksum.startswith("sha256:"):
        errors.append(f"{tool_id}: upstream_checksum must start with sha256:")
    else:
        digest = checksum.split(":", 1)[1]
        if digest != "pending" and not re.fullmatch(r"[0-9a-f]{64}", digest):
            errors.append(f"{tool_id}: upstream_checksum must be sha256:<64hex> or sha256:pending")

for tool_id in rows:
    if tool_id not in defs:
        errors.append(f"{tool_id}: listed in NON_BIJUX_SOURCES.md but no .def exists")

if errors:
    print("non-bijux source coverage check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("non-bijux source coverage: OK")
PY
