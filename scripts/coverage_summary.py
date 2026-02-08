#!/usr/bin/env python3
import argparse
import json
import os
import sys
from pathlib import Path


def percent(hit, total):
    if total == 0:
        return 100.0
    return 100.0 * hit / total


def fmt_pct(p):
    return f"{p:6.2f}%"


def parse_args():
    parser = argparse.ArgumentParser(description="Summarize llvm-cov JSON output.")
    parser.add_argument("report", help="Path to llvm-cov JSON report")
    parser.add_argument(
        "--show-uncovered",
        action="store_true",
        help="Print uncovered file list per crate (off by default)",
    )
    parser.add_argument(
        "--verbose",
        action="store_true",
        help="Alias for --show-uncovered",
    )
    parser.add_argument(
        "--baseline",
        help="Path to baseline llvm-cov JSON report for delta comparison",
    )
    parser.add_argument(
        "--check-thresholds",
        help="Path to JSON file of per-crate coverage thresholds",
    )
    parser.add_argument(
        "--show-worst",
        action="store_true",
        help="Print worst offenders table",
    )
    parser.add_argument(
        "--worst-count",
        type=int,
        default=20,
        help="Number of worst offenders to show (default: 20)",
    )
    return parser.parse_args()


def load_report(path):
    data = json.loads(Path(path).read_text())
    report = data.get("data", [])[0] if data.get("data") else {}
    files = report.get("files", [])
    crates = {}
    crate_name_cache = {}
    for f in files:
        path = f.get("filename", "")
        parts = Path(path).parts
        crate = "workspace"
        if "crates" in parts:
            idx = parts.index("crates")
            if idx + 1 < len(parts):
                crate = parts[idx + 1]
                if crate not in crate_name_cache:
                    manifest = Path("crates") / crate / "Cargo.toml"
                    if manifest.exists():
                        for line in manifest.read_text().splitlines():
                            stripped = line.strip()
                            if stripped.startswith("name ="):
                                crate_name_cache[crate] = stripped.split("=", 1)[1].strip().strip('"')
                                break
                    if crate not in crate_name_cache:
                        crate_name_cache[crate] = crate
                crate = crate_name_cache[crate]
        cov = f.get("summary", {})
        lines = cov.get("lines", {})
        funcs = cov.get("functions", {})
        regions = cov.get("regions", {})
        entry = crates.setdefault(
            crate,
            {
                "lines_hit": 0,
                "lines_total": 0,
                "funcs_hit": 0,
                "funcs_total": 0,
                "regions_hit": 0,
                "regions_total": 0,
                "files": [],
            },
        )
        lines_count = lines.get("count", 0)
        lines_covered = lines.get("covered", 0)
        lines_uncovered = lines.get("notcovered", max(lines_count - lines_covered, 0))
        funcs_count = funcs.get("count", 0)
        funcs_covered = funcs.get("covered", 0)
        funcs_uncovered = funcs.get("notcovered", max(funcs_count - funcs_covered, 0))
        regions_count = regions.get("count", 0)
        regions_covered = regions.get("covered", 0)
        regions_uncovered = regions.get("notcovered", max(regions_count - regions_covered, 0))

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
    return crates


def main():
    args = parse_args()
    data = load_report(args.report)
    baseline = load_report(args.baseline) if args.baseline else None

    show_uncovered = args.show_uncovered or args.verbose or os.getenv("COVERAGE_SHOW_UNCOVERED") == "1"
    show_worst = args.show_worst or os.getenv("COVERAGE_SHOW_WORST") == "1"

    header = "crate | lines | covered | lines % | funcs % | regions %"
    if baseline:
        header += " | lines Δ"
    header += " | uncovered top files"
    print(header)
    print(
        "----- | ----- | ------- | ------- | ------- | --------- | ------- | -------------------"
        if baseline
        else "----- | ----- | ------- | ------- | ------- | --------- | -------------------"
    )

    rows = []
    for crate, entry in sorted(data.items()):
        lines_pct = percent(entry["lines_hit"], entry["lines_total"])
        funcs_pct = percent(entry["funcs_hit"], entry["funcs_total"])
        regions_pct = percent(entry["regions_hit"], entry["regions_total"])
        top_files = sorted(entry["files"], key=lambda x: x[1], reverse=True)[:5]
        top_str = ", ".join([f"{Path(p).name}({m})" for p, m in top_files if m > 0])
        delta = ""
        if baseline and crate in baseline:
            base = baseline[crate]
            base_pct = percent(base["lines_hit"], base["lines_total"])
            delta = f"{lines_pct - base_pct:+.2f}%"
        rows.append((crate, lines_pct, funcs_pct, regions_pct, delta, top_str, entry))

    for crate, lines_pct, funcs_pct, regions_pct, delta, top_str, entry in rows:
        lines_total = entry["lines_total"]
        lines_hit = entry["lines_hit"]
        if baseline:
            print(
                f"{crate} | {lines_total:>5} | {lines_hit:>7} | {fmt_pct(lines_pct)} | {fmt_pct(funcs_pct)} | {fmt_pct(regions_pct)} | {delta:>7} | {top_str}"
            )
        else:
            print(
                f"{crate} | {lines_total:>5} | {lines_hit:>7} | {fmt_pct(lines_pct)} | {fmt_pct(funcs_pct)} | {fmt_pct(regions_pct)} | {top_str}"
            )
        if show_uncovered:
            for path, misses in sorted(entry["files"], key=lambda x: x[1], reverse=True):
                if misses <= 0:
                    continue
                print(f"  - {path} ({misses} lines)")

    if show_worst:
        worst = sorted(rows, key=lambda r: r[1])[: args.worst_count]
        print("\nworst coverage (lines %):")
        for crate, lines_pct, funcs_pct, regions_pct, _delta, _top, _entry in worst:
            print(f"{crate}: {fmt_pct(lines_pct)}")

    if args.check_thresholds:
        thresholds = json.loads(Path(args.check_thresholds).read_text())
        default_threshold = thresholds.get("default", 0.0)
        class_thresholds = thresholds.get("classes", {})
        class_map = thresholds.get("crate_class", {})
        overrides = thresholds.get("overrides", {})
        failures = []
        for crate, entry in data.items():
            lines_pct = percent(entry["lines_hit"], entry["lines_total"])
            if crate in overrides:
                min_pct = overrides[crate]
            else:
                class_name = class_map.get(crate)
                min_pct = class_thresholds.get(class_name, default_threshold)
            if lines_pct < min_pct:
                failures.append((crate, lines_pct, min_pct))
        if failures:
            print("\ncoverage thresholds failed:")
            for crate, actual, minimum in failures:
                print(f"{crate}: {actual:.2f}% < {minimum:.2f}%")
            sys.exit(1)

if __name__ == "__main__":
    main()
