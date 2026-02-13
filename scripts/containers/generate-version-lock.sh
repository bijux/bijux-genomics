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
import subprocess
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
version_map_path = Path(sys.argv[3])
versions_path = root / "containers/versions/versions.toml"
generator_path = root / "scripts/containers/generate-version-lock.sh"
version_map = json.loads(version_map_path.read_text(encoding="utf-8"))

items = []
manifest_candidates = [
    root / "artifacts" / "containers",
    root / "artifacts" / "containers" / "manifests",
]
docker_digest_by_tool = {}
apptainer_sif_sha256_by_tool = {}
frontend_sif_sha256_by_tool = {}
size_by_tool = {}
seen = set()
for base in manifest_candidates:
    if not base.exists():
        continue
    for p in sorted(base.glob("*.json")):
        if p.name in {"lock.json", "summary.json", "report.json"}:
            continue
        if p in seen:
            continue
        seen.add(p)
        try:
            m = json.loads(p.read_text(encoding="utf-8"))
        except Exception:
            continue
        t = str(m.get("tool", "")).strip()
        runtime = str(m.get("runtime", "")).strip()
        d = str(m.get("resolved_image_digest", "")).strip()
        s = m.get("image_size_bytes", 0)
        if not t:
            continue
        if runtime.startswith("docker"):
            docker_digest_by_tool[t] = d
        elif runtime == "apptainer":
            apptainer_sif_sha256_by_tool[t] = d
        # keep most recent non-zero size if available
        try:
            size = int(s)
        except Exception:
            size = 0
        if size > 0:
            size_by_tool[t] = size

frontend_digests = root / "artifacts" / "containers" / "hpc" / "frontend-sif-digests.json"
if frontend_digests.exists():
    try:
        payload = json.loads(frontend_digests.read_text(encoding="utf-8"))
        for row in payload.get("items", []):
            tool = str(row.get("tool", "")).strip()
            sha = str(row.get("sha256", "")).strip()
            if tool and sha:
                frontend_sif_sha256_by_tool[tool] = sha
    except Exception:
        pass
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
        "resolved_image_digest": str(docker_digest_by_tool.get(tool, "")),
        "resolved_sif_sha256": str(apptainer_sif_sha256_by_tool.get(tool, "")),
        "frontend_resolved_sif_sha256": str(frontend_sif_sha256_by_tool.get(tool, "")),
        "image_size_bytes": int(size_by_tool.get(tool, 0)),
        "entry_sha256": hashlib.sha256(canonical.encode("utf-8")).hexdigest(),
    })

payload = {
    "schema_version": "bijux.container.version_lock.v3",
    "source": "containers/versions/versions.toml",
    "version_map_source": "artifacts/containers/version_map.json",
    "build_manifests_source": "artifacts/containers/manifests/*.json",
    "build_date_utc": "",
    "builder_platform": "arm64",
    "generator_script": "scripts/containers/generate-version-lock.sh",
    "generator_sha256": hashlib.sha256(generator_path.read_bytes()).hexdigest(),
    "source_sha256": hashlib.sha256(versions_path.read_bytes()).hexdigest(),
    "items": items,
}
ts = ""
try:
    proc = subprocess.run(
        ["git", "-C", str(root), "log", "-1", "--format=%cI", "--", "containers/versions/versions.toml"],
        capture_output=True, text=True, check=False
    )
    ts = (proc.stdout or "").strip()
except Exception:
    ts = ""
payload["build_date_utc"] = ts or "1970-01-01T00:00:00Z"
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
