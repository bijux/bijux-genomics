#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

if [[ "${1:-}" == "--help" || "${1:-}" == "-h" ]]; then
  cat <<'USAGE'
Usage: scripts/containers/check-hpc-image-naming.sh
USAGE
  exit 0
fi

"$SCRIPT_DIR/ensure-images.sh" --plan >/dev/null
python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import json
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
cfg = root / "configs/ci/tools/hpc_image_naming.toml"
report = root / "artifacts/containers/ensure-images/report.json"
if not cfg.exists():
    print("hpc image naming: missing config", file=sys.stderr)
    raise SystemExit(1)
if not report.exists():
    print("hpc image naming: missing ensure-images report", file=sys.stderr)
    raise SystemExit(1)

conf = tomllib.loads(cfg.read_text(encoding="utf-8"))
rep = json.loads(report.read_text(encoding="utf-8"))
prefix = str(conf.get("registry_prefix", "")).rstrip("/")
tool_re = re.compile(str(conf.get("tool_regex", "")))
ver_re = re.compile(str(conf.get("version_regex", "")))
tag_fmt = str(conf.get("tag_format", ""))
rows = rep.get("hpc_image_refs", [])
if not isinstance(rows, list):
    raise SystemExit("hpc image naming: report missing hpc_image_refs list")

errors = []
for row in rows:
    tool = str(row.get("tool", "")).strip()
    version = str(row.get("version", "")).strip()
    ref = str(row.get("hpc_image_ref", "")).strip()
    if not tool_re.fullmatch(tool):
        errors.append(f"{tool}: tool id does not match tool_regex")
    if not ver_re.fullmatch(version):
        errors.append(f"{tool}: version '{version}' does not match version_regex")
    expected_tag = tag_fmt.replace("{tool}", tool).replace("{version}", version)
    expected_ref = f"{prefix}/{tool}:{expected_tag}"
    if ref != expected_ref:
        errors.append(f"{tool}: hpc_image_ref mismatch, expected {expected_ref}, got {ref}")

if errors:
    print("hpc image naming: FAILED", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print(f"hpc image naming: OK ({len(rows)} refs)")
PY
