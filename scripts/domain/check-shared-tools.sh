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
cfg = root / "configs/domain/shared_tools.toml"
shared = tomllib.loads(cfg.read_text(encoding="utf-8")).get("shared_tools", {})

tools = {}
for tool_file in sorted((root / "domain").glob("*/tools/*.yaml")):
    if tool_file.name == "_schema.yaml":
        continue
    domain = tool_file.parts[-3]
    text = tool_file.read_text(encoding="utf-8")
    def scalar(key):
        m = re.search(rf"^{re.escape(key)}:\s*['\"]?([^'\"\n#]+)['\"]?\s*$", text, flags=re.MULTILINE)
        return m.group(1).strip() if m else ""
    tool_id = scalar("tool_id")
    if not tool_id:
        continue
    tools.setdefault(tool_id, []).append(
        {
            "domain": domain,
            "default_version": scalar("default_version"),
            "license": scalar("license"),
            "upstream": scalar("upstream"),
            "path": str(tool_file.relative_to(root)),
        }
    )

errors = []
for tool_id, rows in sorted(tools.items()):
    if len(rows) <= 1:
        continue
    if tool_id not in shared:
        errors.append(f"{tool_id}: appears in multiple domains but not declared in configs/domain/shared_tools.toml")
        continue
    entry = shared[tool_id]
    domains_declared = sorted(str(d) for d in entry.get("domains", []))
    domains_actual = sorted(r["domain"] for r in rows)
    if domains_actual != domains_declared:
        errors.append(f"{tool_id}: shared domains mismatch declared={domains_declared} actual={domains_actual}")
    for key in ("default_version", "license", "upstream"):
        expected = str(entry.get(key, "")).strip()
        if not expected:
            errors.append(f"{tool_id}: missing {key} in shared_tools config")
            continue
        for row in rows:
            if row[key] and row[key] != expected:
                errors.append(f"{row['path']}: {key}={row[key]} differs from shared_tools.{tool_id}.{key}={expected}")

if errors:
    print("shared-tools check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("shared-tools: OK")
PY
