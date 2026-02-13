#!/usr/bin/env python3
"""Validate repository config contracts defined under configs/."""

from __future__ import annotations

import argparse
import datetime as dt
import pathlib
import re
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


def load_toml(path: pathlib.Path) -> dict:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:  # pragma: no cover
        raise ValueError(f"{path}: invalid TOML ({exc})") from exc


def check_headers(path: pathlib.Path, errs: list[str]) -> None:
    text = path.read_text(encoding="utf-8")
    lines = text.splitlines()
    head = lines[:12]
    has_schema = any(l.startswith("# schema_version = ") for l in head)
    has_owner = any(l.startswith("# owner = ") for l in head)
    if not (has_schema and has_owner):
        errs.append(f"{path}: missing schema header in first 12 lines (# schema_version / # owner)")


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
    ):
        data = load_toml(reg)
        for tool in data.get("tools", []):
            tool_id = str(tool.get("id") or tool.get("tool_id") or "<unknown>")
            known_tools.add(tool_id)
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
    for row in rows:
        tool_id = str(row.get("tool_id", "")).strip()
        stage = str(row.get("stage", "")).strip()
        deprecated_since = str(row.get("deprecated_since", "")).strip()
        removal_after = str(row.get("removal_after", "")).strip()
        rationale = str(row.get("rationale", "")).strip()
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

    known_tools, known_stages = check_registries(errs)
    check_deprecations(known_tools, known_stages, errs)

    if errs:
        print("config-schema: validation failed", file=sys.stderr)
        for err in errs:
            print(f"- {err}", file=sys.stderr)
        return 1
    print("config-schema: OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
