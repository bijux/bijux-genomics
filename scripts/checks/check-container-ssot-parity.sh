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
registry_files = sorted((root / "configs/ci/registry").glob("tool_registry*.toml"))
external_cfg = root / "configs/domain/external_tools.toml"
errors = []

registry_ids = set()
registry_prod_container = set()
for reg in registry_files:
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if not tid or "." in tid:
            continue
        registry_ids.add(tid)
        status = str(row.get("status") or "").strip()
        container = bool(row.get("container", False))
        if status in {"production", "supported"} and container:
            registry_prod_container.add(tid)

external_ids = set()
if external_cfg.exists():
    data = tomllib.loads(external_cfg.read_text(encoding="utf-8"))
    external_ids = set(data.get("non_container_tools", {}).keys())

domain_ids = set()
for tf in sorted((root / "domain").glob("*/tools/*.yaml")):
    if tf.name == "_schema.yaml":
        continue
    text = tf.read_text(encoding="utf-8")
    m = re.search(r'^tool_id:\s*"?([^"\n#]+)"?\s*$', text, flags=re.MULTILINE)
    tid = m.group(1).strip() if m else tf.stem
    domain_ids.add(tid)
    if tid not in registry_ids and tid not in external_ids:
        errors.append(f"{tf.relative_to(root)}: tool_id '{tid}' missing from registry SSoT or external tools policy")

container_ids = set()
for p in (root / "containers/apptainer/bijux").glob("*.def"):
    container_ids.add(p.stem)
for p in (root / "containers/apptainer/non-bijux").glob("*.def"):
    container_ids.add(p.stem)
for p in (root / "containers/docker/arm64").glob("Dockerfile.*"):
    container_ids.add(p.name.split("Dockerfile.", 1)[1])
for tid in sorted(registry_prod_container):
    if tid not in container_ids:
        errors.append(f"registry tool '{tid}' is production/supported container=true but has no container definition")
    if tid not in domain_ids and tid not in external_ids:
        errors.append(f"registry tool '{tid}' is production/supported but missing domain tool contract")

if errors:
    print("container-ssot-parity: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("container-ssot-parity: OK")
PY
