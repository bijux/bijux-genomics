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
    default_host_reference_index_root,
    default_results_stage_root,
    deplete_host_benchmark_defaults,
    load_corpus_spec,
    resolve_bowtie2_index_prefix,
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
    defaults = deplete_host_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.deplete_host benchmarks for corpus-01."
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
        default=os.environ.get("BIJUX_HOST_REFERENCE_INDEX", ""),
        help="Bowtie2 host-reference index prefix or directory.",
    )
    parser.add_argument(
        "--reference-catalog-id",
        default=defaults["reference_catalog_id"],
    )
    parser.add_argument(
        "--reference-index-backend",
        default=defaults["reference_index_backend"],
    )
    parser.add_argument(
        "--host-identity-threshold",
        type=float,
        default=defaults["host_identity_threshold"],
    )
    parser.add_argument(
        "--retain-unmapped-only",
        type=parse_bool_literal,
        default=defaults["retain_unmapped_only"],
    )
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
    host_identity_threshold: float,
    retain_unmapped_only: bool,
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
        "deplete-host",
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
        "--host-identity-threshold",
        str(host_identity_threshold),
        "--retain-unmapped-only",
        "true" if retain_unmapped_only else "false",
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_host" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_host" / sample_id


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


def resolve_reference_index(args: argparse.Namespace, out_root: Path) -> Path:
    requested_path = args.reference_index.strip()
    if requested_path:
        requested_index = Path(requested_path).expanduser().resolve()
    else:
        requested_index = default_host_reference_index_root(
            out_root,
            reference_catalog_id=args.reference_catalog_id,
            reference_index_backend=args.reference_index_backend,
        ).resolve()
    try:
        reference_index = resolve_bowtie2_index_prefix(requested_index)
    except (FileNotFoundError, ValueError) as error:
        raise SystemExit(str(error)) from error
    if not artifact_bundle_exists(reference_index):
        raise SystemExit(
            "missing host reference index bundle or prefix matches: "
            f"{reference_index}. Populate the governed default under "
            f"{default_host_reference_index_root(out_root, reference_catalog_id=args.reference_catalog_id, reference_index_backend=args.reference_index_backend)} "
            "or override it with --reference-index / BIJUX_HOST_REFERENCE_INDEX."
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
        else default_results_stage_root(corpus_root, "fastq.deplete_host")
    )
    args.corpus_root = str(corpus_root)
    args.out_root = str(out_root)

    reference_index = resolve_reference_index(args, out_root)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--reference-index", reference_index)
    append_stage_arg(stage_args, "--host-identity-threshold", args.host_identity_threshold)
    append_stage_arg(stage_args, "--retain-unmapped-only", args.retain_unmapped_only)
    manifest_args: list[str] = []
    append_stage_arg(
        manifest_args, "--reference-catalog-id", args.reference_catalog_id
    )
    append_stage_arg(
        manifest_args, "--reference-index-backend", args.reference_index_backend
    )
    return run_corpus_stage_compat(
        stage_id="fastq.deplete_host",
        args=args,
        stage_args=stage_args,
        manifest_args=manifest_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
