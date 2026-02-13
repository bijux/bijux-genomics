from __future__ import annotations

import json
from pathlib import Path
import re
import sys


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
