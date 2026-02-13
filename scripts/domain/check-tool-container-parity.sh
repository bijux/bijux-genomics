#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
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
external_cfg = root / "configs/domain/external_tools.toml"
external = set(tomllib.loads(external_cfg.read_text(encoding="utf-8")).get("non_container_tools", {}).keys())

docker_tools = set()
for p in (root / "containers/docker/arm64").glob("Dockerfile.*"):
    docker_tools.add(p.name.split("Dockerfile.", 1)[1])
appt_tools = {p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")}
appt_tools |= {p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def")}
all_container_tools = docker_tools | appt_tools

errors = []
for tool_file in sorted((root / "domain").glob("*/tools/*.yaml")):
    if tool_file.name == "_schema.yaml":
        continue
    text = tool_file.read_text(encoding="utf-8")
    def scalar(key):
        m = re.search(rf'^{re.escape(key)}:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
        return m.group(1).strip() if m else ""
    tool_id = scalar("tool_id")
    status = scalar("status")
    if not tool_id or status == "out_of_scope":
        continue
    if tool_id in external:
        continue
    candidates = {tool_id, tool_id.replace("-", "_")}
    if not any(c in all_container_tools for c in candidates):
        errors.append(
            f"{tool_file.relative_to(root)}: tool_id '{tool_id}' has no matching container def (add container or mark in configs/domain/external_tools.toml)"
        )

if errors:
    print("tool/container parity check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("tool/container parity: OK")
PY
