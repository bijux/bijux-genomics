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
errors = []

def parse_list(text: str, key: str):
    out = []
    in_block = False
    for line in text.splitlines():
        if re.match(rf"^{re.escape(key)}:\s*$", line):
            in_block = True
            continue
        if in_block:
            m = re.match(r"^\s*-\s*([^\s#]+)\s*$", line)
            if m:
                out.append(m.group(1).strip().strip('"'))
                continue
            if line and not line.startswith(" "):
                break
    return out

for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    idx = dom_dir / "index.yaml"
    if not idx.exists():
        continue
    text = idx.read_text(encoding="utf-8")
    indexed_stages = set(parse_list(text, "stage_ids"))
    indexed_tools = set(parse_list(text, "tool_ids"))

    fixture_tools = set()
    for fx in (dom_dir / "fixtures").glob("*/*.txt"):
        fixture_tools.add(fx.stem)

    for sf in (dom_dir / "stages").glob("*.yaml"):
        if sf.name == "_schema.yaml":
            continue
        m = re.search(r'^stage_id:\s*"?([^"\n#]+)"?\s*$', sf.read_text(encoding="utf-8"), flags=re.MULTILINE)
        sid = m.group(1).strip() if m else sf.stem
        if sid not in indexed_stages:
            errors.append(f"{sf.relative_to(root)}: orphan stage file not referenced by index.yaml")

    for tf in (dom_dir / "tools").glob("*.yaml"):
        if tf.name == "_schema.yaml":
            continue
        m = re.search(r'^tool_id:\s*"?([^"\n#]+)"?\s*$', tf.read_text(encoding="utf-8"), flags=re.MULTILINE)
        tid = m.group(1).strip() if m else tf.stem
        if tid not in indexed_tools and tid not in fixture_tools:
            errors.append(f"{tf.relative_to(root)}: orphan tool file not referenced by index.yaml or fixtures")

if errors:
    print("orphan stage/tool check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("orphan stage/tool: OK")
PY
