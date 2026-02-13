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
external = set(tomllib.loads((root / "configs/domain/external_tools.toml").read_text(encoding="utf-8")).get("non_container_tools", {}).keys())
errors = []

for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    fx_root = dom_dir / "fixtures"
    readme = fx_root / "README.md"
    if not readme.exists():
        errors.append(f"{readme.relative_to(root)} missing")
        readme_text = ""
    else:
        readme_text = readme.read_text(encoding="utf-8")

    known_stage_ids = set()
    for sf in sorted((dom_dir / "stages").glob("*.yaml")):
        if sf.name == "_schema.yaml":
            continue
        m = re.search(r'^stage_id:\s*"?([^"\n#]+)"?\s*$', sf.read_text(encoding="utf-8"), flags=re.MULTILINE)
        if m:
            known_stage_ids.add(m.group(1).strip())

    known_tools = set()
    for tf in sorted((dom_dir / "tools").glob("*.yaml")):
        if tf.name == "_schema.yaml":
            continue
        m = re.search(r'^tool_id:\s*"?([^"\n#]+)"?\s*$', tf.read_text(encoding="utf-8"), flags=re.MULTILINE)
        known_tools.add(m.group(1).strip() if m else tf.stem)

    fixture_dirs = sorted([p.name for p in fx_root.iterdir() if p.is_dir()]) if fx_root.exists() else []
    for stage_dir in fixture_dirs:
        if stage_dir not in known_stage_ids:
            errors.append(f"{(fx_root / stage_dir).relative_to(root)}: fixture stage directory is not a known stage_id")
        if readme_text:
            if stage_dir not in readme_text:
                errors.append(f"{readme.relative_to(root)}: missing fixture directory listing for '{stage_dir}'")
            if not re.search(rf"(?im)^\s*[-*]\s*`?{re.escape(stage_dir)}`?\s*:.*intent", readme_text):
                errors.append(f"{readme.relative_to(root)}: '{stage_dir}' entry must include intent")

    for fx in sorted(fx_root.glob("*/*.txt")):
        text = fx.read_text(encoding="utf-8").strip()
        if "=" not in text:
            # legacy compact fixture format: "<stage> <tool> ..."
            parts = text.split()
            if len(parts) < 2:
                errors.append(f"{fx.relative_to(root)}: invalid fixture format")
                continue
            tool = parts[1].strip()
            stage_dir = fx.parent.name
            if stage_dir not in known_stage_ids:
                errors.append(f"{fx.relative_to(root)}: fixture path stage '{stage_dir}' is unknown")
            if tool not in known_tools and tool not in external:
                errors.append(f"{fx.relative_to(root)}: legacy fixture tool '{tool}' not found in domain tools or external tools")
            if tool in external:
                continue
            # force migration for shipped tools
            errors.append(f"{fx.relative_to(root)}: legacy fixture format; use key=value contract fields")
            continue
        kv = {}
        for line in text.splitlines():
            if not line.strip():
                continue
            if "=" not in line:
                continue
            k, v = line.split("=", 1)
            kv[k.strip()] = v.strip()
        for key in ("tool", "tool_version", "command", "args", "expected_outputs", "expected_stdout_patterns", "stage"):
            if key not in kv:
                errors.append(f"{fx.relative_to(root)}: missing required key '{key}'")
        if "tool" in kv and not re.fullmatch(r"[a-z0-9_]+", kv["tool"]):
            errors.append(
                f"{fx.relative_to(root)}: tool id '{kv['tool']}' must be snake_case ([a-z0-9_]+)"
            )
        if "tool" in kv and kv["tool"] != fx.stem:
            errors.append(
                f"{fx.relative_to(root)}: tool field '{kv['tool']}' must match fixture filename stem '{fx.stem}'"
            )
        if "tool" in kv and kv["tool"] not in known_tools and kv["tool"] not in external:
            errors.append(
                f"{fx.relative_to(root)}: tool '{kv['tool']}' not found in domain tools or external tools policy"
            )
        # stage path consistency
        stage_dir = fx.parent.name
        if kv.get("stage") and kv["stage"] != stage_dir:
            errors.append(f"{fx.relative_to(root)}: stage mismatch ({kv['stage']} != {stage_dir})")
        if stage_dir not in known_stage_ids:
            errors.append(f"{fx.relative_to(root)}: fixture path stage '{stage_dir}' is unknown")
        shipping = kv.get("shipping", "").strip()
        tool = kv.get("tool", "").strip()
        if shipping == "external" and tool and tool not in external:
            errors.append(
                f"{fx.relative_to(root)}: shipping=external requires tool in configs/domain/external_tools.toml"
            )
        if tool in external and shipping != "external":
            errors.append(
                f"{fx.relative_to(root)}: external tool '{tool}' must declare shipping=external"
            )

if errors:
    print("fixture contracts check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("fixture contracts: OK")
PY
