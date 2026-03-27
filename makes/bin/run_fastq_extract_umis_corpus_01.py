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
    EXTRACT_UMIS_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    extract_umis_benchmark_defaults,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
    select_paired_samples,
    validate_benchmark_layout,
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
    defaults = extract_umis_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.extract_umis benchmarks for the paired corpus-01 cohort."
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
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--umi-pattern", default=defaults["umi_pattern"])
    parser.add_argument(
        "--allow-missing-umi-headers",
        action=argparse.BooleanOptionalAction,
        default=defaults["allow_missing_umi_headers"],
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
    threads: int,
    jobs: int,
    umi_pattern: str,
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
        "umi",
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
        "--umi-pattern",
        umi_pattern,
    ]
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "extract_umis" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "extract_umis" / sample_id


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
    allow_missing_umi_headers: bool,
) -> SampleRun:
    env = os.environ.copy()
    if allow_missing_umi_headers:
        env["BIJUX_ALLOW_NO_UMI"] = "1"
    else:
        env.pop("BIJUX_ALLOW_NO_UMI", None)
    completed = subprocess.run(command, cwd=repo_root, check=False, env=env)
    return SampleRun(
        sample_id=sample["sample_id"],
        r1=str(sample["r1"]),
        r2=str(sample["r2"]),
        layout=sample["layout"],
        status="completed" if completed.returncode == 0 else "failed",
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
            corpus_root, EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id
        )
    )

    validate_benchmark_layout(corpus_root, out_root)
    out_root.mkdir(parents=True, exist_ok=True)

    all_samples = discover_normalized_samples(corpus_root)
    metadata_by_sample = validate_corpus_contract(corpus_root, spec, all_samples)
    samples = select_paired_samples(spec, all_samples, metadata_by_sample)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else EXTRACT_UMIS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=EXTRACT_UMIS_BENCHMARK_CONTRACT.scenario_id,
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
                    r2=str(sample["r2"]),
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
            umi_pattern=args.umi_pattern,
            sample=sample,
        )
        if args.dry_run:
            runs[sample_index] = SampleRun(
                sample_id=sample["sample_id"],
                r1=str(sample["r1"]),
                r2=str(sample["r2"]),
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
                    allow_missing_umi_headers=args.allow_missing_umi_headers,
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
        "schema_version": "bijux.fastq.extract_umis.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": EXTRACT_UMIS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": EXTRACT_UMIS_BENCHMARK_CONTRACT.scenario_id,
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
        "sample_scope": "paired",
        "umi_pattern": args.umi_pattern,
        "allow_missing_umi_headers": args.allow_missing_umi_headers,
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
