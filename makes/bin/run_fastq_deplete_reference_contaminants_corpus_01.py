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
    deplete_reference_contaminants_benchmark_defaults,
    resolve_bowtie2_index_prefix,
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
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
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
    requested_index = Path(args.reference_index).expanduser().resolve()
    try:
        reference_index = resolve_bowtie2_index_prefix(requested_index)
    except (FileNotFoundError, ValueError) as error:
        raise SystemExit(str(error)) from error
    if not artifact_bundle_exists(reference_index):
        raise SystemExit(
            f"missing contaminant reference index bundle or prefix matches: {reference_index}"
        )
    return reference_index


def main() -> int:
    args = parse_args()
    reference_index = resolve_reference_index(args)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--reference-index", reference_index)
    append_stage_arg(stage_args, "--decoy-mode", args.decoy_mode)
    manifest_args: list[str] = []
    append_stage_arg(
        manifest_args, "--reference-catalog-id", args.reference_catalog_id
    )
    append_stage_arg(
        manifest_args, "--reference-index-backend", args.reference_index_backend
    )
    return run_corpus_stage_compat(
        stage_id="fastq.deplete_reference_contaminants",
        args=args,
        stage_args=stage_args,
        manifest_args=manifest_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
