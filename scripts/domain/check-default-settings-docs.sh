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
required_sections = ["inputs", "outputs", "key parameters", "validity limits"]

for dom_dir in sorted((root / "domain").iterdir()):
    if not dom_dir.is_dir():
        continue
    dom = dom_dir.name
    doc = dom_dir / "docs" / "DEFAULT_SETTINGS.md"
    if not doc.exists():
        errors.append(f"domain/{dom}/docs/DEFAULT_SETTINGS.md missing")
        continue
    text = doc.read_text(encoding="utf-8").lower()
    for s in required_sections:
        if s not in text:
            errors.append(f"{doc.relative_to(root)}: missing required section phrase '{s}'")
    stages = []
    for sf in sorted((dom_dir / "stages").glob("*.yaml")):
        if sf.name == "_schema.yaml":
            continue
        m = re.search(r'^stage_id:\s*"?([^"\n#]+)"?\s*$', sf.read_text(encoding="utf-8"), flags=re.MULTILINE)
        if m:
            stages.append(m.group(1).strip())
    for stage in stages:
        if stage.lower() not in text:
            errors.append(f"{doc.relative_to(root)}: missing stage coverage for '{stage}'")
        has_doc_default = bool(re.search(rf"{re.escape(stage.lower())}.*default", text, re.DOTALL))
        has_doc_rationale = bool(re.search(rf"{re.escape(stage.lower())}.*rationale", text, re.DOTALL))
        if not has_doc_default:
            errors.append(f"{doc.relative_to(root)}: missing blessed default description for '{stage}'")
        idx = dom_dir / "index.yaml"
        idx_text = idx.read_text(encoding="utf-8") if idx.exists() else ""
        has_idx_default = bool(re.search(rf"^\s{{2}}{re.escape(stage)}:\s*.+$", idx_text, flags=re.MULTILINE))
        has_idx_rationale = False
        in_rationale = False
        for line in idx_text.splitlines():
            if line.startswith("active_default_rationale:"):
                in_rationale = True
                continue
            if in_rationale and re.match(r"^[A-Za-z0-9_]+:\s*", line):
                break
            if in_rationale and re.match(rf"^\s{{2}}{re.escape(stage)}:\s*.+$", line):
                has_idx_rationale = True
                break
        if not (has_doc_rationale or has_idx_rationale):
            errors.append(f"{doc.relative_to(root)}: missing blessed default rationale for '{stage}'")
        if not (has_doc_default or has_idx_default):
            errors.append(f"{doc.relative_to(root)}: missing blessed default mapping for '{stage}'")
        # single-tool stage must be explicitly justified
    idx = dom_dir / "index.yaml"
    if idx.exists():
        idx_text = idx.read_text(encoding="utf-8")
        in_block = False
        mapping = {}
        current = None
        for line in idx_text.splitlines():
            if line.startswith("stage_tool_compatibility:"):
                in_block = True
                continue
            if in_block and re.match(r"^[A-Za-z0-9_]+:", line):
                break
            if not in_block:
                continue
            m = re.match(r"^\s{2}([a-z0-9._-]+):\s*(.*)$", line)
            if m:
                current = m.group(1)
                rest = m.group(2).strip()
                if rest.startswith("[") and rest.endswith("]"):
                    vals = [x.strip().strip('"') for x in rest[1:-1].split(",") if x.strip()]
                    mapping[current] = vals
                else:
                    mapping[current] = []
                continue
            m2 = re.match(r"^\s{4}-\s*([a-z0-9._-]+)\s*$", line)
            if m2 and current:
                mapping.setdefault(current, []).append(m2.group(1))
        for stage, tools in mapping.items():
            if len(tools) == 1:
                marker = f"single_tool_justification: {stage}".lower()
                if marker not in text:
                    errors.append(f"{doc.relative_to(root)}: missing '{marker}' for single-tool stage")

if errors:
    print("default-settings docs check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)
print("default-settings docs: OK")
PY
