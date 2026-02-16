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
from __future__ import annotations

from pathlib import Path
import re
import sys

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

ROOT = Path(sys.argv[1])

DOMAIN_STAGE_FILES = sorted(ROOT.glob("domain/*/stages/*.yaml"))
CI_STAGE_FILES = [
    ROOT / "configs/ci/stages/stages.toml",
    ROOT / "configs/ci/stages/stages_vcf.toml",
    ROOT / "configs/ci/stages/stages_vcf_downstream.toml",
]
DEPRECATIONS_FILE = ROOT / "configs/ci/registry/deprecations.toml"
REGISTRY_RS = ROOT / "crates/bijux-dna-core/src/stage_executor_registry.rs"
TOOL_REG_FILES = [
    ROOT / "configs/ci/registry/tool_registry.toml",
    ROOT / "configs/ci/registry/tool_registry_experimental.toml",
    ROOT / "configs/ci/registry/tool_registry_vcf.toml",
    ROOT / "configs/ci/registry/tool_registry_vcf_downstream.toml",
]
REFUSAL_DOC = ROOT / "docs/00-intro/REFUSALS.md"


def parse_scalar(text: str, key: str) -> str:
    m = re.search(rf"^{re.escape(key)}:\s*\"?([^\"\n]+)\"?\s*$", text, flags=re.MULTILINE)
    return m.group(1).strip() if m else ""


def parse_list(text: str, key: str) -> list[str]:
    lines = text.splitlines()
    for i, raw in enumerate(lines):
        line = raw.strip()
        prefix = f"{key}:"
        if not line.startswith(prefix):
            continue
        tail = line[len(prefix) :].strip()
        if tail.startswith("[") and tail.endswith("]"):
            inner = tail[1:-1].strip()
            if not inner:
                return []
            parts = [p.strip() for p in inner.split(",") if p.strip()]
            return [p.strip('"').strip("'") for p in parts]
        if tail:
            return [tail.strip('"').strip("'")]
        out: list[str] = []
        for nxt in lines[i + 1 :]:
            if not nxt.startswith("  - "):
                if nxt.strip() == "":
                    continue
                break
            out.append(nxt.strip()[2:].strip().strip('"').strip("'"))
        return out
    return []


def parse_registry_entries(path: Path) -> dict[str, str]:
    text = path.read_text(encoding="utf-8")
    entries: dict[str, str] = {}
    block_re = re.compile(r"StageExecutorEntry\s*\{(.*?)\},", flags=re.DOTALL)
    for block in block_re.findall(text):
        sid_m = re.search(r'stage_id:\s*"([^"]+)"', block)
        readiness_m = re.search(r"readiness:\s*ReadinessBadge::(\w+)", block)
        if sid_m and readiness_m:
            entries[sid_m.group(1).strip()] = readiness_m.group(1).strip().lower()
    return entries


def collect_container_tools(root: Path) -> set[str]:
    ids: set[str] = set()
    for p in (root / "containers/docker/arm64").glob("Dockerfile.*"):
        ids.add(p.name.split("Dockerfile.", 1)[1])
    for p in (root / "containers/apptainer/lunarc").glob("*.def"):
        ids.add(p.stem)
    for p in (root / "containers/apptainer/lunarc").glob("*.def"):
        ids.add(p.stem)
    return ids


def collect_external_tools(root: Path) -> set[str]:
    p = root / "configs/domain/external_tools.toml"
    if not p.exists():
        return set()
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    ext = data.get("non_container_tools", {})
    if not isinstance(ext, dict):
        return set()
    return {str(k).strip() for k in ext.keys() if str(k).strip()}


errors: list[str] = []

# Domain stage metadata.
domain_stages: dict[str, dict[str, object]] = {}
for f in DOMAIN_STAGE_FILES:
    if f.name.startswith("_"):
        continue
    text = f.read_text(encoding="utf-8")
    sid = parse_scalar(text, "stage_id")
    if not sid:
        continue
    status = parse_scalar(text, "status") or "planned"
    domain_stages[sid] = {
        "status": status,
        "tools": parse_list(text, "compatible_tools"),
        "required_outputs": parse_list(text, "required_outputs"),
        "planned_out_of_scope": parse_list(text, "planned_out_of_scope"),
        "path": f,
    }

# CI stages metadata.
ci_stages: dict[str, dict[str, object]] = {}
for p in CI_STAGE_FILES:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in data.get("stages", []):
        sid = str(row.get("id", "")).strip()
        if not sid:
            continue
        ci_stages[sid] = {
            "status": str(row.get("status", "")).strip(),
            "tools": [str(t).strip() for t in row.get("tools", []) if str(t).strip()],
            "experimental": bool(row.get("experimental", False)),
            "metrics_schema": str(row.get("metrics_schema", "")).strip(),
            "output_kinds": [str(k).strip() for k in row.get("output_kinds", []) if str(k).strip()],
            "source": p,
        }

# Tool registries.
known_tools: set[str] = set()
for p in TOOL_REG_FILES:
    data = tomllib.loads(p.read_text(encoding="utf-8"))
    for row in data.get("tools", []):
        tid = str(row.get("id") or row.get("tool_id") or "").strip()
        if tid:
            known_tools.add(tid)

container_tools = collect_container_tools(ROOT)
external_tools = collect_external_tools(ROOT)

# Deprecations.
dep_rows = tomllib.loads(DEPRECATIONS_FILE.read_text(encoding="utf-8")).get("deprecations", [])
deprecated_stage_ids = {
    str(r.get("stage", "")).strip() for r in dep_rows if str(r.get("stage", "")).strip()
}

# Code-backed registry.
registry_stages = parse_registry_entries(REGISTRY_RS)

# 1/2/3: Stage registry parity with out_of_scope/deprecations allowlist.
allowed_missing = {
    sid
    for sid, meta in domain_stages.items()
    if str(meta["status"]) == "out_of_scope"
    or bool(meta["planned_out_of_scope"])
    or sid in deprecated_stage_ids
}
for sid, meta in ci_stages.items():
    if str(meta["status"]) in {"out_of_scope", "planned"}:
        allowed_missing.add(sid)

for sid in sorted(domain_stages.keys()):
    if sid not in registry_stages and sid not in allowed_missing:
        errors.append(
            f"missing executor for stage '{sid}' (not out_of_scope/deprecated/planned)"
        )

for sid in sorted(registry_stages.keys()):
    if sid not in domain_stages:
        errors.append(
            f"hidden executor: '{sid}' exists in code registry but not in domain/*/stages/*.yaml"
        )

# 4: configs/ci/stages derivable from domain yaml.
for sid, meta in ci_stages.items():
    if sid not in domain_stages:
        errors.append(
            f"{Path(meta['source']).relative_to(ROOT)}: stage '{sid}' absent from domain stage yaml"
        )

for sid, meta in domain_stages.items():
    status = str(meta["status"])
    if status == "supported" and sid not in ci_stages:
        errors.append(f"domain stage '{sid}' is supported but missing from configs/ci/stages/*.toml")

# 5/6: required tools declaration + tool registry + container policy.
for sid, meta in ci_stages.items():
    status = str(meta["status"])
    tools = list(meta["tools"])
    if status == "supported" and not tools:
        errors.append(f"stage '{sid}' is supported but has no required tools declaration in CI stage config")
    for tid in tools:
        if tid not in known_tools:
            errors.append(f"stage '{sid}' requires unknown tool '{tid}' (missing tool registry entry)")
        if tid not in container_tools and tid not in external_tools:
            errors.append(
                f"stage '{sid}' tool '{tid}' has no docker+apptainer container and is not justified as host/external"
            )

# 7: artifact schema keys declaration gate.
for sid, meta in domain_stages.items():
    status = str(meta["status"])
    required_outputs = list(meta["required_outputs"])
    ci_meta = ci_stages.get(sid, {})
    output_kinds = list(ci_meta.get("output_kinds", [])) if isinstance(ci_meta, dict) else []
    metrics_schema = str(ci_meta.get("metrics_schema", "")).strip() if isinstance(ci_meta, dict) else ""
    if status == "supported" and not required_outputs and not output_kinds and not metrics_schema:
        errors.append(
            f"stage '{sid}' is supported but has no artifact schema declaration "
            "(domain required_outputs and CI output_kinds/metrics_schema are empty)"
        )

# 8: refusal docs + reason code governance for unsupported/out_of_scope stages.
refusal_doc_text = REFUSAL_DOC.read_text(encoding="utf-8") if REFUSAL_DOC.exists() else ""
for sid, meta in domain_stages.items():
    status = str(meta["status"])
    if status != "out_of_scope":
        continue
    if sid not in refusal_doc_text:
        errors.append(
            f"out_of_scope stage '{sid}' missing refusal docs entry in {REFUSAL_DOC.relative_to(ROOT)}"
        )
    planned_notes = list(meta["planned_out_of_scope"])
    if not planned_notes:
        errors.append(f"out_of_scope stage '{sid}' missing planned_out_of_scope reason notes")

# 9: refuse-always marker must include issue id.
for path in ROOT.glob("crates/**/*.rs"):
    text = path.read_text(encoding="utf-8", errors="ignore")
    for ln, line in enumerate(text.splitlines(), start=1):
        l = line.lower()
        if "refusal" in l and "always" in l and "issue=" not in l:
            errors.append(
                f"{path.relative_to(ROOT)}:{ln}: refuse-always path must include issue=<ID> marker"
            )

# 10: readiness badge policy for stable profiles.
stable_stage_file = ROOT / "configs/ci/stages/stages.toml"
stable_cfg = tomllib.loads(stable_stage_file.read_text(encoding="utf-8"))
for row in stable_cfg.get("stages", []):
    sid = str(row.get("id", "")).strip()
    status = str(row.get("status", "")).strip()
    if not sid or status != "supported":
        continue
    readiness = registry_stages.get(sid)
    if readiness is None:
        continue
    if readiness not in {"supported", "certified"}:
        errors.append(
            f"stable profile stage '{sid}' has readiness '{readiness}', expected supported|certified"
        )

if errors:
    print("stage-registry-governance: FAILED", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("stage-registry-governance: OK")
PY
