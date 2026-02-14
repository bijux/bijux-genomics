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
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
docker = {p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*")}
registry_paths = [
    root / "configs/ci/registry/tool_registry.toml",
    root / "configs/ci/registry/tool_registry_vcf.toml",
    root / "configs/ci/registry/tool_registry_experimental.toml",
    root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
required = set()
for rp in registry_paths:
    if not rp.exists():
        continue
    data = tomllib.loads(rp.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        if not isinstance(row, dict):
            continue
        tool = str(row.get("id") or row.get("tool_id") or "").strip()
        runtimes = row.get("runtimes", [])
        if tool and isinstance(runtimes, list) and "docker" in runtimes:
            required.add(tool)

waiver_path = root / "containers/docker/arm64/WAIVERS.toml"
waived = set()
if waiver_path.exists():
    data = tomllib.loads(waiver_path.read_text(encoding="utf-8"))
    for row in data.get("waiver", []):
        tool = str(row.get("tool_id", "")).strip()
        reason = str(row.get("reason", "")).strip()
        owner = str(row.get("owner", "")).strip()
        expires = str(row.get("expires_on", "")).strip()
        if not tool:
            print("docker arm64 completeness: waiver missing tool_id", file=sys.stderr)
            raise SystemExit(1)
        if not reason or not owner or not expires:
            print(f"docker arm64 completeness: waiver for {tool} missing reason/owner/expires_on", file=sys.stderr)
            raise SystemExit(1)
        waived.add(tool)

missing = sorted((required - docker) - waived)
if missing:
    print("docker arm64 completeness: missing dockerfile for docker runtime registry tools:", file=sys.stderr)
    for tool in missing:
        print(f"- {tool}", file=sys.stderr)
    raise SystemExit(1)

print("docker arm64 completeness: OK")
PY
