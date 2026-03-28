from __future__ import annotations

import argparse
import os
import subprocess
from pathlib import Path


def append_stage_arg(stage_args: list[str], flag: str, value: object) -> None:
    if value is None:
        return
    if isinstance(value, str):
        if not value:
            return
        normalized = value
    elif isinstance(value, bool):
        normalized = "true" if value else "false"
    else:
        normalized = str(value)
    stage_args.extend([flag, normalized])


def run_corpus_stage_compat(
    *,
    stage_id: str,
    args: argparse.Namespace,
    stage_args: list[str] | None = None,
    extra_env: dict[str, str] | None = None,
) -> int:
    repo_root = Path(args.repo_root).resolve()
    runtime_env = os.environ.copy()
    if extra_env:
        runtime_env.update(extra_env)
    command = ["cargo", "run", "-q", "-p", "bijux-dna", "--"]
    platform = getattr(args, "platform", "")
    if platform:
        command.extend(["--platform", platform])
    command.extend(["bench", "corpus-fastq", "--stage", stage_id])

    config = getattr(args, "config", "")
    if config:
        command.extend(["--config", config])
    corpus_root = getattr(args, "corpus_root", "")
    if corpus_root:
        command.extend(["--corpus-root", corpus_root])
    out_root = getattr(args, "out_root", "")
    if out_root:
        command.extend(["--out-root", out_root])
    tools = getattr(args, "tools", "")
    if tools:
        command.extend(["--tools", tools])
    command.extend(["--threads", str(getattr(args, "threads", 1))])
    command.extend(["--jobs", str(getattr(args, "jobs", 1))])

    sample_jobs = getattr(args, "sample_jobs", None)
    if sample_jobs is not None:
        command.extend(["--sample-jobs", str(sample_jobs)])
    sample_limit = getattr(args, "sample_limit", 0)
    if sample_limit:
        command.extend(["--sample-limit", str(sample_limit)])
    if getattr(args, "resume", True) is False:
        command.extend(["--resume", "false"])
    if getattr(args, "dry_run", False):
        command.append("--dry-run")
    for stage_arg in stage_args or []:
        command.extend(["--stage-arg", stage_arg])

    completed = subprocess.run(
        command,
        cwd=repo_root,
        check=False,
        env=runtime_env,
    )
    return completed.returncode
