#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    MERGE_PAIRS_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    merge_pairs_benchmark_defaults,
    normalize_tool_csv,
    require_canonical_tool_roster,
    select_paired_samples,
    validate_corpus_contract,
)


@dataclass
class SampleRun:
    sample_id: str
    r1: str
    r2: str
    layout: str
    status: str
    exit_code: int
    command: list[str]
    report_json: str


def parse_args() -> argparse.Namespace:
    defaults = merge_pairs_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.merge_pairs benchmarks for the paired corpus-01 cohort."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument(
        "--out-root",
        default="",
        help="Benchmark output root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.merge_pairs/lunarc.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "lunarc-apptainer"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="",
        help="Comma-separated merge tool ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--sample-jobs",
        type=int,
        default=1,
        help="Number of corpus samples to benchmark concurrently.",
    )
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--merge-overlap", type=int, default=defaults["merge_overlap"])
    parser.add_argument("--min-length", type=int, default=defaults["min_length"])
    parser.add_argument(
        "--unmerged-read-policy",
        default=defaults["unmerged_read_policy"],
        help="emit_unmerged_pairs or omit_unmerged_pairs.",
    )
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Skip samples that already have report.json in the output tree.",
    )
    parser.add_argument(
        "--dry-run",
        action=argparse.BooleanOptionalAction,
        default=False,
        help="Write the run manifest without executing sample commands.",
    )
    return parser.parse_args()


def build_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    merge_overlap: int | None,
    min_length: int | None,
    unmerged_read_policy: str,
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
        "merge",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--r2",
        str(sample["r2"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--threads",
        str(threads),
        "--unmerged-read-policy",
        unmerged_read_policy,
    ]
    if merge_overlap is not None:
        command.extend(["--merge-overlap", str(merge_overlap)])
    if min_length is not None:
        command.extend(["--min-length", str(min_length)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "merge_pairs" / sample_id / "report.json"


def run_sample_command(
    *,
    repo_root: Path,
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False)
    status = "completed" if completed.returncode == 0 else "failed"
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]),
        layout=sample["layout"],
        status=status,
        exit_code=completed.returncode,
        command=command,
        report_json=str(sample_report),
    )


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
        else default_results_stage_root(corpus_root, MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id)
    )
    out_root.mkdir(parents=True, exist_ok=True)

    all_samples = discover_normalized_samples(corpus_root)
    metadata_by_sample = validate_corpus_contract(corpus_root, spec, all_samples)
    samples = select_paired_samples(spec, all_samples, metadata_by_sample)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else MERGE_PAIRS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id,
    )

    runs: list[SampleRun | None] = [None] * len(samples)
    failures = 0
    pending: list[tuple[int, dict, Path, list[str]]] = []
    for sample_index, sample in enumerate(samples):
        sample_report = report_path(out_root, sample["sample_id"])
        if args.resume and sample_report.is_file():
            runs[sample_index] = (
                SampleRun(
                    sample_id=sample["sample_id"],
                    r1=str(sample["r1"]),
                    r2=str(sample["r2"]),
                    layout=sample["layout"],
                    status="skipped_existing_report",
                    exit_code=0,
                    command=[],
                    report_json=str(sample_report),
                )
            )
            continue

        command = build_command(
            out_root=out_root,
            platform=args.platform,
            tools=",".join(tools),
            threads=args.threads,
            jobs=args.jobs,
            merge_overlap=args.merge_overlap,
            min_length=args.min_length,
            unmerged_read_policy=args.unmerged_read_policy,
            sample=sample,
        )
        if args.dry_run:
            runs[sample_index] = (
                SampleRun(
                    sample_id=sample["sample_id"],
                    r1=str(sample["r1"]),
                    r2=str(sample["r2"]),
                    layout=sample["layout"],
                    status="dry_run",
                    exit_code=0,
                    command=command,
                    report_json=str(sample_report),
                )
            )
            continue
        pending.append((sample_index, sample, sample_report, command))

    if pending:
        max_workers = max(1, args.sample_jobs)
        with ThreadPoolExecutor(max_workers=max_workers) as executor:
            futures = {
                executor.submit(
                    run_sample_command,
                    repo_root=repo_root,
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

    completed_runs = [run for run in runs if run is not None]
    if len(completed_runs) != len(samples):
        raise SystemExit(
            f"merge corpus runner drift: expected {len(samples)} sample runs, found {len(completed_runs)}"
        )

    payload = {
        "schema_version": "bijux.fastq.merge_pairs.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": MERGE_PAIRS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": MERGE_PAIRS_BENCHMARK_CONTRACT.scenario_id,
        "tool_kind": "benchmark",
        "platform": args.platform,
        "tools": tools,
        "threads": args.threads,
        "jobs": args.jobs,
        "sample_jobs": args.sample_jobs,
        "sample_limit": args.sample_limit or None,
        "dry_run": args.dry_run,
        "merge_overlap": args.merge_overlap,
        "min_length": args.min_length,
        "unmerged_read_policy": args.unmerged_read_policy,
        "repo_root": str(repo_root),
        "corpus_root": str(corpus_root),
        "out_root": str(out_root),
        "samples_total": len(completed_runs),
        "samples_failed": failures,
        "runs": [asdict(run) for run in completed_runs],
    }
    (out_root / "run_manifest.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
