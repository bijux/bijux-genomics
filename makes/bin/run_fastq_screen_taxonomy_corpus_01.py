#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from dataclasses import dataclass
from pathlib import Path

from benchmark_fastq_corpus.config import add_workspace_config_argument
from benchmark_fastq_corpus.runner_compat import (
    append_stage_arg,
    run_corpus_stage_compat,
)
from corpus_01_fastq_benchmark_support import (
    artifact_bundle_exists,
    default_results_stage_root,
    default_screen_taxonomy_database_root,
    load_corpus_spec,
    screen_taxonomy_benchmark_defaults,
)


@dataclass
class SampleRun:
    sample_id: str
    r1: str
    r2: str | None
    layout: str
    status: str
    exit_code: int
    command: list[str]
    report_json: str


def parse_args() -> argparse.Namespace:
    defaults = screen_taxonomy_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.screen_taxonomy benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--threads", type=int, default=defaults["threads"])
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument(
        "--database-root",
        default=os.environ.get("BIJUX_TAXONOMY_DB", ""),
        help="Concrete taxonomy database directory or artifact prefix used for the governed run.",
    )
    parser.add_argument(
        "--database-catalog-id",
        default=defaults["database_catalog_id"],
    )
    parser.add_argument(
        "--database-artifact-id",
        default=defaults["database_artifact_id"],
    )
    parser.add_argument(
        "--database-namespace",
        default=defaults["database_namespace"],
    )
    parser.add_argument(
        "--database-scope",
        default=defaults["database_scope"],
    )
    add_workspace_config_argument(parser)
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
    )
    parser.add_argument(
        "--dry-run",
        action=argparse.BooleanOptionalAction,
        default=False,
    )
    return parser.parse_args()


def build_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    database_root: Path,
    threads: int,
    jobs: int,
    sample: dict,
) -> list[str]:
    command = [
        "cargo",
        "run",
        "-q",
        "-p",
        "bijux-dna",
        "--",
        "--platform",
        platform,
        "bench",
        "fastq",
        "screen-taxonomy",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--database-root",
        str(database_root),
        "--out",
        str(out_root),
        "--tools",
        tools,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "screen_taxonomy" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "screen_taxonomy" / sample_id


def sample_report_is_resume_ready(sample_report: Path) -> bool:
    try:
        payload = json.loads(sample_report.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError):
        return False
    if payload.get("failures"):
        return False
    gate = payload.get("gate")
    if isinstance(gate, dict) and gate.get("passes") is False:
        return False
    records = payload.get("records")
    return isinstance(records, list) and bool(records)


def reset_sample_payload(out_root: Path, sample_id: str) -> None:
    current_sample_root = sample_root(out_root, sample_id)
    if current_sample_root.is_dir():
        shutil.rmtree(current_sample_root)


def run_sample_command(
    *,
    repo_root: Path,
    runtime_env: dict[str, str],
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False, env=runtime_env)
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]) if sample["r2"] is not None else None,
        layout=sample["layout"],
        status="completed" if completed.returncode == 0 else "failed",
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


def resolve_database_root(args: argparse.Namespace, out_root: Path) -> Path:
    requested_path = args.database_root.strip()
    if requested_path:
        database_root = Path(requested_path).expanduser().resolve()
    else:
        database_root = default_screen_taxonomy_database_root(
            out_root,
            database_namespace=args.database_namespace,
            database_scope=args.database_scope,
            database_artifact_id=args.database_artifact_id,
        ).resolve()
    if not artifact_bundle_exists(database_root):
        raise SystemExit(
            "missing taxonomy database bundle or prefix matches: "
            f"{database_root}. Populate the governed default under "
            f"{default_screen_taxonomy_database_root(out_root, database_namespace=args.database_namespace, database_scope=args.database_scope, database_artifact_id=args.database_artifact_id)} "
            "or override it with --database-root / BIJUX_TAXONOMY_DB."
        )
    return database_root


def main() -> int:
    args = parse_args()
    repo_root = Path(args.repo_root).resolve()
    spec = load_corpus_spec(repo_root)
    corpus_root = (
        Path(args.corpus_root).expanduser().resolve()
        if args.corpus_root
        else Path(spec["preferred_root"]).expanduser().resolve()
    )
    out_root = (
        Path(args.out_root).expanduser().resolve()
        if args.out_root
        else default_results_stage_root(corpus_root, "fastq.screen_taxonomy")
    )
    args.corpus_root = str(corpus_root)
    args.out_root = str(out_root)

    database_root = resolve_database_root(args, out_root)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--database-root", database_root)
    manifest_args: list[str] = []
    append_stage_arg(manifest_args, "--database-catalog-id", args.database_catalog_id)
    append_stage_arg(manifest_args, "--database-artifact-id", args.database_artifact_id)
    append_stage_arg(manifest_args, "--database-namespace", args.database_namespace)
    append_stage_arg(manifest_args, "--database-scope", args.database_scope)
    return run_corpus_stage_compat(
        stage_id="fastq.screen_taxonomy",
        args=args,
        stage_args=stage_args,
        manifest_args=manifest_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
