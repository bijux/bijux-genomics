#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    TRIM_READS_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
    trim_reads_benchmark_defaults,
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
    defaults = trim_reads_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.trim_reads benchmarks for corpus-01."
    )
    parser.add_argument(
        "--repo-root",
        default=".",
        help="Repository root that contains Cargo.toml and configs/runtime/corpora/corpus-01.toml.",
    )
    parser.add_argument(
        "--corpus-root",
        default="",
        help="Materialized corpus root. Defaults to the preferred_root from the corpus spec.",
    )
    parser.add_argument(
        "--out-root",
        default="",
        help="Benchmark output root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.trim_reads/lunarc.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "lunarc-apptainer"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="",
        help="Comma-separated trim tool ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--sample-limit",
        type=int,
        default=0,
        help="Optional positive limit for local or frontend preflight runs.",
    )
    parser.add_argument("--min-length", type=int, default=defaults["min_length"])
    parser.add_argument("--quality-cutoff", type=int, default=defaults["quality_cutoff"])
    parser.add_argument("--n-policy", default=defaults["n_policy"])
    parser.add_argument("--adapter-policy", default=defaults["adapter_policy"])
    parser.add_argument("--polyx-policy", default=defaults["polyx_policy"])
    parser.add_argument(
        "--contaminant-policy", default=defaults["contaminant_policy"]
    )
    parser.add_argument(
        "--adapter-bank-preset",
        default=defaults["adapter_bank_preset"] or "",
    )
    parser.add_argument(
        "--polyx-preset",
        default=defaults["polyx_preset"] or "",
    )
    parser.add_argument(
        "--contaminant-preset",
        default=defaults["contaminant_preset"] or "",
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


def optional_str(raw: str) -> str | None:
    value = raw.strip()
    return value or None


def build_command(
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    min_length: int | None,
    quality_cutoff: int | None,
    n_policy: str,
    adapter_policy: str,
    polyx_policy: str,
    contaminant_policy: str,
    adapter_bank_preset: str | None,
    polyx_preset: str | None,
    contaminant_preset: str | None,
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
        "trim-reads",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--threads",
        str(threads),
        "--n-policy",
        n_policy,
        "--adapter-policy",
        adapter_policy,
        "--polyx-policy",
        polyx_policy,
        "--contaminant-policy",
        contaminant_policy,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if min_length is not None:
        command.extend(["--min-length", str(min_length)])
    if quality_cutoff is not None:
        command.extend(["--quality-cutoff", str(quality_cutoff)])
    if adapter_bank_preset is not None:
        command.extend(["--adapter-bank-preset", adapter_bank_preset])
    if polyx_preset is not None:
        command.extend(["--polyx-preset", polyx_preset])
    if contaminant_preset is not None:
        command.extend(["--contaminant-preset", contaminant_preset])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "trim_reads" / sample_id / "report.json"


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
        else default_results_stage_root(corpus_root, TRIM_READS_BENCHMARK_CONTRACT.stage_id)
    )
    out_root.mkdir(parents=True, exist_ok=True)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools) if args.tools else TRIM_READS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        TRIM_READS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=TRIM_READS_BENCHMARK_CONTRACT.scenario_id,
    )
    adapter_bank_preset = optional_str(args.adapter_bank_preset)
    polyx_preset = optional_str(args.polyx_preset)
    contaminant_preset = optional_str(args.contaminant_preset)
    runs: list[SampleRun] = []
    failures = 0

    for sample in samples:
        sample_report = report_path(out_root, sample["sample_id"])
        if args.resume and sample_report.is_file():
            runs.append(
                SampleRun(
                    sample_id=sample["sample_id"],
                    r1=str(sample["r1"]),
                    r2=str(sample["r2"]) if sample["r2"] is not None else None,
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
            min_length=args.min_length,
            quality_cutoff=args.quality_cutoff,
            n_policy=args.n_policy,
            adapter_policy=args.adapter_policy,
            polyx_policy=args.polyx_policy,
            contaminant_policy=args.contaminant_policy,
            adapter_bank_preset=adapter_bank_preset,
            polyx_preset=polyx_preset,
            contaminant_preset=contaminant_preset,
            sample=sample,
        )
        if args.dry_run:
            completed_return_code = 0
            status = "dry_run"
        else:
            completed = subprocess.run(command, cwd=repo_root, check=False)
            completed_return_code = completed.returncode
            status = "completed" if completed.returncode == 0 else "failed"
            if completed.returncode != 0:
                failures += 1
        runs.append(
            SampleRun(
                sample_id=sample["sample_id"],
                r1=str(sample["r1"]),
                r2=str(sample["r2"]) if sample["r2"] is not None else None,
                layout=sample["layout"],
                status=status,
                exit_code=completed_return_code,
                command=command,
                report_json=str(sample_report),
            )
        )

    payload = {
        "schema_version": "bijux.fastq.trim_reads.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": TRIM_READS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": TRIM_READS_BENCHMARK_CONTRACT.scenario_id,
        "tool_kind": "benchmark",
        "platform": args.platform,
        "tools": tools,
        "threads": args.threads,
        "jobs": args.jobs,
        "sample_limit": args.sample_limit or None,
        "dry_run": args.dry_run,
        "min_length": args.min_length,
        "quality_cutoff": args.quality_cutoff,
        "n_policy": args.n_policy,
        "adapter_policy": args.adapter_policy,
        "polyx_policy": args.polyx_policy,
        "contaminant_policy": args.contaminant_policy,
        "adapter_bank_preset": adapter_bank_preset,
        "polyx_preset": polyx_preset,
        "contaminant_preset": contaminant_preset,
        "repo_root": str(repo_root),
        "corpus_root": str(corpus_root),
        "out_root": str(out_root),
        "samples_total": len(runs),
        "samples_failed": failures,
        "runs": [asdict(run) for run in runs],
    }
    (out_root / "run_manifest.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return 1 if failures else 0


if __name__ == "__main__":
    sys.exit(main())
