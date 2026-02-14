#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env
LC_ALL=C
export LC_ALL
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
OUT_DIR="${ROOT}/artifacts/assets-refresh/golden/toy-runs-v1"
TARGET_DIR="${ROOT}/assets/golden/toy-runs-v1"
REPORT_DIR="${ROOT}/artifacts/assets-refresh/golden"

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"
mkdir -p "${REPORT_DIR}"

"${ROOT}/scripts/test/toy_runs.sh" refresh --accept --profile all --out "${OUT_DIR}"

# Preserve generation provenance docs per golden bundle.
for bundle in "${OUT_DIR}"/*; do
  [[ -d "$bundle" ]] || continue
  cat > "$bundle/GENERATE.md" <<'MD'
# GENERATE

## Command(s)
Generated via `scripts/assets/refresh-golden.sh`.

## Tool versions
- `python3` and `shasum` versions are recorded in `artifacts/assets-refresh/golden/report.json`.

## Input origins
- Derived from repository mini reference toy runs (`scripts/test/toy_runs.sh refresh --accept --profile all`).

## Expected outputs
- `manifest.json`
- `metrics.json`
- `artifact_checksums.json`
- `report.html`
- `CHECKSUMS.sha256`
MD
  (
    cd "$bundle"
    shasum -a 256 artifact_checksums.json manifest.json metrics.json report.html GENERATE.md > CHECKSUMS.sha256
  )
done

python3 - "$OUT_DIR" "$REPORT_DIR/report.json" <<'PY'
import hashlib
import json
import subprocess
import sys
from pathlib import Path

out_dir = Path(sys.argv[1])
report_path = Path(sys.argv[2])
files = sorted([p for p in out_dir.rglob("*") if p.is_file()])
checksums = {}
for p in files:
    h = hashlib.sha256()
    h.update(p.read_bytes())
    checksums[p.relative_to(out_dir).as_posix()] = h.hexdigest()

tool_versions = {}
for cmd in [["python3", "--version"], ["shasum", "-a", "256", "--version"]]:
    name = cmd[0]
    try:
        out = subprocess.check_output(cmd, stderr=subprocess.STDOUT, text=True).strip().splitlines()[0]
    except Exception:
        out = "unknown"
    tool_versions[name] = out

report = {
    "schema_version": "bijux.assets.refresh_report.v1",
    "asset": "golden/toy-runs-v1",
    "generator_command": "scripts/assets/refresh-golden.sh",
    "inputs": list(checksums.keys()),
    "input_list": list(checksums.keys()),
    "output_checksums": checksums,
    "tool_versions": tool_versions,
    "checksums": checksums,
}
report_path.write_text(json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8")
print(f"wrote {report_path}")
PY

rm -rf "${TARGET_DIR}"
mkdir -p "$(dirname "${TARGET_DIR}")"
cp -R "${OUT_DIR}" "${TARGET_DIR}"
echo "golden refresh: wrote ${TARGET_DIR}"
