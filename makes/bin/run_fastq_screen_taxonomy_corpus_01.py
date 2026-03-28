#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import shutil
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    SCREEN_TAXONOMY_BENCHMARK_CONTRACT,
    artifact_bundle_exists,
    artifact_bundle_size_bytes,
    benchmark_runtime_env,
    default_screen_taxonomy_database_root,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
    resolve_artifact_lineage_json,
    screen_taxonomy_benchmark_defaults,
    sha256_artifact_bundle,
    sha256_file,
    validate_benchmark_layout,
    validate_corpus_contract,
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
        else default_results_stage_root(
            corpus_root, SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id
        )
    )
    validate_benchmark_layout(corpus_root, out_root)
    out_root.mkdir(parents=True, exist_ok=True)
    database_root = resolve_database_root(args, out_root)
    database_digest = sha256_artifact_bundle(database_root)
    database_lineage_json = resolve_artifact_lineage_json(database_root)
    runtime_env = benchmark_runtime_env(out_root)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else SCREEN_TAXONOMY_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id,
    )

    runs: list[SampleRun | None] = [None] * len(samples)
    failures = 0
    pending: list[tuple[int, dict, Path, list[str]]] = []

    for sample_index, sample in enumerate(samples):
        current_sample_root = sample_root(out_root, sample["sample_id"])
        sample_report = report_path(out_root, sample["sample_id"])
        if args.resume and current_sample_root.is_dir() and not sample_report.is_file():
            reset_sample_payload(out_root, sample["sample_id"])
        if args.resume and sample_report.is_file():
            if sample_report_is_resume_ready(sample_report):
                runs[sample_index] = SampleRun(
                    sample_id=sample["sample_id"],
                    r1=str(sample["r1"]),
                    r2=str(sample["r2"]) if sample["r2"] is not None else None,
                    layout=sample["layout"],
                    status="skipped_existing_report",
                    exit_code=0,
                    command=[],
                    report_json=str(sample_report),
                )
                continue
            reset_sample_payload(out_root, sample["sample_id"])
        command = build_command(
            out_root=out_root,
            platform=args.platform,
            tools=",".join(tools),
            database_root=database_root,
            threads=args.threads,
            jobs=args.jobs,
            sample=sample,
        )
        if args.dry_run:
            runs[sample_index] = SampleRun(
                sample_id=sample["sample_id"],
                r1=str(sample["r1"]),
                r2=str(sample["r2"]) if sample["r2"] is not None else None,
                layout=sample["layout"],
                status="dry_run",
                exit_code=0,
                command=command,
                report_json=str(sample_report),
            )
            continue
        pending.append((sample_index, sample, sample_report, command))

    if pending:
        with ThreadPoolExecutor(max_workers=max(1, args.sample_jobs)) as executor:
            futures = {
                executor.submit(
                    run_sample_command,
                    repo_root=repo_root,
                    runtime_env=runtime_env,
                    sample=sample,
                    command=command,
                    sample_report=sample_report,
                ): sample_index
                for sample_index, sample, sample_report, command in pending
            }
            for future in as_completed(futures):
                sample_index = futures[future]
                run = future.result()
                runs[sample_index] = run
                if run.exit_code != 0:
                    failures += 1

    manifest = {
        "schema_version": "bijux.fastq.screen_taxonomy.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": SCREEN_TAXONOMY_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": SCREEN_TAXONOMY_BENCHMARK_CONTRACT.scenario_id,
        "tool_kind": "benchmark",
        "platform": args.platform,
        "repo_root": str(repo_root),
        "corpus_id": "corpus-01",
        "corpus_root": str(corpus_root),
        "out_root": str(out_root),
        "tools": tools,
        "threads": args.threads,
        "jobs": args.jobs,
        "sample_jobs": args.sample_jobs,
        "sample_limit": args.sample_limit or None,
        "dry_run": args.dry_run,
        "database_root": str(database_root),
        "database_digest": database_digest,
        "database_size_bytes": artifact_bundle_size_bytes(database_root),
        "database_lineage_json": (
            str(database_lineage_json) if database_lineage_json is not None else None
        ),
        "database_lineage_digest": (
            sha256_file(database_lineage_json)
            if database_lineage_json is not None
            else None
        ),
        "database_catalog_id": args.database_catalog_id,
        "database_artifact_id": args.database_artifact_id,
        "database_namespace": args.database_namespace,
        "database_scope": args.database_scope,
        "samples_total": len(runs),
        "samples_failed": failures,
        "runs": [asdict(run) for run in runs if run is not None],
    }
    (out_root / "run_manifest.json").write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
