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
    REPORT_QC_BENCHMARK_CONTRACT,
    REPORT_QC_CONTRIBUTOR_CONTRACTS,
    benchmark_tool_root,
    default_results_stage_root,
    discover_normalized_samples,
    load_corpus_spec,
    normalize_tool_csv,
    require_canonical_tool_roster,
    validate_corpus_contract,
)

GOVERNED_QC_INPUTS_SCHEMA_VERSION = "bijux.fastq.report_qc.inputs.v1"

UPSTREAM_STAGE_COMMANDS = {
    "fastq.validate_reads": {
        "subcommand": "validate-reads",
        "tools": "fastqvalidator",
        "extra_args": [],
    },
    "fastq.detect_adapters": {
        "subcommand": "detect-adapters",
        "tools": "fastqc",
        "extra_args": ["--threads", "1"],
    },
    "fastq.profile_reads": {
        "subcommand": "profile-reads",
        "tools": "seqkit_stats",
        "extra_args": ["--threads", "1"],
    },
    "fastq.profile_read_lengths": {
        "subcommand": "profile-read-lengths",
        "tools": "seqkit_stats",
        "extra_args": ["--threads", "1", "--histogram-bins", "100"],
    },
}


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
    governed_qc_manifest: str
    governed_qc_input_count: int
    upstream_commands: list[list[str]]


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Run fastq.report_qc benchmarks for corpus-01."
    )
    parser.add_argument("--repo-root", default=".")
    parser.add_argument("--corpus-root", default="")
    parser.add_argument(
        "--out-root",
        default="",
        help="Benchmark output root. Defaults to <corpus-root-parent>/results/<corpus-dir>/fastq.report_qc/lunarc.",
    )
    parser.add_argument(
        "--platform",
        default=os.environ.get("BIJUX_PLATFORM", "apptainer-amd64"),
        help="bijux-dna platform name.",
    )
    parser.add_argument(
        "--tools",
        default="",
        help="Comma-separated QC aggregation tool ids. Defaults to the governed benchmark cohort.",
    )
    parser.add_argument("--jobs", type=int, default=1)
    parser.add_argument("--sample-limit", type=int, default=0)
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


def report_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "report_qc" / sample_id / "report.json"


def manifest_path(out_root: Path, sample_id: str) -> Path:
    return out_root / "bench" / "report_qc" / sample_id / "governed_qc_inputs_manifest.json"


def upstream_stage_root(corpus_root: Path, stage_id: str) -> Path:
    return default_results_stage_root(corpus_root, stage_id)


def contributor_artifact_path(
    corpus_root: Path,
    sample_id: str,
    stage_id: str,
    tool_id: str,
    relative_path: str,
) -> Path:
    return benchmark_tool_root(
        upstream_stage_root(corpus_root, stage_id),
        stage_id,
        sample_id,
        tool_id,
    ) / relative_path


def required_contributor_artifacts(corpus_root: Path, sample_id: str) -> list[dict]:
    artifacts: list[dict] = []
    for contract in REPORT_QC_CONTRIBUTOR_CONTRACTS:
        artifact_path = contributor_artifact_path(
            corpus_root,
            sample_id,
            contract.stage_id,
            contract.tool_id,
            contract.relative_path,
        )
        artifacts.append(
            {
                "name": contract.artifact_name,
                "path": artifact_path,
                "role": contract.artifact_role,
                "optional": False,
                "stage_id": contract.stage_id,
                "tool_id": contract.tool_id,
                "artifact_id": contract.artifact_id,
            }
        )
    return artifacts


def ensure_upstream_stage_outputs(
    *,
    repo_root: Path,
    corpus_root: Path,
    platform: str,
    sample: dict,
    stage_id: str,
    dry_run: bool,
) -> list[str] | None:
    stage_spec = UPSTREAM_STAGE_COMMANDS[stage_id]
    out_root = upstream_stage_root(corpus_root, stage_id)
    out_root.mkdir(parents=True, exist_ok=True)
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
        stage_spec["subcommand"],
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        stage_spec["tools"],
    ]
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    command.extend(stage_spec["extra_args"])
    if dry_run:
        return command
    completed = subprocess.run(command, cwd=repo_root, check=False)
    if completed.returncode != 0:
        raise SystemExit(
            f"{stage_id} governed QC bootstrap failed for {sample['sample_id']} with exit code {completed.returncode}"
        )
    return command


def ensure_governed_qc_inputs(
    *,
    repo_root: Path,
    corpus_root: Path,
    out_root: Path,
    platform: str,
    sample: dict,
    dry_run: bool,
) -> tuple[Path, int, list[list[str]]]:
    upstream_commands: list[list[str]] = []
    artifacts = required_contributor_artifacts(corpus_root, sample["sample_id"])

    missing_stage_ids = sorted(
        {
            artifact["stage_id"]
            for artifact in artifacts
            if not artifact["path"].exists()
        }
    )
    for stage_id in missing_stage_ids:
        command = ensure_upstream_stage_outputs(
            repo_root=repo_root,
            corpus_root=corpus_root,
            platform=platform,
            sample=sample,
            stage_id=stage_id,
            dry_run=dry_run,
        )
        if command is not None:
            upstream_commands.append(command)

    if not dry_run:
        unresolved = [
            artifact["path"]
            for artifact in artifacts
            if not artifact["path"].exists()
        ]
        if unresolved:
            raise SystemExit(
                "report-qc governed input resolution failed for "
                f"{sample['sample_id']}: missing {', '.join(str(path) for path in unresolved)}"
            )

    payload = {
        "schema_version": GOVERNED_QC_INPUTS_SCHEMA_VERSION,
        "qc_inputs": [
            {
                "name": artifact["name"],
                "path": str(artifact["path"]),
                "role": artifact["role"],
                "optional": False,
            }
            for artifact in artifacts
        ],
        "contributors": [
            {
                "contributor_id": f"{artifact['stage_id']}.{artifact['tool_id']}",
                "stage_id": artifact["stage_id"],
                "tool_id": artifact["tool_id"],
                "artifact_id": artifact["artifact_id"],
                "artifact_role": artifact["role"],
                "path": str(artifact["path"]),
            }
            for artifact in artifacts
        ],
        "raw_fastqc_dir": str(
            contributor_artifact_path(
                corpus_root,
                sample["sample_id"],
                "fastq.detect_adapters",
                "fastqc",
                "fastqc",
            )
        ),
    }
    path = manifest_path(out_root, sample["sample_id"])
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    return path, len(artifacts), upstream_commands


def build_report_qc_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    jobs: int,
    sample: dict,
    governed_qc_manifest: Path,
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
        "report-qc",
        "--sample-id",
        sample["sample_id"],
        "--r1",
        str(sample["r1"]),
        "--out",
        str(out_root),
        "--tools",
        tools,
        "--aggregation-engine",
        "multiqc",
        "--aggregation-scope",
        "governed_qc_artifacts",
        "--governed-qc-manifest",
        str(governed_qc_manifest),
    ]
    if jobs > 1:
        command.extend(["--jobs", str(jobs)])
    if sample["r2"] is not None:
        command.extend(["--r2", str(sample["r2"])])
    return command


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
        else default_results_stage_root(corpus_root, REPORT_QC_BENCHMARK_CONTRACT.stage_id)
    )
    out_root.mkdir(parents=True, exist_ok=True)

    samples = discover_normalized_samples(corpus_root)
    validate_corpus_contract(corpus_root, spec, samples)
    if args.sample_limit > 0:
        samples = samples[: args.sample_limit]

    requested_tools = (
        normalize_tool_csv(args.tools)
        if args.tools
        else REPORT_QC_BENCHMARK_CONTRACT.tools
    )
    tools = require_canonical_tool_roster(
        repo_root,
        REPORT_QC_BENCHMARK_CONTRACT.stage_id,
        requested_tools,
        scenario_id=REPORT_QC_BENCHMARK_CONTRACT.scenario_id,
    )

    runs: list[SampleRun] = []
    failures = 0
    for sample in samples:
        sample_report = report_path(out_root, sample["sample_id"])
        governed_manifest, governed_qc_input_count, upstream_commands = ensure_governed_qc_inputs(
            repo_root=repo_root,
            corpus_root=corpus_root,
            out_root=out_root,
            platform=args.platform,
            sample=sample,
            dry_run=args.dry_run,
        )
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
                    governed_qc_manifest=str(governed_manifest),
                    governed_qc_input_count=governed_qc_input_count,
                    upstream_commands=upstream_commands,
                )
            )
            continue
        command = build_report_qc_command(
            out_root=out_root,
            platform=args.platform,
            tools=",".join(tools),
            jobs=args.jobs,
            sample=sample,
            governed_qc_manifest=governed_manifest,
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
                governed_qc_manifest=str(governed_manifest),
                governed_qc_input_count=governed_qc_input_count,
                upstream_commands=upstream_commands,
            )
        )

    payload = {
        "schema_version": "bijux.fastq.report_qc.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": spec["corpus_id"],
        "stage_id": REPORT_QC_BENCHMARK_CONTRACT.stage_id,
        "scenario_id": REPORT_QC_BENCHMARK_CONTRACT.scenario_id,
        "tool_kind": "benchmark",
        "platform": args.platform,
        "tools": tools,
        "jobs": args.jobs,
        "sample_limit": args.sample_limit or None,
        "dry_run": args.dry_run,
        "aggregation_engine": "multiqc",
        "aggregation_scope": "governed_qc_artifacts",
        "report_only": True,
        "mutates_fastq": False,
        "may_change_read_count": False,
        "governed_contributor_stage_ids": sorted(
            {contract.stage_id for contract in REPORT_QC_CONTRIBUTOR_CONTRACTS}
        ),
        "governed_contributor_tool_ids": sorted(
            {contract.tool_id for contract in REPORT_QC_CONTRIBUTOR_CONTRACTS}
        ),
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
