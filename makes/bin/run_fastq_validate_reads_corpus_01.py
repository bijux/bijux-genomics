#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
import sys
import tomllib
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path


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
        description="Run fastq.validate_reads benchmarks for corpus-01."
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
        help="Benchmark output root. Defaults to <corpus-root>/benchmarks/fastq.validate_reads/lunarc.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "lunarc-apptainer"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="fastqvalidator,fastqc,fastq_scan,seqtk,fqtools",
        help="Comma-separated validate tool ids.",
    )
    parser.add_argument("--threads", type=int, default=1)
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--validation-mode", default="")
    parser.add_argument("--pair-sync-policy", default="")
    parser.add_argument(
        "--resume",
        action=argparse.BooleanOptionalAction,
        default=True,
        help="Skip samples that already have report.json in the output tree.",
    )
    return parser.parse_args()


def load_corpus_spec(repo_root: Path) -> dict:
    path = repo_root / "configs" / "runtime" / "corpora" / "corpus-01.toml"
    with path.open("rb") as handle:
        return tomllib.load(handle)


def discover_samples(corpus_root: Path) -> list[dict]:
    normalized = corpus_root / "normalized"
    if not normalized.is_dir():
        raise SystemExit(f"missing normalized corpus directory: {normalized}")

    sample_ids: set[str] = set()
    for path in normalized.glob("sample_*_R1.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R1.fastq.gz"))
    for path in normalized.glob("sample_*_R2.fastq.gz"):
        sample_ids.add(path.name.removesuffix("_R2.fastq.gz"))

    samples: list[dict] = []
    for sample_id in sorted(sample_ids):
        r1 = normalized / f"{sample_id}_R1.fastq.gz"
        r2 = normalized / f"{sample_id}_R2.fastq.gz"
        if not r1.is_file():
            raise SystemExit(f"missing R1 for sample {sample_id}: {r1}")
        sample = {
            "sample_id": sample_id,
            "r1": r1,
            "r2": r2 if r2.is_file() else None,
            "layout": "pe" if r2.is_file() else "se",
        }
        samples.append(sample)
    if len(samples) != 20:
        raise SystemExit(
            f"expected 20 normalized samples for corpus-01, found {len(samples)}"
        )
    return samples


def build_command(
    repo_root: Path,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    validation_mode: str,
    pair_sync_policy: str,
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
        "validate-reads",
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
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    if validation_mode:
        command.extend(["--validation-mode", validation_mode])
    if pair_sync_policy:
        command.extend(["--pair-sync-policy", pair_sync_policy])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "validate_reads" / sample_id / "report.json"


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
        else (corpus_root / "benchmarks" / "fastq.validate_reads" / "lunarc")
    )
    out_root.mkdir(parents=True, exist_ok=True)

    samples = discover_samples(corpus_root)
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
            repo_root=repo_root,
            out_root=out_root,
            platform=args.platform,
            tools=args.tools,
            threads=args.threads,
            jobs=args.jobs,
            validation_mode=args.validation_mode,
            pair_sync_policy=args.pair_sync_policy,
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
        "schema_version": "bijux.fastq.validate_reads.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "platform": args.platform,
        "tools": [tool for tool in args.tools.split(",") if tool],
        "threads": args.threads,
        "jobs": args.jobs,
        "validation_mode": args.validation_mode or None,
        "pair_sync_policy": args.pair_sync_policy or None,
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
