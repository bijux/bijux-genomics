#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/versions/lock.json}"
case "$OUT" in
  "$ROOT_DIR"/containers/versions/*) ;;
  *) ensure_artifacts_dir "$(dirname "$OUT")" ;;
esac

TMP_ROOT="${ISO_ROOT:-$ROOT_DIR/artifacts/tmp}"
ensure_artifacts_dir "$TMP_ROOT"
mkdir -p "$TMP_ROOT"
version_map="$TMP_ROOT/version-map.lock.$$.json"
trap 'rm -f "$version_map"' EXIT
"$SCRIPT_DIR/extract-version-map.sh" "$version_map" >/dev/null

python3 - "$ROOT_DIR" "$OUT" "$version_map" <<'PY'
from pathlib import Path
import hashlib
import json
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
version_map_path = Path(sys.argv[3])
versions_path = root / "containers/versions/versions.toml"
version_map = json.loads(version_map_path.read_text(encoding="utf-8"))

items = []
manifest_dir = root / "artifacts" / "containers" / "manifests"
digest_by_tool = {}
size_by_tool = {}
if manifest_dir.exists():
    for p in sorted(manifest_dir.glob("*.json")):
        try:
            m = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        t = str(m.get("tool", "")).strip()
        d = str(m.get("resolved_image_digest", "")).strip()
        s = m.get("image_size_bytes", 0)
        if t:
            digest_by_tool[t] = d
            try:
                size_by_tool[t] = int(s)
            except Exception:
                size_by_tool[t] = 0
for row in version_map.get("items", []):
    tool = row.get("tool")
    canonical = json.dumps(row, sort_keys=True, separators=(",", ":"))
    items.append({
        "tool": tool,
        "version": str(row.get("version", "")),
        "status": str(row.get("status", "")),
        "source": str(row.get("source", "")),
        "source_sha256": str(row.get("source_sha256", "")),
        "pinned_commit": str(row.get("pinned_commit", "")),
        "resolved_image_digest": str(digest_by_tool.get(tool, "")),
        "image_size_bytes": int(size_by_tool.get(tool, 0)),
        "entry_sha256": hashlib.sha256(canonical.encode("utf-8")).hexdigest(),
    })

payload = {
    "schema_version": "bijux.container.version_lock.v2",
    "source": "containers/versions/versions.toml",
    "version_map_source": "artifacts/containers/version_map.json",
    "build_manifests_source": "artifacts/containers/manifests/*.json",
    "source_sha256": hashlib.sha256(versions_path.read_bytes()).hexdigest(),
    "items": items,
}
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
