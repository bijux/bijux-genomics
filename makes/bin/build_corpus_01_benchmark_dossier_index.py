#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    CORPUS_01_PUBLICATION_CONTRACTS,
    benchmark_remote_corpus_root,
    benchmark_remote_results_legacy_root,
    default_local_results_stage_root,
    default_results_stage_root,
    legacy_local_results_stage_root,
    resolve_existing_dossier_path,
)


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Build a machine-readable corpus-01 FASTQ dossier index."
    )
    parser.add_argument(
        "--docs-root",
        default="docs/benchmark",
        help="Benchmark docs root.",
    )
    parser.add_argument(
        "--json-out",
        default="docs/benchmark/corpus-01-dossier-index.json",
        help="Machine-readable dossier index output path.",
    )
    parser.add_argument(
        "--markdown-out",
        default="docs/benchmark/corpus-01-dossier-index.md",
        help="Markdown dossier index output path.",
    )
    return parser.parse_args()


def path_is_under(path: Path, root: Path) -> bool:
    try:
        path.relative_to(root)
    except ValueError:
        return False
    return True


def classify_run_root_source(run_root: Path, stage_id: str) -> str:
    remote_corpus_root = benchmark_remote_corpus_root()
    if run_root == default_local_results_stage_root(remote_corpus_root, stage_id):
        return "local-cache-mirror"
    if run_root == legacy_local_results_stage_root(remote_corpus_root, stage_id):
        return "local-results-root"
    if run_root == default_results_stage_root(remote_corpus_root, stage_id):
        return "remote-results-root"
    remote_legacy_root = benchmark_remote_results_legacy_root() / remote_corpus_root.name / stage_id / "lunarc"
    if run_root == remote_legacy_root:
        return "remote-results-legacy-root"
    if path_is_under(run_root, benchmark_remote_corpus_root().parent):
        return "remote-custom"
    return "custom"


def build_stage_entry(docs_root: Path, stage_id: str, sample_scope: str) -> dict:
    stage_docs_root = docs_root / stage_id / "corpus-01"
    summary_path = stage_docs_root / "summary.json"
    dossier_path = resolve_existing_dossier_path(stage_docs_root)
    remote_corpus_root = benchmark_remote_corpus_root()
    expected_remote_run_root = default_results_stage_root(remote_corpus_root, stage_id)
    expected_local_cache_mirror_run_root = default_local_results_stage_root(
        remote_corpus_root,
        stage_id,
    )
    expected_local_results_run_root = legacy_local_results_stage_root(
        remote_corpus_root,
        stage_id,
    )
    expected_remote_legacy_run_root = (
        benchmark_remote_results_legacy_root() / remote_corpus_root.name / stage_id / "lunarc"
    )
    entry = {
        "stage_id": stage_id,
        "sample_scope": sample_scope,
        "status": "missing",
        "summary_path": str(summary_path),
        "dossier_path": str(dossier_path),
        "expected_remote_run_root": str(expected_remote_run_root),
        "expected_remote_legacy_run_root": str(expected_remote_legacy_run_root),
        "expected_local_cache_mirror_run_root": str(expected_local_cache_mirror_run_root),
        "expected_local_results_run_root": str(expected_local_results_run_root),
    }
    if not summary_path.is_file():
        return entry

    summary = json.loads(summary_path.read_text(encoding="utf-8"))
    run_root = Path(str(summary.get("run_root", ""))).expanduser() if summary.get("run_root") else None
    entry.update(
        {
            "status": "published",
            "generated_at_utc": summary.get("generated_at_utc"),
            "platform": summary.get("platform"),
            "corpus_root": summary.get("corpus_root"),
            "run_root": str(run_root) if run_root is not None else None,
            "run_root_source": classify_run_root_source(run_root, stage_id)
            if run_root is not None
            else "missing",
        }
    )
    return entry


def build_index(docs_root: Path) -> dict:
    stages = [
        build_stage_entry(docs_root, contract.stage_id, contract.sample_scope)
        for contract in CORPUS_01_PUBLICATION_CONTRACTS
    ]
    return {
        "corpus_id": "corpus-01",
        "stage_count": len(stages),
        "published_stage_count": sum(1 for stage in stages if stage["status"] == "published"),
        "missing_stage_count": sum(1 for stage in stages if stage["status"] != "published"),
        "stages": stages,
    }


def render_markdown(index: dict) -> str:
    lines = [
        "# `corpus-01` FASTQ dossier index",
        "",
        f"- Governed publication stages: `{index['stage_count']}`",
        f"- Published summaries: `{index['published_stage_count']}`",
        f"- Missing summaries: `{index['missing_stage_count']}`",
        "",
        "## Stage index",
        "",
    ]
    for stage in index["stages"]:
        if stage["status"] == "published":
            lines.append(
                f"- `{stage['stage_id']}`: `{stage['generated_at_utc']}` from `{stage['run_root_source']}`"
            )
            lines.append(f"  - published run root: `{stage['run_root']}`")
            lines.append(f"  - expected remote run root: `{stage['expected_remote_run_root']}`")
            lines.append(
                f"  - expected local cache mirror run root: `{stage['expected_local_cache_mirror_run_root']}`"
            )
        else:
            lines.append(f"- `{stage['stage_id']}`: `missing`")
            lines.append(f"  - expected remote run root: `{stage['expected_remote_run_root']}`")
    return "\n".join(lines) + "\n"


def main() -> int:
    args = parse_args()
    docs_root = Path(args.docs_root).resolve()
    index = build_index(docs_root)

    json_out = Path(args.json_out).resolve()
    json_out.parent.mkdir(parents=True, exist_ok=True)
    json_out.write_text(json.dumps(index, indent=2) + "\n", encoding="utf-8")

    markdown_out = Path(args.markdown_out).resolve()
    markdown_out.parent.mkdir(parents=True, exist_ok=True)
    markdown_out.write_text(render_markdown(index), encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
