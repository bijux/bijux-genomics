#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/versions/lock.json}"
ensure_artifacts_dir "$(dirname "$OUT")"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import hashlib
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
versions_path = root / "containers/versions/versions.toml"
versions = tomllib.loads(versions_path.read_text(encoding="utf-8"))

items = []
for tool in sorted(versions):
    row = versions[tool]
    canonical = json.dumps(row, sort_keys=True, separators=(",", ":"))
    items.append({
        "tool": tool,
        "entry_sha256": hashlib.sha256(canonical.encode("utf-8")).hexdigest(),
    })

payload = {
    "schema_version": "bijux.container.version_lock.v1",
    "source": "containers/versions/versions.toml",
    "source_sha256": hashlib.sha256(versions_path.read_bytes()).hexdigest(),
    "items": items,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
