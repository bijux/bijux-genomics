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
errors = []

runtime_allowed = {}
for p in (root / "containers/network").glob("*.network.toml"):
    d = tomllib.loads(p.read_text(encoding="utf-8"))
    tid = str(d.get("tool_id", p.stem)).strip()
    runtime_allowed[tid] = bool(d.get("runtime_network", False))

download_pat = re.compile(r"\b(curl|wget)\b")

for p in sorted((root / "containers/apptainer").rglob("*.def")):
    text = p.read_text(encoding="utf-8", errors="ignore")
    tool = p.stem
    chunks = []
    if "%runscript" in text:
        chunks.append(text.split("%runscript", 1)[1].split("\n%", 1)[0])
    if "%environment" in text:
        chunks.append(text.split("%environment", 1)[1].split("\n%", 1)[0])
    for chunk in chunks:
        if download_pat.search(chunk) and not runtime_allowed.get(tool, False):
            errors.append(f"{p.relative_to(root)}: runtime curl/wget forbidden unless runtime_network=true")

for p in sorted((root / "containers/docker/arm64").glob("Dockerfile.*")):
    tool = p.name.split("Dockerfile.", 1)[1]
    for i, line in enumerate(p.read_text(encoding="utf-8", errors="ignore").splitlines(), start=1):
        s = line.strip()
        if s.startswith("ENTRYPOINT") or s.startswith("CMD"):
            if download_pat.search(s) and not runtime_allowed.get(tool, False):
                errors.append(f"{p.relative_to(root)}:{i}: runtime curl/wget in CMD/ENTRYPOINT forbidden unless runtime_network=true")

if errors:
    print("runtime download policy: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)
print("runtime download policy: OK")
PY

