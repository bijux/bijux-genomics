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
    discover_normalized_samples,
    load_corpus_spec,
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
    parser = argparse.ArgumentParser(
        description="Run fastq.trim_polyg_tails benchmarks for corpus-01."
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
        help="Benchmark output root. Defaults to <corpus-root>/benchmarks/fastq.trim_polyg_tails/lunarc.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "lunarc-apptainer"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="fastp,bbduk",
        help="Comma-separated trim-polyg tool ids.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument(
        "--polyx-preset",
        default="illumina_twocolor",
        help="PolyX preset name to pin chemistry-aware trim behavior.",
    )
    parser.add_argument("--trim-polyg", type=str, default="")
    parser.add_argument("--min-polyg-run", type=int, default=10)
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Skip samples that already have report.json in the output tree.",
    )
    return parser.parse_args()


def parse_optional_bool(raw: str) -> bool | None:
    if not raw:
        return None
    normalized = raw.strip().lower()
    if normalized in {"1", "true", "yes"}:
        return True
    if normalized in {"0", "false", "no"}:
        return False
    raise SystemExit(f"unsupported boolean value for --trim-polyg: {raw}")


def build_command(
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    polyx_preset: str,
    trim_polyg: bool | None,
    min_polyg_run: int,
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
        "trim-polyg-tails",
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
        "--polyx-preset",
        polyx_preset,
        "--min-polyg-run",
        str(min_polyg_run),
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if trim_polyg is not None:
        command.extend(["--trim-polyg", "true" if trim_polyg else "false"])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "trim_polyg_tails" / sample_id / "report.json"


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
        else (corpus_root / "benchmarks" / "fastq.trim_polyg_tails" / "lunarc")
    )
    out_root.mkdir(parents=True, exist_ok=True)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    trim_polyg = parse_optional_bool(args.trim_polyg)
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
            tools=args.tools,
            threads=args.threads,
            jobs=args.jobs,
            polyx_preset=args.polyx_preset,
            trim_polyg=trim_polyg,
            min_polyg_run=args.min_polyg_run,
            sample=sample,
        )
        completed = subprocess.run(command, cwd=repo_root, check=False)
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
                exit_code=completed.returncode,
                command=command,
                report_json=str(sample_report),
            )
        )

    payload = {
        "schema_version": "bijux.fastq.trim_polyg_tails.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "platform": args.platform,
        "tools": [tool for tool in args.tools.split(",") if tool],
        "threads": args.threads,
        "jobs": args.jobs,
        "polyx_preset": args.polyx_preset,
        "trim_polyg": trim_polyg,
        "min_polyg_run": args.min_polyg_run,
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
