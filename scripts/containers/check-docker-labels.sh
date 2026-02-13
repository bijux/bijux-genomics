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

root = Path(sys.argv[1])
required = [
    "org.opencontainers.image.title",
    "org.opencontainers.image.version",
    "org.opencontainers.image.source",
    "org.opencontainers.image.licenses",
]

errors = []
docker_versions = {}
for path in sorted((root / "containers/docker/arm64").glob("Dockerfile.*")):
    text = path.read_text(encoding="utf-8")
    missing = [k for k in required if k not in text]
    if missing:
        errors.append(f"{path.relative_to(root)} missing labels: {', '.join(missing)}")
    tool_id = path.name.split("Dockerfile.", 1)[1]
    m_tool = re.search(r'org\.opencontainers\.image\.tool="?([A-Za-z0-9_.-]+)"?', text)
    if m_tool and m_tool.group(1) != tool_id:
        errors.append(f"{path.relative_to(root)} tool label mismatch: {m_tool.group(1)} != {tool_id}")
    m_ver = re.search(r'org\.opencontainers\.image\.version="?([A-Za-z0-9_.:-]+)"?', text)
    if m_ver:
        docker_versions[tool_id] = m_ver.group(1)

# Parity: when both Dockerfile and Apptainer def exist for a tool, versions must match.
for path in sorted((root / "containers/apptainer").rglob("*.def")):
    tool_id = path.stem
    if tool_id not in docker_versions:
        continue
    text = path.read_text(encoding="utf-8")
    m = re.search(r'org\.opencontainers\.image\.version\s+([^\s]+)', text)
    if not m:
        continue
    appt_ver = m.group(1).strip().strip('"')
    if docker_versions[tool_id] != appt_ver:
        errors.append(
            f"version parity mismatch for {tool_id}: docker={docker_versions[tool_id]} apptainer={appt_ver}"
        )

if errors:
    print("docker label policy check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("docker label policy: OK")
PY
