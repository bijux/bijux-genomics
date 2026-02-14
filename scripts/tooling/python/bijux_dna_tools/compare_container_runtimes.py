from __future__ import annotations

import json
from pathlib import Path
import re
import sys
try:
    import tomllib
except ModuleNotFoundError:  # pragma: no cover
    import tomli as tomllib


def load_manifests(path: Path) -> dict[str, dict]:
    items: dict[str, dict] = {}
    for mf in sorted(path.glob("*.json")):
        if mf.name in {"report.json", "summary.json"}:
            continue
        try:
            data = json.loads(mf.read_text(encoding="utf-8"))
        except Exception:
            continue
        tool = str(data.get("tool", "")).strip()
        if tool:
            items[tool] = data
    return items


def normalize_version(s: str) -> str:
    return re.sub(r"\s+", " ", s.strip().lower())


def load_expected_regexes(root: Path) -> dict[str, str]:
    reg_paths = [
        root / "configs/ci/registry/tool_registry.toml",
        root / "configs/ci/registry/tool_registry_vcf.toml",
        root / "configs/ci/registry/tool_registry_experimental.toml",
        root / "configs/ci/registry/tool_registry_vcf_downstream.toml",
    ]
    out: dict[str, str] = {}
    for rp in reg_paths:
        if not rp.exists():
            continue
        data = tomllib.loads(rp.read_text(encoding="utf-8"))
        for row in data.get("tools", []):
            if not isinstance(row, dict):
                continue
            tool = str(row.get("id") or row.get("tool_id") or "").strip()
            rx = str(row.get("expected_version_regex") or "").strip()
            if tool and rx:
                out[tool] = rx
    return out


def main() -> int:
    if len(sys.argv) != 3:
        print("usage: compare_container_runtimes.py <docker-artifacts-dir> <apptainer-artifacts-dir>", file=sys.stderr)
        return 2

    docker_dir = Path(sys.argv[1])
    appt_dir = Path(sys.argv[2])
    if not docker_dir.exists() or not appt_dir.exists():
        print("missing runtime manifest directories", file=sys.stderr)
        return 1

    docker = load_manifests(docker_dir)
    appt = load_manifests(appt_dir)
    root = Path(__file__).resolve().parents[3]
    expected_regex = load_expected_regexes(root)
    shared = sorted(set(docker) & set(appt))
    if not shared:
        print("no shared tool manifests to compare", file=sys.stderr)
        return 1

    errors: list[str] = []
    for tool in shared:
        d = docker[tool]
        a = appt[tool]
        if d.get("status") != "ok" or a.get("status") != "ok":
            errors.append(f"{tool}: non-ok status docker={d.get('status')} apptainer={a.get('status')}")
            continue
        dv = normalize_version(str(d.get("version_output", "")))
        av = normalize_version(str(a.get("version_output", "")))
        if not dv or not av:
            errors.append(f"{tool}: missing version_output in one runtime")
        elif dv != av:
            errors.append(f"{tool}: version_output mismatch docker='{dv}' apptainer='{av}'")
        regex = expected_regex.get(tool, r"v?[0-9]+\.[0-9]+([.-][0-9A-Za-z]+)?")
        if dv and not re.search(regex, dv, flags=re.I):
            errors.append(f"{tool}: docker version_output does not match expected pattern '{regex}'")
        if av and not re.search(regex, av, flags=re.I):
            errors.append(f"{tool}: apptainer version_output does not match expected pattern '{regex}'")
        # basic behavior parity: help/minimal/negative exit contracts
        for key in ("help_actual_exit_code", "minimal_actual_exit_code", "negative_actual_exit_code"):
            if str(d.get(key, "")) != str(a.get(key, "")):
                errors.append(f"{tool}: {key} mismatch docker={d.get(key)} apptainer={a.get(key)}")

    if errors:
        print("container runtime parity: FAILED", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 1

    print(f"container runtime parity: OK ({len(shared)} shared tools)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
