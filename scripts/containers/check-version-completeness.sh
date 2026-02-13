#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

python3 - "$ROOT_DIR" <<'PY'
from pathlib import Path
import sys

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
versions_path = root / "containers/versions/versions.toml"
versions = tomllib.loads(versions_path.read_text(encoding="utf-8"))
known = set(versions.keys())

container_tools = set()
for path in (root / "containers/docker/arm64").glob("Dockerfile.*"):
    container_tools.add(path.name.split("Dockerfile.", 1)[1])
for path in (root / "containers/apptainer/bijux").glob("*.def"):
    container_tools.add(path.stem)
for path in (root / "containers/apptainer/non-bijux").glob("*.def"):
    container_tools.add(path.stem)

missing = sorted(tool for tool in container_tools if tool not in known)
if missing:
    print("container versions completeness check failed:", file=sys.stderr)
    for tool in missing:
        print(f"- missing {tool} in containers/versions/versions.toml", file=sys.stderr)
    raise SystemExit(1)

print("container versions completeness: OK")
PY
