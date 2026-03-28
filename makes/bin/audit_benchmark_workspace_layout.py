#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

import corpus_01_fastq_benchmark_support as support


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Audit the governed local benchmark workspace layout."
    )
    parser.add_argument(
        "--json-out",
        default="docs/benchmark/workspace-layout-status.json",
    )
    parser.add_argument(
        "--markdown-out",
        default="docs/benchmark/workspace-layout-status.md",
    )
    return parser.parse_args()


def workspace_layout_report() -> dict:
    local_results_root = support.benchmark_local_results_root()
    local_cache_mirror_root = support.benchmark_local_cache_mirror_root()
    remote_workspace_root = local_cache_mirror_root.parent
    issues: list[dict[str, str]] = []

    duplicate_pairs = [
        (
            "duplicate-remote-results-root",
            local_cache_mirror_root / "results",
            local_cache_mirror_root / "bijux-dna-results",
        ),
        (
            "duplicate-remote-reference-root",
            local_cache_mirror_root / "reference",
            local_cache_mirror_root / "bijux-reference",
        ),
    ]
    for issue_id, canonical_root, legacy_root in duplicate_pairs:
        if canonical_root.exists() and legacy_root.exists():
            issues.append(
                {
                    "issue_id": issue_id,
                    "detail": f"both {canonical_root} and {legacy_root} exist",
                }
            )

    for sibling_name in ["results", "corpus_01", "extra-data"]:
        sibling_root = remote_workspace_root / sibling_name
        if sibling_root.exists():
            issues.append(
                {
                    "issue_id": "unexpected-remote-workspace-sibling",
                    "detail": f"unexpected sibling beside .cache: {sibling_root}",
                }
            )

    return {
        "local_results_root": str(local_results_root),
        "local_cache_mirror_root": str(local_cache_mirror_root),
        "remote_workspace_root": str(remote_workspace_root),
        "status": "clear" if not issues else "incomplete",
        "issue_count": len(issues),
        "issues": issues,
    }


def render_markdown(report: dict) -> str:
    lines = [
        "# Benchmark Workspace Layout Status",
        "",
        f"- Local results root: `{report['local_results_root']}`",
        f"- Local cache mirror root: `{report['local_cache_mirror_root']}`",
        f"- Mirrored remote workspace root: `{report['remote_workspace_root']}`",
        f"- Status: `{report['status']}`",
        f"- Issues: `{report['issue_count']}`",
        "",
    ]
    for issue in report["issues"]:
        lines.append(f"- `{issue['issue_id']}`: {issue['detail']}")
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    report = workspace_layout_report()
    json_out = Path(args.json_out).resolve()
    markdown_out = Path(args.markdown_out).resolve()
    json_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
    markdown_out.write_text(render_markdown(report), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
