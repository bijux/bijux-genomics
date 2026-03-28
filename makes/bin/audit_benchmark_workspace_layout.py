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


def _entry_names(root: Path) -> list[str]:
    if not root.is_dir():
        return []
    return sorted(path.name for path in root.iterdir())


def summarize_root_pair(*, scope: str, canonical_root: Path, legacy_root: Path) -> dict:
    canonical_entries = _entry_names(canonical_root)
    legacy_entries = _entry_names(legacy_root)
    shared_entries = sorted(set(canonical_entries) & set(legacy_entries))
    return {
        "scope": scope,
        "canonical_root": str(canonical_root),
        "legacy_root": str(legacy_root),
        "canonical_exists": canonical_root.exists(),
        "legacy_exists": legacy_root.exists(),
        "canonical_entries": canonical_entries,
        "legacy_entries": legacy_entries,
        "shared_entries": shared_entries,
        "canonical_only_entries": sorted(set(canonical_entries) - set(legacy_entries)),
        "legacy_only_entries": sorted(set(legacy_entries) - set(canonical_entries)),
        "status": (
            "duplicate"
            if canonical_root.exists() and legacy_root.exists()
            else "clear"
        ),
    }


def summarize_local_stage_layout(*, local_results_root: Path, local_cache_mirror_root: Path) -> dict:
    archive_corpus_root = local_results_root / "corpus_01"
    cache_corpus_root = local_cache_mirror_root / "results" / "corpus_01"
    archive_stage_ids = _entry_names(archive_corpus_root)
    cache_stage_ids = _entry_names(cache_corpus_root)
    return {
        "archive_corpus_root": str(archive_corpus_root),
        "cache_corpus_root": str(cache_corpus_root),
        "archive_stage_ids": archive_stage_ids,
        "cache_stage_ids": cache_stage_ids,
        "shared_stage_ids": sorted(set(archive_stage_ids) & set(cache_stage_ids)),
        "archive_only_stage_ids": sorted(set(archive_stage_ids) - set(cache_stage_ids)),
        "cache_only_stage_ids": sorted(set(cache_stage_ids) - set(archive_stage_ids)),
        "authoritative_stage_root": str(cache_corpus_root),
    }


def workspace_layout_report() -> dict:
    local_results_root = support.benchmark_local_results_root()
    local_cache_mirror_root = support.benchmark_local_cache_mirror_root()
    remote_workspace_root = local_cache_mirror_root.parent
    issues: list[dict[str, str]] = []

    root_pairs = [
        summarize_root_pair(
            scope="remote-results",
            canonical_root=local_cache_mirror_root / "results",
            legacy_root=local_cache_mirror_root / "bijux-dna-results",
        ),
        summarize_root_pair(
            scope="remote-reference",
            canonical_root=local_cache_mirror_root / "reference",
            legacy_root=local_cache_mirror_root / "bijux-reference",
        ),
    ]
    for pair in root_pairs:
        if pair["status"] == "duplicate":
            issues.append(
                {
                    "issue_id": f"duplicate-{pair['scope']}-root",
                    "detail": (
                        f"both {pair['canonical_root']} and {pair['legacy_root']} exist"
                    ),
                }
            )

    unexpected_remote_siblings: list[str] = []
    for sibling_name in ["results", "corpus_01", "extra-data"]:
        sibling_root = remote_workspace_root / sibling_name
        if sibling_root.exists():
            unexpected_remote_siblings.append(str(sibling_root))
            issues.append(
                {
                    "issue_id": "unexpected-remote-workspace-sibling",
                    "detail": f"unexpected sibling beside .cache: {sibling_root}",
                }
            )

    local_stage_layout = summarize_local_stage_layout(
        local_results_root=local_results_root,
        local_cache_mirror_root=local_cache_mirror_root,
    )

    return {
        "local_results_root": str(local_results_root),
        "local_cache_mirror_root": str(local_cache_mirror_root),
        "remote_workspace_root": str(remote_workspace_root),
        "authoritative_roots": {
            "remote_results_root": str(local_cache_mirror_root / "results"),
            "remote_reference_root": str(local_cache_mirror_root / "reference"),
            "local_stage_root": local_stage_layout["authoritative_stage_root"],
        },
        "root_pairs": root_pairs,
        "unexpected_remote_siblings": unexpected_remote_siblings,
        "local_stage_layout": local_stage_layout,
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
        f"- Authoritative remote results root: `{report['authoritative_roots']['remote_results_root']}`",
        f"- Authoritative remote reference root: `{report['authoritative_roots']['remote_reference_root']}`",
        f"- Authoritative local publication root: `{report['authoritative_roots']['local_stage_root']}`",
        f"- Status: `{report['status']}`",
        f"- Issues: `{report['issue_count']}`",
        "",
        "## Root Pairs",
        "",
    ]
    for pair in report["root_pairs"]:
        lines.append(
            f"- `{pair['scope']}`: `{pair['status']}` "
            f"(canonical `{pair['canonical_root']}`, legacy `{pair['legacy_root']}`)"
        )
        if pair["canonical_entries"]:
            lines.append(
                "  - canonical entries: "
                + ", ".join(f"`{entry}`" for entry in pair["canonical_entries"])
            )
        if pair["legacy_entries"]:
            lines.append(
                "  - legacy entries: "
                + ", ".join(f"`{entry}`" for entry in pair["legacy_entries"])
            )
        if pair["shared_entries"]:
            lines.append(
                "  - shared entries: "
                + ", ".join(f"`{entry}`" for entry in pair["shared_entries"])
            )
    lines.extend(
        [
            "",
            "## Local Stage Layout",
            "",
            f"- Archive corpus root: `{report['local_stage_layout']['archive_corpus_root']}`",
            f"- Cache corpus root: `{report['local_stage_layout']['cache_corpus_root']}`",
        ]
    )
    if report["local_stage_layout"]["shared_stage_ids"]:
        lines.append(
            "- Shared stage ids: "
            + ", ".join(
                f"`{entry}`" for entry in report["local_stage_layout"]["shared_stage_ids"]
            )
        )
    if report["local_stage_layout"]["archive_only_stage_ids"]:
        lines.append(
            "- Archive-only stage ids: "
            + ", ".join(
                f"`{entry}`"
                for entry in report["local_stage_layout"]["archive_only_stage_ids"]
            )
        )
    if report["local_stage_layout"]["cache_only_stage_ids"]:
        lines.append(
            "- Cache-only stage ids: "
            + ", ".join(
                f"`{entry}`"
                for entry in report["local_stage_layout"]["cache_only_stage_ids"]
            )
        )
    lines.append("")
    lines.append("## Issues")
    lines.append("")
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
