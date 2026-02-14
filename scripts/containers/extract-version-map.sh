#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/artifacts/containers/version_map.json}"
ensure_artifacts_dir "$(dirname "$OUT")"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))

items = []
for tool in sorted(versions.keys()):
    row = versions[tool]
    items.append({
        "tool": tool,
        "version": str(row.get("version", "")),
        "status": str(row.get("status", "production")),
        "source": str(row.get("source", "")),
        "source_sha256": str(row.get("source_sha256", "")),
        "pinned_commit": str(row.get("pinned_commit", "")),
        "date_pinned": str(row.get("date_pinned", "")),
    })

payload = {"schema_version": "bijux.container.version_map.v1", "source": "containers/versions/versions.toml", "items": items}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
