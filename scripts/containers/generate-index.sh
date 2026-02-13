#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../.." && pwd)
source "${ROOT_DIR}/scripts/_lib/common.sh"
require_stable_env

OUT="${1:-$ROOT_DIR/containers/index.md}"

python3 - "$ROOT_DIR" "$OUT" <<'PY'
from pathlib import Path
import sys

root = Path(sys.argv[1])
out = Path(sys.argv[2])
tool_ids = root / "containers/TOOL_IDS.txt"
if not tool_ids.exists():
    raise SystemExit("missing containers/TOOL_IDS.txt")

rows = []
for raw in tool_ids.read_text(encoding="utf-8").splitlines():
    line = raw.strip()
    if not line or line.startswith("#"):
        continue
    parts = line.split("\t")
    if len(parts) != 2:
        raise SystemExit(f"invalid TOOL_IDS row: {line}")
    tool_id, status = parts
    ap_bijux = (root / "containers/apptainer/bijux" / f"{tool_id}.def").exists()
    ap_non = (root / "containers/apptainer/non-bijux" / f"{tool_id}.def").exists()
    dk_arm64 = (root / "containers/docker/arm64" / f"Dockerfile.{tool_id}").exists()
    dk_amd64 = (root / "containers/docker/amd64" / f"Dockerfile.{tool_id}").exists()

    if ap_bijux and ap_non:
        ap_src = "invalid:both"
    elif ap_bijux:
        ap_src = "bijux"
    elif ap_non:
        ap_src = "non-bijux"
    else:
        ap_src = "none"

    if dk_arm64 and dk_amd64:
        docker_src = "arm64+amd64"
    elif dk_arm64:
        docker_src = "arm64"
    elif dk_amd64:
        docker_src = "amd64"
    else:
        docker_src = "none"

    rows.append((tool_id, status, ap_src, docker_src))

lines = []
lines.append("# Containers Index")
lines.append("")
lines.append("<!-- GENERATED FILE - DO NOT EDIT -->")
lines.append("<!-- source: scripts/containers/generate-index.sh -->")
lines.append("")
lines.append("Purpose: Authoritative tool/container index for container governance and CI checks.")
lines.append("")
lines.append("## Authority")
lines.append("- Tool IDs + lifecycle status: `containers/TOOL_IDS.txt` (generated from registry).")
lines.append("- Container version metadata: `containers/versions/versions.toml` + `containers/versions/lock.json`.")
lines.append("- Non-bijux provenance: `containers/apptainer/non-bijux/NON_BIJUX_SOURCES.md`.")
lines.append("- Ownership map: `containers/OWNERS.toml`.")
lines.append("")
lines.append("## Tool Container Coverage")
lines.append("| tool_id | status | apptainer_source | docker_source |")
lines.append("|---|---|---|---|")
for tool_id, status, ap_src, docker_src in rows:
    lines.append(f"| `{tool_id}` | `{status}` | `{ap_src}` | `{docker_src}` |")
lines.append("")
out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
PY
