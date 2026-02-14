#!/usr/bin/env bash
set -euo pipefail
IFS=$'\n\t'
LC_ALL=C
export LC_ALL
SCRIPT_DIR=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
ROOT_DIR=$(cd "${SCRIPT_DIR}/../../" && pwd)
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
stage_files = [
    root / "configs/ci/stages/stages.toml",
    root / "configs/ci/stages/stages_vcf.toml",
    root / "configs/ci/stages/stages_vcf_downstream.toml",
]
cfg_stages: set[str] = set()
for p in stage_files:
    d = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in d.get("stages", []):
        sid = str(row.get("id", "")).strip()
        if sid:
            cfg_stages.add(sid)

dom_stages: set[str] = set()
for f in sorted(root.glob("domain/*/stages/*.yaml")):
    if f.name.startswith("_"):
        continue
    text = f.read_text(encoding="utf-8")
    m = re.search(r'^stage_id:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    if m:
        dom_stages.add(m.group(1).strip())

domain_tools: set[str] = set()
for f in sorted(root.glob("domain/*/tools/*.yaml")):
    if f.name.startswith("_"):
        continue
    text = f.read_text(encoding="utf-8")
    m = re.search(r'^tool_id:\s*"?([^"\n]+)"?', text, re.MULTILINE)
    if m:
        domain_tools.add(m.group(1).strip())

container_tools: set[str] = set()
for p in (root / "containers/docker/arm64").glob("Dockerfile.*"):
    container_tools.add(p.name.split("Dockerfile.", 1)[1])
for p in (root / "containers/apptainer/bijux").glob("*.def"):
    container_tools.add(p.stem)
for p in (root / "containers/apptainer/non-bijux").glob("*.def"):
    container_tools.add(p.stem)

errors: list[str] = []
for sid in sorted(cfg_stages - dom_stages):
    errors.append(f"configs/ci/stages: stage '{sid}' not found under domain/**/stages/*.yaml")

for p in stage_files:
    d = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in d.get("stages", []):
        sid = str(row.get("id", "")).strip()
        if not sid:
            continue
        tools = [str(t).strip() for t in row.get("tools", []) if str(t).strip()]
        if not tools:
            errors.append(f"{p.relative_to(root)}: stage '{sid}' has no tools declared")
            continue
        has_impl = False
        for tid in tools:
            if tid in domain_tools and tid in container_tools:
                has_impl = True
                break
        if not has_impl:
            errors.append(
                f"{p.relative_to(root)}: stage '{sid}' has no tool with both domain tool yaml and container definition"
            )

if errors:
    print("stage-domain-parity: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("stage-domain-parity: OK")
PY
