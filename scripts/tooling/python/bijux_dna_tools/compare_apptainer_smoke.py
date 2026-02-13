from __future__ import annotations

import json
from pathlib import Path
import sys


def load_statuses(base: Path) -> dict[str, str]:
    statuses: dict[str, str] = {}
    for manifest_file in sorted(base.glob("*.json")):
        if manifest_file.name in {"report.json", "summary.json"}:
            continue
        payload = json.loads(manifest_file.read_text(encoding="utf-8"))
        tool = payload.get("tool")
        status = payload.get("status")
        if tool:
            statuses[str(tool)] = str(status)
    return statuses


def main() -> int:
    root = Path(sys.argv[1] if len(sys.argv) > 1 else "artifacts/container")
    left_dir = root / "apptainer-bijux-run"
    right_dir = root / "apptainer-apptainer-run"
    if not left_dir.exists() or not right_dir.exists():
        print("missing smoke artifact dirs for compare", file=sys.stderr)
        return 1

    left = load_statuses(left_dir)
    right = load_statuses(right_dir)

    missing_left = sorted(set(right) - set(left))
    missing_right = sorted(set(left) - set(right))
    mismatch = sorted(t for t in set(left) & set(right) if left[t] != right[t])

    if missing_left or missing_right or mismatch:
        print("smoke mode mismatch detected")
        if missing_left:
            print("missing in bijux-run:", ",".join(missing_left))
        if missing_right:
            print("missing in apptainer-run:", ",".join(missing_right))
        if mismatch:
            print("status mismatch:", ",".join(f"{t}:{left[t]}!={right[t]}" for t in mismatch))
        return 1

    print(f"smoke mode compare OK for {len(left)} tools")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
