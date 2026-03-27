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
    FILTER_READS_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    filter_reads_benchmark_defaults,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
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
    defaults = filter_reads_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.filter_reads benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "lunarc-apptainer"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--threads", type=int, default=defaults["threads"])
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--sample-jobs",
        type=int,
        default=1,
        help="Number of corpus samples to benchmark concurrently.",
    )
    parser.add_argument(
        "--sample-limit",
        type=int,
        default=0,
        help="Optional positive limit for local or frontend preflight runs.",
    )
    parser.add_argument("--max-n", type=int, default=defaults["max_n"])
    parser.add_argument(
        "--max-n-fraction",
        type=float,
        default=defaults["max_n_fraction"],
    )
    parser.add_argument(
        "--max-n-count",
        type=int,
        default=defaults["max_n_count"],
    )
    parser.add_argument(
        "--low-complexity-threshold",
        type=float,
        default=defaults["low_complexity_threshold"],
    )
    parser.add_argument(
        "--entropy-threshold",
        type=float,
        default=defaults["entropy_threshold"],
    )
    parser.add_argument("--kmer-ref", default=defaults["kmer_ref"])
    parser.add_argument("--polyx-policy", default=defaults["polyx_policy"])
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
    max_n: int | None,
    max_n_fraction: float | None,
    max_n_count: int | None,
    low_complexity_threshold: float | None,
    entropy_threshold: float | None,
    kmer_ref: str | None,
    polyx_policy: str | None,
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
        "filter",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if max_n is not None:
        command.extend(["--max-n", str(max_n)])
    if max_n_fraction is not None:
        command.extend(["--max-n-fraction", str(max_n_fraction)])
    if max_n_count is not None:
        command.extend(["--max-n-count", str(max_n_count)])
    if low_complexity_threshold is not None:
        command.extend(
            ["--low-complexity-threshold", str(low_complexity_threshold)]
        )
    if entropy_threshold is not None:
        command.extend(["--entropy-threshold", str(entropy_threshold)])
    if kmer_ref:
        command.extend(["--kmer-ref", kmer_ref])
    if polyx_policy:
        command.extend(["--polyx-policy", polyx_policy])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "filter" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "filter" / sample_id


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
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False)
    status = "completed" if completed.returncode == 0 else "failed"
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]) if sample["r2"] is not None else None,
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
        else default_results_stage_root(
            corpus_root, FILTER_READS_BENCHMARK_CONTRACT.stage_id
        )
    )
    validate_benchmark_layout(corpus_root, out_root)
    out_root.mkdir(parents=True, exist_ok=True)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else FILTER_READS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        FILTER_READS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=FILTER_READS_BENCHMARK_CONTRACT.scenario_id,
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
            threads=args.threads,
            jobs=args.jobs,
            max_n=args.max_n,
            max_n_fraction=args.max_n_fraction,
            max_n_count=args.max_n_count,
            low_complexity_threshold=args.low_complexity_threshold,
            entropy_threshold=args.entropy_threshold,
            kmer_ref=args.kmer_ref,
            polyx_policy=args.polyx_policy,
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

    payload = {
        "schema_version": "bijux.fastq.filter_reads.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": FILTER_READS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": FILTER_READS_BENCHMARK_CONTRACT.scenario_id,
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
        "sample_limit": None if args.sample_limit <= 0 else args.sample_limit,
        "dry_run": args.dry_run,
        "samples_total": len(samples),
        "samples_failed": failures,
        "max_n": args.max_n,
        "max_n_fraction": args.max_n_fraction,
        "max_n_count": args.max_n_count,
        "low_complexity_threshold": args.low_complexity_threshold,
        "entropy_threshold": args.entropy_threshold,
        "kmer_ref": args.kmer_ref,
        "polyx_policy": args.polyx_policy,
        "runs": [asdict(run) for run in runs if run is not None],
    }
    (out_root / "run_manifest.json").write_text(
        json.dumps(payload, indent=2) + "\n", encoding="utf-8"
    )
    return 0 if failures == 0 else 1


if __name__ == "__main__":
    raise SystemExit(main())
