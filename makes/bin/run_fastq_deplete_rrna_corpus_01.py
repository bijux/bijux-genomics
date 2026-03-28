#!/usr/bin/env python3
from __future__ import annotations

import argparse
import hashlib
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
    deplete_rrna_benchmark_defaults,
)


def sha256_path(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as handle:
        for chunk in iter(lambda: handle.read(1024 * 1024), b""):
            digest.update(chunk)
    return f"sha256:{digest.hexdigest()}"


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
    defaults = deplete_rrna_benchmark_defaults()
    parser = argparse.ArgumentParser(
        description="Run fastq.deplete_rrna benchmarks for corpus-01."
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
        "--rrna-db",
        default=os.environ.get("BIJUX_RRNA_DB", ""),
        help="Concrete SortMeRNA reference FASTA path for the governed rRNA bundle.",
    )
    parser.add_argument(
        "--rrna-bundle-id",
        default=defaults["rrna_bundle_id"],
        help="Stable governed bundle identifier recorded in the run manifest.",
    )
    parser.add_argument(
        "--min-identity",
        type=float,
        default=defaults["min_identity"],
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
    rrna_db: Path,
    min_identity: float,
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
        "deplete-rrna",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--rrna-db",
        str(rrna_db),
        "--min-identity",
        str(min_identity),
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if threads > 1:
        command.extend(["--threads", str(threads)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_rrna" / sample_id / "report.json"


def sample_root(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "deplete_rrna" / sample_id


def tool_root(out_root: Path, sample_id: str, tool: str) -> Path:
    return sample_root(out_root, sample_id) / "tools" / tool


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


def sortmerna_sample_workdir(out_root: Path, sample_id: str) -> Path:
    return tool_root(out_root, sample_id, "sortmerna") / "sortmerna_workdir"


def sortmerna_shared_index_dir(out_root: Path, rrna_bundle_id: str) -> Path:
    return (
        out_root
        / "_reference_cache"
        / "fastq.deplete_rrna"
        / rrna_bundle_id
        / "sortmerna_workdir"
        / "idx"
    )


def sortmerna_shared_index_seeded(shared_idx_dir: Path) -> bool:
    return shared_idx_dir.is_dir() and any(shared_idx_dir.iterdir())


def _clone_index_cache(source_idx_dir: Path, dest_idx_dir: Path) -> None:
    dest_idx_dir.mkdir(parents=True, exist_ok=True)
    for source_path in source_idx_dir.iterdir():
        dest_path = dest_idx_dir / source_path.name
        if dest_path.exists():
            if dest_path.is_dir():
                shutil.rmtree(dest_path)
            else:
                dest_path.unlink()
        if source_path.is_dir():
            shutil.copytree(source_path, dest_path, copy_function=os.link)
        else:
            try:
                os.link(source_path, dest_path)
            except OSError:
                shutil.copy2(source_path, dest_path)


def lunarc_input_bind_root() -> Path:
    return Path.home() / "bijux"


def lunarc_container_input_path(host_path: Path) -> str:
    bind_root = lunarc_input_bind_root().resolve()
    resolved = host_path.resolve()
    return "/data/input/" + str(resolved.relative_to(bind_root))


def prepare_sortmerna_sample_workdir(
    out_root: Path,
    sample_id: str,
    rrna_bundle_id: str,
) -> Path:
    shared_idx_dir = sortmerna_shared_index_dir(out_root, rrna_bundle_id)
    shared_idx_dir.mkdir(parents=True, exist_ok=True)
    sample_workdir = sortmerna_sample_workdir(out_root, sample_id)
    sample_workdir.mkdir(parents=True, exist_ok=True)
    sample_idx_dir = sample_workdir / "idx"
    if sample_idx_dir.is_symlink() or sample_idx_dir.is_file():
        sample_idx_dir.unlink()
    elif sample_idx_dir.is_dir():
        shutil.rmtree(sample_idx_dir)
    if sortmerna_shared_index_seeded(shared_idx_dir):
        _clone_index_cache(shared_idx_dir, sample_idx_dir)
    else:
        sample_idx_dir.mkdir(parents=True, exist_ok=True)
    return shared_idx_dir


def promote_sortmerna_sample_index_cache(
    out_root: Path,
    sample_id: str,
    rrna_bundle_id: str,
) -> Path:
    shared_idx_dir = sortmerna_shared_index_dir(out_root, rrna_bundle_id)
    sample_idx_dir = sortmerna_sample_workdir(out_root, sample_id) / "idx"
    if not sample_idx_dir.is_dir():
        raise FileNotFoundError(f"missing SortMeRNA sample idx dir: {sample_idx_dir}")
    if not sortmerna_shared_index_seeded(shared_idx_dir):
        _clone_index_cache(sample_idx_dir, shared_idx_dir)
    return shared_idx_dir


def prune_sortmerna_sample_payload(out_root: Path, sample_id: str) -> None:
    sample_workdir = sortmerna_sample_workdir(out_root, sample_id)
    if sample_workdir.is_dir():
        shutil.rmtree(sample_workdir)


def warm_sortmerna_shared_index_cache(
    *,
    platform: str,
    rrna_db: Path,
    seed_r1: Path,
    shared_idx_dir: Path,
    threads: int,
) -> None:
    if platform not in {"apptainer-amd64", "lunarc-apptainer"}:
        return
    shared_idx_dir.mkdir(parents=True, exist_ok=True)
    if sortmerna_shared_index_seeded(shared_idx_dir):
        return
    cache_workdir = shared_idx_dir.parent
    cache_workdir.mkdir(parents=True, exist_ok=True)
    sif_path = (
        lunarc_input_bind_root()
        / "bijux-dna-container"
        / "apptainer"
        / "sif"
        / "sortmerna.sif"
    )
    if not sif_path.is_file():
        raise SystemExit(f"missing SortMeRNA Apptainer image: {sif_path}")
    warm_threads = max(1, min(4, threads or 1))
    bind_root = lunarc_input_bind_root().resolve()
    if not rrna_db.resolve().is_relative_to(bind_root):
        raise SystemExit(f"rrna_db must live under {bind_root} for Lunarc warmup: {rrna_db}")
    if not seed_r1.resolve().is_relative_to(bind_root):
        raise SystemExit(f"seed FASTQ must live under {bind_root} for Lunarc warmup: {seed_r1}")
    command = [
        "apptainer",
        "exec",
        "--cleanenv",
        "--no-home",
        "--containall",
        "--bind",
        f"{bind_root}:/data/input:ro",
        "--bind",
        f"{cache_workdir}:/data/output",
        "--pwd",
        "/data/output",
        str(sif_path),
        "/usr/local/bin/sortmerna-bin",
        "--ref",
        lunarc_container_input_path(rrna_db),
        "--reads",
        lunarc_container_input_path(seed_r1),
        "--workdir",
        "/data/output/",
        "--index",
        "1",
        "--threads",
        str(warm_threads),
    ]
    completed = subprocess.run(command, check=False)
    if completed.returncode != 0:
        raise SystemExit(
            "SortMeRNA shared-index warmup failed with exit code "
            f"{completed.returncode} for {shared_idx_dir}"
        )
    if not sortmerna_shared_index_seeded(shared_idx_dir):
        raise SystemExit(f"SortMeRNA warmup did not materialize idx files at {shared_idx_dir}")


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


def resolve_rrna_db(args: argparse.Namespace) -> Path:
    if not args.rrna_db.strip():
        raise SystemExit(
            "fastq.deplete_rrna corpus benchmarking requires --rrna-db "
            "(or BIJUX_RRNA_DB) so the governed rRNA bundle path is explicit."
        )
    rrna_db = Path(args.rrna_db).expanduser().resolve()
    if not rrna_db.is_file():
        raise SystemExit(f"missing rRNA bundle FASTA: {rrna_db}")
    return rrna_db


def main() -> int:
    args = parse_args()
    rrna_db = resolve_rrna_db(args)
    stage_args: list[str] = []
    append_stage_arg(stage_args, "--rrna-db", rrna_db)
    append_stage_arg(stage_args, "--rrna-bundle-id", args.rrna_bundle_id)
    append_stage_arg(stage_args, "--min-identity", args.min_identity)
    return run_corpus_stage_compat(
        stage_id="fastq.deplete_rrna",
        args=args,
        stage_args=stage_args,
    )


if __name__ == "__main__":
    raise SystemExit(main())
