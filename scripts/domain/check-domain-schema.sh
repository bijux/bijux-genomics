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


def parse_scalar_from_text(text: str, key: str) -> str | None:
    pat = re.compile(rf"^{re.escape(key)}:\s*\"?([^\"\n#]+)\"?\s*$", re.MULTILINE)
    m = pat.search(text)
    if not m:
        return None
    return m.group(1).strip()


def parse_inline_list(text: str, key: str):
    m = re.search(rf"^{re.escape(key)}:\s*\[(.*?)\]\s*$", text, re.MULTILINE)
    if not m:
        return []
    raw = m.group(1).strip()
    if not raw:
        return []
    return [part.strip().strip('"').strip("'") for part in raw.split(",") if part.strip()]


def parse_stage_slug(stage_id: str, domain: str) -> str | None:
    prefix = f"{domain}."
    if not stage_id.startswith(prefix):
        return None
    return stage_id[len(prefix):]


def load_registry_production_bindings(root_dir: Path):
    reg = root_dir / "configs/ci/registry/tool_registry.toml"
    data = tomllib.loads(reg.read_text(encoding="utf-8"))
    bindings = set()
    for row in data.get("tools", []):
        tool_id = str(row.get("id") or row.get("tool_id") or "").strip()
        status = str(row.get("status") or "").strip()
        if not tool_id or status not in {"production", "supported"}:
            continue
        for sid in row.get("bindings", []):
            bindings.add((str(sid).strip(), tool_id))
    return bindings


production_bindings = load_registry_production_bindings(root)

downstream_stages_cfg = root / "configs" / "ci" / "stages" / "stages_vcf_downstream.toml"
if not downstream_stages_cfg.exists():
    errors.append(f"{downstream_stages_cfg}: missing required downstream stages registry file")
else:
    ds = tomllib.loads(downstream_stages_cfg.read_text(encoding="utf-8"))
    rows = ds.get("stages", [])
    if not rows:
        errors.append(f"{downstream_stages_cfg}: must define at least one [[stages]] entry")
    for row in rows:
        sid = str(row.get("id", "")).strip()
        status = str(row.get("status", "")).strip()
        if not sid.startswith("vcf."):
            errors.append(f"{downstream_stages_cfg}: downstream stage id must start with 'vcf.': {sid}")
        if status not in {"planned", "experimental", "production", "supported"}:
            errors.append(f"{downstream_stages_cfg}: invalid stage status '{status}' for {sid}")


for dom_dir in sorted(p for p in domain_root.iterdir() if p.is_dir()):
    dom = dom_dir.name
    stage_schema = dom_dir / "stages" / "_schema.yaml"
    tool_schema = dom_dir / "tools" / "_schema.yaml"
    if not stage_schema.exists() or not tool_schema.exists():
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
        defaults_source = parse_scalar(stage_file, "defaults_source")
        if not defaults_source:
            errors.append(f"{stage_file}: missing defaults_source")
        elif not (
            defaults_source.startswith("citation:")
            or defaults_source.startswith("doc_ref:")
        ):
            errors.append(
                f"{stage_file}: defaults_source must start with citation: or doc_ref: (got {defaults_source})"
            )

        slug = parse_stage_slug(stage_id or "", dom)
        if stage_id and slug is None:
            errors.append(f"{stage_file}: stage_id must use '<domain>.<stage_slug>' format")
        if slug is not None:
            if not re.fullmatch(r"[a-z0-9_]+", slug):
                errors.append(
                    f"{stage_file}: stage slug '{slug}' must match [a-z0-9_]+"
                )
            if "__" in slug:
                errors.append(f"{stage_file}: stage slug '{slug}' must not contain '__'")
            parts = [p for p in slug.split("_") if p]
            for i in range(len(parts) - 1):
                if parts[i] == parts[i + 1]:
                    errors.append(
                        f"{stage_file}: stage slug '{slug}' has repeated adjacent token '{parts[i]}'"
                    )

        if dom == "vcf":
            compatible = parse_inline_list(text, "compatible_tools")
            single_just = parse_scalar_from_text(text, "single_tool_justification")
            if len(compatible) < 2 and not single_just:
                errors.append(
                    f"{stage_file}: single-tool stage requires single_tool_justification when compatible_tools has <2 tools"
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
            if not re.fullmatch(r"[a-z0-9_]+", tool_id):
                errors.append(
                    f"{tool_file}: tool_id '{tool_id}' must be snake_case ([a-z0-9_]+)"
                )
        scope = parse_scalar(tool_file, "scope")
        if required_tool_scope and scope != required_tool_scope:
            errors.append(f"{tool_file}: scope must be {required_tool_scope} (got {scope})")

    metrics_file = dom_dir / "metrics.yaml"
    if not metrics_file.exists():
        errors.append(f"{dom_dir}: missing metrics.yaml")
    else:
        mtext = read_text(metrics_file)
        schema_version = parse_scalar_from_text(mtext, "schema_version")
        declared_domain = parse_scalar_from_text(mtext, "domain")
        if not schema_version or not schema_version.startswith("bijux."):
            errors.append(
                f"{metrics_file}: schema_version must exist and start with 'bijux.'"
            )
        if declared_domain != dom:
            errors.append(
                f"{metrics_file}: domain must be '{dom}' (got {declared_domain})"
            )
        has_metric_ids = bool(re.search(r"^metric_ids:\s*$", mtext, re.MULTILINE))
        has_metrics = bool(re.search(r"^metrics:\s*$", mtext, re.MULTILINE))
        if not (has_metric_ids or has_metrics):
            errors.append(f"{metrics_file}: must define either metric_ids: or metrics:")
        if has_metric_ids and not re.search(r"^\s*-\s*[a-z0-9_]+\s*$", mtext, re.MULTILINE):
            errors.append(
                f"{metrics_file}: metric_ids must contain at least one snake_case metric id"
            )
        if has_metrics and not re.search(r"^\s*-\s*id:\s*\"?[a-z0-9_]+\"?\s*$", mtext, re.MULTILINE):
            errors.append(f"{metrics_file}: metrics entries must include id fields")

    # Every production tool binding for this domain must have fixture coverage in that stage.
    fixture_pairs = set()
    for fx in (dom_dir / "fixtures").glob("*/*.txt"):
        stage_dir = fx.parent.name
        tool = fx.stem
        fixture_pairs.add((stage_dir, tool))
    for stage_id, tool_id in sorted(production_bindings):
        if not stage_id.startswith(f"{dom}."):
            continue
        if (stage_id, tool_id) not in fixture_pairs:
            errors.append(
                f"domain/{dom}/fixtures: missing production fixture for binding ({stage_id}, {tool_id})"
            )

if errors:
    print("domain schema check failed:", file=sys.stderr)
    for err in errors:
        print(f"- {err}", file=sys.stderr)
    raise SystemExit(1)

print("domain schema: OK")
PY
