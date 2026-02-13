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
domain_root = root / "domain"
errors = []


def read_text(path: Path) -> str:
    return path.read_text(encoding="utf-8")


def top_level_keys(text: str):
    keys = []
    for line in text.splitlines():
        if line.startswith("#") or not line.strip():
            continue
        m = re.match(r"^([A-Za-z0-9_]+):", line)
        if m:
            keys.append(m.group(1))
    return set(keys)


def parse_scalar(path: Path, key: str) -> str | None:
    pat = re.compile(rf"^{re.escape(key)}:\s*\"?([^\"\n#]+)\"?\s*$", re.MULTILINE)
    m = pat.search(read_text(path))
    if not m:
        return None
    return m.group(1).strip()


def parse_required_fields(schema_path: Path):
    fields = []
    in_section = False
    for raw in read_text(schema_path).splitlines():
        line = raw.rstrip()
        if re.match(r"^required_fields:\s*$", line):
            in_section = True
            continue
        if in_section:
            m = re.match(r"^\s*-\s*([A-Za-z0-9_]+)\s*$", line)
            if m:
                fields.append(m.group(1))
                continue
            if line and not line.startswith(" "):
                break
    return fields


for dom_dir in sorted(p for p in domain_root.iterdir() if p.is_dir()):
    dom = dom_dir.name
    stage_schema = dom_dir / "stages" / "_schema.yaml"
    tool_schema = dom_dir / "tools" / "_schema.yaml"
    if not stage_schema.exists() or not tool_schema.exists():
        # vcf currently has no _schema files; skip strict schema check for it.
        continue

    required_stage = parse_required_fields(stage_schema)
    required_tool = parse_required_fields(tool_schema)
    required_scope = parse_scalar(stage_schema, "required_scope")
    required_domain = parse_scalar(stage_schema, "domain")
    required_tool_scope = parse_scalar(tool_schema, "required_scope")

    stage_ids_seen = set()
    tool_ids_seen = set()

    for stage_file in sorted((dom_dir / "stages").glob("*.yaml")):
        if stage_file.name == "_schema.yaml":
            continue
        text = read_text(stage_file)
        keys = top_level_keys(text)
        missing = [k for k in required_stage if k not in keys]
        if missing:
            errors.append(f"{stage_file}: missing required fields: {missing}")
        stage_id = parse_scalar(stage_file, "stage_id")
        if not stage_id:
            errors.append(f"{stage_file}: missing stage_id")
        else:
            if stage_id in stage_ids_seen:
                errors.append(f"{stage_file}: duplicate stage_id in domain {dom}: {stage_id}")
            stage_ids_seen.add(stage_id)
        scope = parse_scalar(stage_file, "scope")
        if required_scope and scope != required_scope:
            errors.append(f"{stage_file}: scope must be {required_scope} (got {scope})")
        declared_domain = parse_scalar(stage_file, "domain")
        if required_domain and declared_domain != required_domain:
            errors.append(
                f"{stage_file}: domain must be {required_domain} (got {declared_domain})"
            )

    for tool_file in sorted((dom_dir / "tools").glob("*.yaml")):
        if tool_file.name == "_schema.yaml":
            continue
        text = read_text(tool_file)
        keys = top_level_keys(text)
        missing = [k for k in required_tool if k not in keys]
        if missing:
            errors.append(f"{tool_file}: missing required fields: {missing}")
        tool_id = parse_scalar(tool_file, "tool_id")
        if not tool_id:
            errors.append(f"{tool_file}: missing tool_id")
        else:
            if tool_id in tool_ids_seen:
                errors.append(f"{tool_file}: duplicate tool_id in domain {dom}: {tool_id}")
            tool_ids_seen.add(tool_id)
        scope = parse_scalar(tool_file, "scope")
        if required_tool_scope and scope != required_tool_scope:
            errors.append(f"{tool_file}: scope must be {required_tool_scope} (got {scope})")

if errors:
    print("domain schema check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("domain schema: OK")
PY
