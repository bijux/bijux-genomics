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
import json
import sys

root = Path(sys.argv[1])
lock = json.loads((root / "containers/versions/lock.json").read_text(encoding="utf-8"))
errors = []

required_top = [
    "schema_version", "source", "source_sha256", "build_date_utc",
    "builder_platform", "generator_script", "generator_sha256", "items"
]
for k in required_top:
    if k not in lock:
        errors.append(f"missing top-level key: {k}")

if lock.get("schema_version") != "bijux.container.version_lock.v3":
    errors.append("schema_version must be bijux.container.version_lock.v3")

items = lock.get("items")
if not isinstance(items, list) or not items:
    errors.append("items must be non-empty list")
else:
    seen = set()
    for i, row in enumerate(items):
        if not isinstance(row, dict):
            errors.append(f"items[{i}] must be object")
            continue
        for k in [
            "tool",
            "version",
            "status",
            "source",
            "entry_sha256",
            "resolved_image_digest",
            "resolved_sif_sha256",
            "sif_digest_sha256",
            "frontend_resolved_sif_sha256",
            "frontend_sif_digest_sha256",
            "frontend_smoke_version_output_sha256",
        ]:
            if k not in row:
                errors.append(f"items[{i}] missing key: {k}")
        tool = str(row.get("tool", "")).strip()
        if not tool:
            errors.append(f"items[{i}] has empty tool")
        elif tool in seen:
            errors.append(f"duplicate tool in lock items: {tool}")
        seen.add(tool)

if errors:
    print("lock schema: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("lock schema: OK")
PY
