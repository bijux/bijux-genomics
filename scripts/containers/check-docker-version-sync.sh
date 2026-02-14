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
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
errors = []

for dockerfile in sorted((root / "containers/docker/arm64").glob("Dockerfile.*")):
    tool = dockerfile.name.split("Dockerfile.", 1)[1]
    text = dockerfile.read_text(encoding="utf-8")
    m = re.search(r"^ARG\s+TOOL_VERSION\s*=\s*([^\s#]+)\s*$", text, flags=re.M)
    if not m:
        errors.append(f"{dockerfile.relative_to(root)}: missing ARG TOOL_VERSION=<version>")
        continue
    docker_v = m.group(1).strip().strip('"\'')
    reg = versions.get(tool)
    if not isinstance(reg, dict):
        errors.append(f"{dockerfile.relative_to(root)}: tool '{tool}' missing in versions.toml")
        continue
    reg_v = str(reg.get("version", "")).strip()
    placeholder = (
        docker_v in {"unknown", "planned", "latest-pinned"} or
        docker_v.endswith("-planned")
    )
    if not placeholder and docker_v != reg_v:
        errors.append(f"{dockerfile.relative_to(root)}: TOOL_VERSION '{docker_v}' != versions.toml '{reg_v}'")

    if 'org.opencontainers.image.version="${TOOL_VERSION}"' not in text:
        errors.append(f"{dockerfile.relative_to(root)}: image version label must reference TOOL_VERSION build arg")

if errors:
    print("docker version sync: failed", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("docker version sync: OK")
PY
