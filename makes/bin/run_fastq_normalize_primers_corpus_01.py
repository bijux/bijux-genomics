#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import os
import subprocess
from concurrent.futures import ThreadPoolExecutor, as_completed
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path

from corpus_01_fastq_benchmark_support import (
    NORMALIZE_PRIMERS_BENCHMARK_CONTRACT,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    normalize_primers_benchmark_defaults,
    normalize_tool_csv,
    require_canonical_tool_roster,
    validate_benchmark_layout,
    validate_corpus_contract,
)


def parse_bool_literal(value: str) -> bool:
    lowered = value.strip().lower()
    if lowered in {"true", "1", "yes", "y"}:
        return True
    if lowered in {"false", "0", "no", "n"}:
        return False
    raise argparse.ArgumentTypeError(f"invalid boolean literal: {value}")


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
    defaults = normalize_primers_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.normalize_primers benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument("--out-root", default="")
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
    )
    parser.add_argument("--tools", default="")
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
    parser.add_argument("--primer-set-id", default=defaults["primer_set_id"])
    parser.add_argument(
        "--orientation-policy",
        default=defaults["orientation_policy"],
    )
    parser.add_argument(
        "--max-mismatch-rate",
        type=float,
        default=defaults["max_mismatch_rate"],
    )
    parser.add_argument(
        "--min-overlap-bp",
        type=int,
        default=defaults["min_overlap_bp"],
    )
    parser.add_argument(
        "--strict-5p-anchor",
        type=parse_bool_literal,
        default=defaults["strict_5p_anchor"],
    )
    parser.add_argument(
        "--allow-iupac-codes",
        type=parse_bool_literal,
        default=defaults["allow_iupac_codes"],
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
    jobs: int,
    primer_set_id: str,
    orientation_policy: str,
    max_mismatch_rate: float,
    min_overlap_bp: int,
    strict_5p_anchor: bool,
    allow_iupac_codes: bool,
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
        "normalize-primers",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--primer-set-id",
        primer_set_id,
        "--orientation-policy",
        orientation_policy,
        "--max-mismatch-rate",
        str(max_mismatch_rate),
        "--min-overlap-bp",
        str(min_overlap_bp),
        "--strict-5p-anchor",
        "true" if strict_5p_anchor else "false",
        "--allow-iupac-codes",
        "true" if allow_iupac_codes else "false",
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "normalize_primers" / sample_id / "report.json"


def run_sample_command(
    *,
    repo_root: Path,
    sample: dict,
    command: list[str],
    sample_report: Path,
) -> SampleRun:
    completed = subprocess.run(command, cwd=repo_root, check=False)
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
            corpus_root, NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id
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
        else NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.scenario_id,
    )

    runs: list[SampleRun | None] = [None] * len(samples)
    failures = 0
    pending: list[tuple[int, dict, Path, list[str]]] = []
    for sample_index, sample in enumerate(samples):
        sample_report = report_path(out_root, sample["sample_id"])
        if args.resume and sample_report.is_file():
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
        command = build_command(
            out_root=out_root,
            platform=args.platform,
            tools=",".join(tools),
            jobs=args.jobs,
            primer_set_id=args.primer_set_id,
            orientation_policy=args.orientation_policy,
            max_mismatch_rate=args.max_mismatch_rate,
            min_overlap_bp=args.min_overlap_bp,
            strict_5p_anchor=args.strict_5p_anchor,
            allow_iupac_codes=args.allow_iupac_codes,
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

    manifest = {
        "schema_version": "bijux.fastq.normalize_primers.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": NORMALIZE_PRIMERS_BENCHMARK_CONTRACT.scenario_id,
        "tool_kind": "benchmark",
        "platform": args.platform,
        "repo_root": str(repo_root),
        "corpus_id": "corpus-01",
        "corpus_root": str(corpus_root),
        "out_root": str(out_root),
        "tools": tools,
        "jobs": args.jobs,
        "sample_jobs": args.sample_jobs,
        "sample_limit": args.sample_limit or None,
        "dry_run": args.dry_run,
        "primer_set_id": args.primer_set_id,
        "orientation_policy": args.orientation_policy,
        "max_mismatch_rate": args.max_mismatch_rate,
        "min_overlap_bp": args.min_overlap_bp,
        "strict_5p_anchor": args.strict_5p_anchor,
        "allow_iupac_codes": args.allow_iupac_codes,
        "failures": failures,
        "runs": [asdict(run) for run in runs if run is not None],
    }
    (out_root / "run_manifest.json").write_text(
        json.dumps(manifest, indent=2) + "\n",
        encoding="utf-8",
    )
    return 1 if failures else 0


if __name__ == "__main__":
    raise SystemExit(main())
