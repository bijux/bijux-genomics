#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT_DIR="${1:-$ROOT_DIR/containers/docs/tools}"
mkdir -p "$OUT_DIR"

python3 - "$ROOT_DIR" "$OUT_DIR" <<'PY'
from pathlib import Path
import json
import sys
try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

root = Path(sys.argv[1])
out = Path(sys.argv[2])
versions = tomllib.loads((root / "containers/versions/versions.toml").read_text(encoding="utf-8"))
images = tomllib.loads((root / "configs/ci/tools/images.toml").read_text(encoding="utf-8"))

licenses = {}
for p in sorted((root / "containers/licenses").glob("*.license.toml")):
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    licenses[p.stem] = data

network = {}
for p in sorted((root / "containers/network").glob("*.network.toml")):
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    network[p.stem] = data

status = {}
for p in sorted((root / "artifacts/containers").glob("*.json")):
    if p.name in {"summary.json", "report.json"}:
        continue
    try:
        d = json.loads(p.read_text(encoding="utf-8"))
    except Exception:
        continue
    tool = str(d.get("tool", "")).strip()
    if tool:
        status[tool] = str(d.get("status", "unknown"))

docker_ids = {p.name.split("Dockerfile.", 1)[1] for p in (root / "containers/docker/arm64").glob("Dockerfile.*")}
apptainer_ids = {p.stem for p in (root / "containers/apptainer/bijux").glob("*.def")}
apptainer_ids |= {p.stem for p in (root / "containers/apptainer/non-bijux").glob("*.def")}

for tool in sorted(versions.keys()):
    v = versions.get(tool, {})
    l = licenses.get(tool, {})
    n = network.get(tool, {})
    runtimes = []
    if tool in docker_ids:
        runtimes.append("docker-arm64")
    if tool in apptainer_ids:
        runtimes.append("apptainer")
    limitations = []
    if bool(n.get("runtime_network", False)):
        limitations.append("Runtime network access required.")
    if not runtimes:
        limitations.append("No runtime implementation currently present.")
    if not limitations:
        limitations.append("No declared limitations.")

    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: scripts/containers/generate-tool-docs.sh -->",
        f"# {tool}",
        "",
        "Purpose: generated per-tool container contract summary.",
        "",
        f"- Version: `{v.get('version', '')}`",
        f"- License: `{l.get('spdx', l.get('upstream_license', 'unknown'))}`",
        f"- Runtime support: `{', '.join(runtimes) if runtimes else 'none'}`",
        f"- Smoke status: `{status.get(tool, 'unknown')}`",
        "",
        "## Known Limitations",
    ]
    lines.extend([f"- {x}" for x in limitations])
    (out / f"{tool}.md").write_text("\n".join(lines) + "\n", encoding="utf-8")

index_lines = [
    "<!-- GENERATED FILE - DO NOT EDIT -->",
    "<!-- source: scripts/containers/generate-tool-docs.sh -->",
    "# Tool Docs Index",
    "",
]
for tool in sorted(versions.keys()):
    index_lines.append(f"- `{tool}`: `containers/docs/tools/{tool}.md`")
(out / "index.md").write_text("\n".join(index_lines) + "\n", encoding="utf-8")
print(f"generated {len(versions)} tool docs under {out}")
PY

