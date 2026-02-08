#!/usr/bin/env python3
import json
import sys
from pathlib import Path


def percent(hit, total):
    if total == 0:
        return 100.0
    return 100.0 * hit / total


def fmt_pct(p):
    return f"{p:6.2f}%"


def main():
    if len(sys.argv) < 2:
        print("usage: coverage_summary.py <llvm-cov.json>", file=sys.stderr)
        sys.exit(2)
    data = json.loads(Path(sys.argv[1]).read_text())

    rows = []
    for file in data.get("data", []):
        for fn in file.get("functions", []):
            pass

    # Aggregate by package/crate name (from file path)
    crates = {}
    for f in data.get("data", []):
        for fn in f.get("functions", []):
            pass
    # llvm-cov json structure
    files = data.get("data", [])[0].get("files", []) if data.get("data") else []

    for f in files:
        path = f.get("filename", "")
        parts = Path(path).parts
        crate = "workspace"
        if "crates" in parts:
            idx = parts.index("crates")
            if idx + 1 < len(parts):
                crate = parts[idx + 1]
        cov = f.get("summary", {})
        lines = cov.get("lines", {})
        funcs = cov.get("functions", {})
        regions = cov.get("regions", {})
        entry = crates.setdefault(crate, {
            "lines_hit": 0, "lines_total": 0,
            "funcs_hit": 0, "funcs_total": 0,
            "regions_hit": 0, "regions_total": 0,
            "files": []
        })
        entry["lines_hit"] += lines.get("count", 0) - lines.get("missed", 0)
        entry["lines_total"] += lines.get("count", 0)
        entry["funcs_hit"] += funcs.get("count", 0) - funcs.get("missed", 0)
        entry["funcs_total"] += funcs.get("count", 0)
        entry["regions_hit"] += regions.get("count", 0) - regions.get("missed", 0)
        entry["regions_total"] += regions.get("count", 0)
        entry["files"].append((path, lines.get("missed", 0)))

    print("crate | lines % | funcs % | regions % | uncovered top files")
    print("----- | ------- | ------- | --------- | -------------------")
    for crate, entry in sorted(crates.items()):
        lines_pct = percent(entry["lines_hit"], entry["lines_total"])
        funcs_pct = percent(entry["funcs_hit"], entry["funcs_total"])
        regions_pct = percent(entry["regions_hit"], entry["regions_total"])
        top_files = sorted(entry["files"], key=lambda x: x[1], reverse=True)[:5]
        top_str = ", ".join([f"{Path(p).name}({m})" for p, m in top_files if m > 0])
        print(f"{crate} | {fmt_pct(lines_pct)} | {fmt_pct(funcs_pct)} | {fmt_pct(regions_pct)} | {top_str}")

if __name__ == "__main__":
    main()
