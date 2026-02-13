#!/usr/bin/env python3
"""Validate repository config contracts defined under configs/."""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import sys
try:
    import tomllib  # type: ignore[attr-defined]
except ModuleNotFoundError:  # pragma: no cover
    try:
        import tomli as tomllib  # type: ignore[no-redef]
    except ModuleNotFoundError:
        import toml as tomllib  # type: ignore[no-redef]


ROOT = pathlib.Path(__file__).resolve().parents[2]
CONFIGS = ROOT / "configs"
REGISTRY_DIR = CONFIGS / "ci" / "registry"
FLOATING = {"latest", "main", "master", ""}
ALLOWED_TOOL_STATUS = {"production", "experimental", "planned"}
HEADER_KEYS = ("schema_version", "owner", "purpose", "authority", "stability", "last_updated")


def load_toml(path: pathlib.Path) -> dict:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover
        line = getattr(exc, "lineno", None)
        col = getattr(exc, "colno", None)
        where = f" line {line} col {col}" if line is not None and col is not None else ""
        raise ValueError(f"{path}: invalid TOML{where} ({exc})") from exc


def check_headers(path: pathlib.Path, errs: list[str]) -> None:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    head = lines[:16]
    for key in HEADER_KEYS:
        if not any(l.startswith(f"# {key} = ") for l in head):
            errs.append(f"{path}: missing '# {key} = ...' in first 16 lines")


def parse_date(value: str, path: pathlib.Path, field: str, errs: list[str]) -> dt.date | None:
    try:
        return dt.date.fromisoformat(value)
    except Exception:
        errs.append(f"{path}: {field} must be ISO date YYYY-MM-DD (got '{value}')")
        return None


def check_registries(errs: list[str]) -> tuple[set[str], set[str]]:
    known_tools: set[str] = set()
    known_stages: set[str] = set()
    for reg in (
        REGISTRY_DIR / "tool_registry.toml",
        REGISTRY_DIR / "tool_registry_experimental.toml",
        REGISTRY_DIR / "tool_registry_vcf.toml",
        REGISTRY_DIR / "tool_registry_vcf_downstream.toml",
    ):
        data = load_toml(reg)
        for tool in data.get("tools", []):
            tool_id = str(tool.get("id") or tool.get("tool_id") or "<unknown>")
            known_tools.add(tool_id)
            status = str(tool.get("status", "")).strip()
            if status not in ALLOWED_TOOL_STATUS:
                errs.append(
                    f"{reg}: tool '{tool_id}' status must be one of {sorted(ALLOWED_TOOL_STATUS)} (got '{status}')"
                )
            for stage in tool.get("stage_ids", []):
                known_stages.add(str(stage))
            version = str(tool.get("version", "")).strip()
            if version.lower() in FLOATING:
                errs.append(f"{reg}: tool '{tool_id}' has floating version '{version}'")
            default_version = str(tool.get("default_version", "")).strip()
            if default_version.lower() in FLOATING:
                errs.append(
                    f"{reg}: tool '{tool_id}' has floating default_version '{default_version}'"
                )
    return known_tools, known_stages


def check_deprecations(known_tools: set[str], known_stages: set[str], errs: list[str]) -> None:
    dep_path = REGISTRY_DIR / "deprecations.toml"
    data = load_toml(dep_path)
    rows = data.get("deprecations", [])
    seen = set()
    today = dt.date.today()
    for row in rows:
        tool_id = str(row.get("tool_id", "")).strip()
        stage = str(row.get("stage", "")).strip()
        deprecated_since = str(row.get("deprecated_since", "")).strip()
        removal_after = str(row.get("removal_after", "")).strip()
        rationale = str(row.get("rationale", "")).strip()
        replacement = str(row.get("replacement", "")).strip()
        if not tool_id:
            errs.append(f"{dep_path}: deprecation entry missing tool_id")
        elif tool_id not in known_tools:
            errs.append(f"{dep_path}: unknown tool_id '{tool_id}'")
        if not stage:
            errs.append(f"{dep_path}: deprecation entry for tool '{tool_id}' missing stage")
        elif stage not in known_stages:
            errs.append(f"{dep_path}: unknown stage '{stage}' for tool '{tool_id}'")
        if not rationale:
            errs.append(f"{dep_path}: deprecation entry for tool '{tool_id}' missing rationale")
        if not replacement:
            errs.append(f"{dep_path}: deprecation entry for tool '{tool_id}' missing replacement")
        since_d = parse_date(deprecated_since, dep_path, "deprecated_since", errs)
        after_d = parse_date(removal_after, dep_path, "removal_after", errs)
        if since_d and after_d and after_d <= since_d:
            errs.append(
                f"{dep_path}: tool '{tool_id}' stage '{stage}' has removal_after <= deprecated_since"
            )
        key = (tool_id, stage)
        if key in seen:
            errs.append(f"{dep_path}: duplicate deprecation entry for tool '{tool_id}' stage '{stage}'")
        seen.add(key)


def check_deprecation_references(errs: list[str]) -> None:
    dep_path = REGISTRY_DIR / "deprecations.toml"
    data = load_toml(dep_path)
    rows = data.get("deprecations", [])
    today = dt.date.today()

    required_tools: set[str] = set()
    for path in (
        CONFIGS / "ci" / "tools" / "required_tools.toml",
        CONFIGS / "ci" / "tools" / "required_tools_vcf.toml",
        CONFIGS / "ci" / "tools" / "required_tools_vcf_downstream.toml",
    ):
        cfg = load_toml(path)
        required_tools |= {str(t) for t in cfg.get("required_tools", [])}

    declared_stages: set[str] = set()
    for path in (
        CONFIGS / "ci" / "stages" / "stages.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf_downstream.toml",
    ):
        cfg = load_toml(path)
        for row in cfg.get("stages", []):
            sid = str(row.get("id", "")).strip()
            if sid:
                declared_stages.add(sid)

    param_stages: set[str] = set()
    for path in (
        CONFIGS / "ci" / "params" / "param_registry.toml",
        CONFIGS / "ci" / "params" / "param_registry_vcf.toml",
        CONFIGS / "ci" / "params" / "param_registry_downstream.toml",
    ):
        cfg = load_toml(path)
        for row in cfg.get("entries", []) + cfg.get("params", []):
            sid = str(row.get("stage_id", "")).strip()
            if sid:
                param_stages.add(sid)

    for row in rows:
        tool_id = str(row.get("tool_id", "")).strip()
        stage = str(row.get("stage", "")).strip()
        removal_after = parse_date(str(row.get("removal_after", "")).strip(), dep_path, "removal_after", errs)
        if not removal_after or today <= removal_after:
            continue
        if tool_id and tool_id in required_tools:
            errs.append(f"{dep_path}: deprecated tool '{tool_id}' past removal_after is still required")
        if stage and stage in declared_stages:
            errs.append(f"{dep_path}: deprecated stage '{stage}' past removal_after is still declared")
        if stage and stage in param_stages:
            errs.append(f"{dep_path}: deprecated stage '{stage}' past removal_after still appears in param registries")


def check_required_tools_registry_parity(errs: list[str]) -> None:
    known_tools: set[str] = set()
    for reg in (
        REGISTRY_DIR / "tool_registry.toml",
        REGISTRY_DIR / "tool_registry_experimental.toml",
        REGISTRY_DIR / "tool_registry_vcf.toml",
        REGISTRY_DIR / "tool_registry_vcf_downstream.toml",
    ):
        data = load_toml(reg)
        for row in data.get("tools", []):
            tid = str(row.get("id") or row.get("tool_id") or "").strip()
            if tid:
                known_tools.add(tid)

    for req in (
        CONFIGS / "ci" / "tools" / "required_tools.toml",
        CONFIGS / "ci" / "tools" / "required_tools_vcf.toml",
        CONFIGS / "ci" / "tools" / "required_tools_vcf_downstream.toml",
    ):
        data = load_toml(req)
        for tid in data.get("required_tools", []):
            if str(tid) not in known_tools:
                errs.append(f"{req}: required_tools entry '{tid}' has no registry definition")


def check_stage_domain_parity(errs: list[str]) -> None:
    stage_ids: set[str] = set()
    for p in (
        CONFIGS / "ci" / "stages" / "stages.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf_downstream.toml",
    ):
        d = load_toml(p)
        for row in d.get("stages", []):
            sid = str(row.get("id", "")).strip()
            if sid:
                stage_ids.add(sid)

    domain_stage_ids: set[str] = set()
    for f in (ROOT / "domain").glob("*/*/*.yaml"):
        if f.parts[-2] != "stages" or f.name.startswith("_"):
            continue
        text = f.read_text(encoding="utf-8")
        for line in text.splitlines():
            if line.startswith("stage_id:"):
                value = line.split(":", 1)[1].strip().strip("\"'")
                if value:
                    domain_stage_ids.add(value)
                break

    for sid in sorted(stage_ids - domain_stage_ids):
        errs.append(f"configs/ci/stages: stage '{sid}' not found under domain/**/stages/*.yaml")


def check_runtime_platforms(errs: list[str]) -> None:
    path = CONFIGS / "runtime" / "platforms.toml"
    data = load_toml(path)
    allowed_top = {"default", "platforms"}
    unknown_top = set(data.keys()) - allowed_top
    if unknown_top:
        errs.append(f"{path}: unknown top-level keys: {sorted(unknown_top)}")
    default = str(data.get("default", "")).strip()
    if not default:
        errs.append(f"{path}: missing non-empty top-level 'default'")
    platforms = data.get("platforms")
    if not isinstance(platforms, dict) or not platforms:
        errs.append(f"{path}: [platforms] table must exist and be non-empty")
        return
    if default and default not in platforms:
        errs.append(f"{path}: default platform '{default}' not present under [platforms]")
    allowed_platform_keys = {"runner", "container_dir", "image_prefix", "arch", "runtime", "notes"}
    for pid, cfg in platforms.items():
        if not isinstance(cfg, dict):
            errs.append(f"{path}: platforms.{pid} must be a table")
            continue
        unknown = set(cfg.keys()) - allowed_platform_keys
        if unknown:
            errs.append(f"{path}: platforms.{pid} has unknown keys: {sorted(unknown)}")
        if not (cfg.get("runner") or cfg.get("runtime")):
            errs.append(f"{path}: platforms.{pid} requires one of 'runner' or 'runtime'")


def _collect_stage_ids_from_stage_toml(path: pathlib.Path, errs: list[str]) -> dict[str, str]:
    data = load_toml(path)
    rows = data.get("stages", [])
    out: dict[str, str] = {}
    for row in rows:
        sid = str(row.get("id", "")).strip()
        status = str(row.get("status", "")).strip()
        if not sid:
            errs.append(f"{path}: stage entry missing id")
            continue
        if not status:
            errs.append(f"{path}: stage '{sid}' missing status")
            continue
        out[sid] = status
    return out


def _collect_param_stage_ids(path: pathlib.Path, errs: list[str]) -> set[str]:
    data = load_toml(path)
    ids: set[str] = set()
    rows_entries = data.get("entries", [])
    rows_params = data.get("params", [])
    for row in rows_entries + rows_params:
        sid = str(row.get("stage_id", "")).strip()
        if not sid:
            errs.append(f"{path}: parameter entry missing stage_id")
            continue
        ids.add(sid)
    return ids


def check_stage_param_coverage(errs: list[str]) -> None:
    stage_files = (
        CONFIGS / "ci" / "stages" / "stages.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf.toml",
        CONFIGS / "ci" / "stages" / "stages_vcf_downstream.toml",
    )
    param_files = (
        CONFIGS / "ci" / "params" / "param_registry.toml",
        CONFIGS / "ci" / "params" / "param_registry_vcf.toml",
        CONFIGS / "ci" / "params" / "param_registry_downstream.toml",
    )
    for p in stage_files + param_files:
        if not p.exists():
            errs.append(f"{p}: missing required config file")
            return

    stages: dict[str, str] = {}
    for p in stage_files:
        stages.update(_collect_stage_ids_from_stage_toml(p, errs))

    params: set[str] = set()
    for p in param_files:
        params |= _collect_param_stage_ids(p, errs)

    for sid, status in sorted(stages.items()):
        if status not in {"production", "supported"}:
            continue
        if sid not in params:
            errs.append(
                f"configs/ci/params: missing parameter registry entry for production stage '{sid}'"
            )


def check_images_contract(errs: list[str]) -> None:
    path = CONFIGS / "ci" / "tools" / "images.toml"
    data = load_toml(path)
    for key, value in data.items():
        if not isinstance(value, dict):
            continue
        enabled = value.get("enabled", None)
        if enabled is not None and not isinstance(enabled, bool):
            errs.append(f"{path}: [{key}] enabled must be boolean")

    planned_tools: set[str] = set()
    for reg in (
        REGISTRY_DIR / "tool_registry.toml",
        REGISTRY_DIR / "tool_registry_experimental.toml",
        REGISTRY_DIR / "tool_registry_vcf.toml",
        REGISTRY_DIR / "tool_registry_vcf_downstream.toml",
    ):
        rdata = load_toml(reg)
        for tool in rdata.get("tools", []):
            tid = str(tool.get("id") or tool.get("tool_id") or "").strip()
            status = str(tool.get("status") or "").strip()
            if tid and status == "planned":
                planned_tools.add(tid)

    for tid in sorted(planned_tools):
        entry = data.get(tid)
        if not isinstance(entry, dict):
            errs.append(f"{path}: planned tool '{tid}' must have a section with enabled=false")
            continue
        if entry.get("enabled") is not False:
            errs.append(f"{path}: planned tool '{tid}' must set enabled=false")


def main() -> int:
    parser = argparse.ArgumentParser(description="Validate configs contract rules.")
    parser.add_argument("--root", default=str(ROOT), help="Repo root path (default: auto-detected)")
    args = parser.parse_args()

    root = pathlib.Path(args.root).resolve()
    configs = root / "configs"
    errs: list[str] = []

    for path in sorted(configs.rglob("*")):
        if not path.is_file():
            continue
        if path.suffix not in {".toml", ".yaml", ".yml"}:
            continue
        check_headers(path, errs)
        if path.suffix == ".toml":
            try:
                load_toml(path)
            except ValueError as exc:
                errs.append(str(exc))

    known_tools, known_stages = check_registries(errs)
    check_deprecations(known_tools, known_stages, errs)
    check_deprecation_references(errs)
    check_required_tools_registry_parity(errs)
    check_stage_domain_parity(errs)
    check_runtime_platforms(errs)
    check_stage_param_coverage(errs)
    check_images_contract(errs)

    if errs:
        print("config-schema: validation failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("config-schema: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
