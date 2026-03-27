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
    DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT,
    artifact_bundle_exists,
    artifact_bundle_size_bytes,
    benchmark_runtime_env,
    default_results_stage_root,
    deplete_reference_contaminants_benchmark_defaults,
    discover_normalized_samples,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
    sha256_artifact_bundle,
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
    defaults = deplete_reference_contaminants_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.deplete_reference_contaminants benchmarks for corpus-01."
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
    parser.add_argument(
        "--reference-index",
        default=os.environ.get("BIJUX_CONTAMINANT_REFERENCE_INDEX", ""),
        help="Contaminant-reference Bowtie2 index prefix or directory.",
    )
    parser.add_argument(
        "--reference-catalog-id",
        default=defaults["reference_catalog_id"],
    )
    parser.add_argument(
        "--reference-index-backend",
        default=defaults["reference_index_backend"],
    )
    parser.add_argument("--decoy-mode", default=defaults["decoy_mode"])
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
    reference_index: Path,
    decoy_mode: str,
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
        "deplete-reference-contaminants",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--reference-index",
        str(reference_index),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--decoy-mode",
        decoy_mode,
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_reference_contaminants" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_reference_contaminants" / sample_id


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


def resolve_reference_index(args: argparse.Namespace) -> Path:
    if not args.reference_index.strip():
        raise SystemExit(
            "fastq.deplete_reference_contaminants corpus benchmarking requires "
            "--reference-index (or BIJUX_CONTAMINANT_REFERENCE_INDEX) so the governed "
            "contaminant index is explicit."
        )
    reference_index = Path(args.reference_index).expanduser().resolve()
    if not artifact_bundle_exists(reference_index):
        raise SystemExit(
            f"missing contaminant reference index bundle or prefix matches: {reference_index}"
        )
    return reference_index


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
            corpus_root, DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id
        )
    )
    reference_index = resolve_reference_index(args)
    reference_index_digest = sha256_artifact_bundle(reference_index)

    validate_benchmark_layout(corpus_root, out_root)
    out_root.mkdir(parents=True, exist_ok=True)
    runtime_env = benchmark_runtime_env(out_root)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]
    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.scenario_id,
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
            reference_index=reference_index,
            decoy_mode=args.decoy_mode,
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
        "schema_version": "bijux.fastq.deplete_reference_contaminants.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "stage_id": DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": DEPLETE_REFERENCE_CONTAMINANTS_BENCHMARK_CONTRACT.scenario_id,
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
        "reference_index": str(reference_index),
        "reference_index_digest": reference_index_digest,
        "reference_index_size_bytes": artifact_bundle_size_bytes(reference_index),
        "reference_catalog_id": args.reference_catalog_id,
        "reference_index_backend": args.reference_index_backend,
        "decoy_mode": args.decoy_mode,
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
