"""Execute every catalogued Python lane through the existing case receipt harness."""

from __future__ import annotations

import json
import os
from pathlib import Path
import subprocess
import sys


def selections(lane):
    excluded = set(lane["excluded_modules"])
    modules = set(lane["explicit_modules"])
    if lane["harness"] == "pytest":
        return lane["discovery_roots"]
    for root in lane["discovery_roots"]:
        modules.update(".".join(path.with_suffix("").parts)
                       for path in Path(root).rglob("test*.py"))
    return sorted(modules - excluded)


def main():
    root = Path(__file__).resolve().parents[2]
    os.chdir(root)
    output = root / "artifacts/ci/distributed/python"
    output.mkdir(parents=True, exist_ok=True)
    summary = {"all_passed": False, "source_commit": subprocess.check_output(
        ["git", "rev-parse", "HEAD"], text=True).strip(), "lanes": []}
    receipt = output / "receipt.json"
    receipt.write_text(json.dumps(summary, indent=2) + "\n")
    catalog = json.loads((root / "configs/ci/local-test-lanes.json").read_text())
    lanes = [lane for lane in catalog["lanes"] if lane["harness"] in ("pytest", "unittest")]
    if not lanes:
        raise ValueError("local catalog declares no Python test lanes")
    for lane in lanes:
        lane_output = output / lane["id"]
        lane_output.mkdir(exist_ok=True)
        args = [sys.executable, "-B", "makes/bin/python_test_receipt.py",
                "--source-root", str(root), "--scratch", str(lane_output / "scratch"),
                "--harness", lane["harness"]]
        expected = lane_output / "expected.json"
        actual = lane_output / "actual.json"
        subprocess.run([*args, "--discover", "--output", str(expected), *selections(lane)], check=True)
        subprocess.run([*args, "--expected", str(expected), "--output", str(actual), *selections(lane)], check=True)
        summary["lanes"].append({"id": lane["id"], "receipt": str(actual.relative_to(root))})
    summary["all_passed"] = True
    receipt.write_text(json.dumps(summary, indent=2) + "\n")


if __name__ == "__main__":
    main()
