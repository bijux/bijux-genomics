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

    # llvm-cov json structure (single report entry)
    report = data.get("data", [])[0] if data.get("data") else {}
    files = report.get("files", [])

    # Aggregate by package/crate name (from file path)
    crates = {}
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
        lines_count = lines.get("count", 0)
        lines_covered = lines.get("covered", 0)
        lines_uncovered = lines.get("notcovered", max(lines_count - lines_covered, 0))
        funcs_count = funcs.get("count", 0)
        funcs_covered = funcs.get("covered", 0)
        funcs_uncovered = funcs.get("notcovered", max(funcs_count - funcs_covered, 0))
        regions_count = regions.get("count", 0)
        regions_covered = regions.get("covered", 0)
        regions_uncovered = regions.get("notcovered", max(regions_count - regions_covered, 0))

        # Fallback when only count is provided.
        if lines_count and not lines_covered and not lines_uncovered:
            lines_covered = lines_count
        if funcs_count and not funcs_covered and not funcs_uncovered:
            funcs_covered = funcs_count
        if regions_count and not regions_covered and not regions_uncovered:
            regions_covered = regions_count

        entry["lines_hit"] += lines_covered
        entry["lines_total"] += lines_count
        entry["funcs_hit"] += funcs_covered
        entry["funcs_total"] += funcs_count
        entry["regions_hit"] += regions_covered
        entry["regions_total"] += regions_count
        entry["files"].append((path, lines_uncovered))

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
