#!/usr/bin/env python3
import json
import os
import sys


def load_json(path):
    if path == "-":
        return json.load(sys.stdin)
    with open(path, "r", encoding="utf-8") as f:
        return json.load(f)


def percent(covered, count):
    if count == 0:
        return 100.0
    return (covered / count) * 100.0


def relpath(path):
    try:
        return os.path.relpath(path, os.getcwd())
    except Exception:
        return path


def crate_from_path(path):
    parts = relpath(path).split(os.sep)
    if len(parts) >= 2 and parts[0] == "crates":
        return parts[1]
    return None


def collect_files(data):
    files = []
    for entry in data.get("data", []):
        for file_entry in entry.get("files", []):
            filename = file_entry.get("filename")
            summary = file_entry.get("summary", {})
            files.append((filename, summary))
    return files


def summarize(files):
    crates = {}
    total = {
        "lines": {"covered": 0, "count": 0},
        "functions": {"covered": 0, "count": 0},
        "regions": {"covered": 0, "count": 0},
        "files": [],
    }
    for filename, summary in files:
        if not filename:
            continue
        crate = crate_from_path(filename)
        if crate is None:
            continue
        if crate not in crates:
            crates[crate] = {
                "lines": {"covered": 0, "count": 0},
                "functions": {"covered": 0, "count": 0},
                "regions": {"covered": 0, "count": 0},
                "files": [],
            }
        for key in ("lines", "functions", "regions"):
            metrics = summary.get(key, {})
            covered = metrics.get("covered", 0)
            count = metrics.get("count", 0)
            crates[crate][key]["covered"] += covered
            crates[crate][key]["count"] += count
            total[key]["covered"] += covered
            total[key]["count"] += count
        line_metrics = summary.get("lines", {})
        crates[crate]["files"].append(
            (filename, percent(line_metrics.get("covered", 0), line_metrics.get("count", 0)))
        )
        total["files"].append(
            (filename, percent(line_metrics.get("covered", 0), line_metrics.get("count", 0)))
        )
    return crates, total


def top_uncovered(files, limit=5):
    ranked = sorted(files, key=lambda item: item[1])
    top = []
    for filename, pct in ranked:
        if pct >= 100.0:
            continue
        top.append((filename, pct))
        if len(top) >= limit:
            break
    return top


def format_top(files, limit=5):
    top = top_uncovered(files, limit=limit)
    if not top:
        return "-"
    formatted = []
    for filename, pct in top:
        formatted.append(f"{relpath(filename)} ({pct:.1f}%)")
    return "; ".join(formatted)


def print_table(crates, total):
    rows = []
    for name in sorted(crates):
        data = crates[name]
        rows.append(
            (
                name,
                percent(data["lines"]["covered"], data["lines"]["count"]),
                percent(data["functions"]["covered"], data["functions"]["count"]),
                percent(data["regions"]["covered"], data["regions"]["count"]),
                format_top(data["files"]),
            )
        )
    rows.append(
        (
            "TOTAL",
            percent(total["lines"]["covered"], total["lines"]["count"]),
            percent(total["functions"]["covered"], total["functions"]["count"]),
            percent(total["regions"]["covered"], total["regions"]["count"]),
            format_top(total["files"]),
        )
    )

    headers = ("crate", "lines %", "funcs %", "regions %", "uncovered top files (top 5)")
    widths = [len(h) for h in headers]
    for row in rows:
        widths[0] = max(widths[0], len(row[0]))
        widths[1] = max(widths[1], len(f"{row[1]:.1f}"))
        widths[2] = max(widths[2], len(f"{row[2]:.1f}"))
        widths[3] = max(widths[3], len(f"{row[3]:.1f}"))
        widths[4] = max(widths[4], len(row[4]))

    def fmt_row(cols):
        return " | ".join(
            [
                f"{cols[0]:<{widths[0]}}",
                f"{cols[1]:>{widths[1]}.1f}",
                f"{cols[2]:>{widths[2]}.1f}",
                f"{cols[3]:>{widths[3]}.1f}",
                f"{cols[4]:<{widths[4]}}",
            ]
        )

    print(fmt_row(headers))
    print(
        "-+-".join(
            [
                "-" * widths[0],
                "-" * widths[1],
                "-" * widths[2],
                "-" * widths[3],
                "-" * widths[4],
            ]
        )
    )
    for row in rows:
        print(fmt_row(row))


def main():
    # TODO(workspace): add coverage thresholds once gates are defined.
    if len(sys.argv) != 2:
        print("Usage: coverage_summary.py <llvm-cov-report.json or ->", file=sys.stderr)
        sys.exit(2)
    data = load_json(sys.argv[1])
    files = collect_files(data)
    crates, total = summarize(files)
    print_table(crates, total)


if __name__ == "__main__":
    main()
