#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from datetime import datetime, timezone
from pathlib import Path

from benchmark_fastq_corpus.support import (
    DETECT_ADAPTERS_BENCHMARK_CONTRACT,
    PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT,
    PROFILE_READS_BENCHMARK_CONTRACT,
    benchmark_local_results_root,
    benchmark_remote_repo_root,
    benchmark_sample_root,
    default_results_stage_root,
    load_json,
    localize_results_path,
    stage_run_dir_name,
)

EXPECTED_STAGE_TOOLS = {
    "fastq.validate_reads": ["fastqvalidator", "fastqc", "fastq_scan", "seqtk", "fqtools"],
    "fastq.detect_adapters": DETECT_ADAPTERS_BENCHMARK_CONTRACT.tools,
    "fastq.profile_reads": PROFILE_READS_BENCHMARK_CONTRACT.tools,
    "fastq.profile_read_lengths": PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT.tools,
}


def build_fastq_stage_command(
    *,
    bench_subcommand: str,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    sample: dict,
    extra_args: list[str] | None = None,
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
        bench_subcommand,
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
    if extra_args:
        command.extend(extra_args)
    return command


def build_validate_reads_command(
    *,
    out_root: Path,
    platform: str,
    tools: str,
    threads: int,
    jobs: int,
    validation_mode: str,
    pair_sync_policy: str,
    sample: dict,
) -> list[str]:
    extra_args: list[str] = []
    if validation_mode:
        extra_args.extend(["--validation-mode", validation_mode])
    if pair_sync_policy:
        extra_args.extend(["--pair-sync-policy", pair_sync_policy])
    return build_fastq_stage_command(
        bench_subcommand="validate-reads",
        out_root=out_root,
        platform=platform,
        tools=tools,
        threads=threads,
        jobs=jobs,
        sample=sample,
        extra_args=extra_args,
    )


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser(
        description="Repair corpus-01 FASTQ local result manifests from mirrored sample artifacts."
    )
    parser.add_argument(
        "--results-root",
        default=str(benchmark_local_results_root()),
        help="Canonical local result mirror root.",
    )
    parser.add_argument(
        "--corpus-id",
        default="corpus_01",
        help="Corpus identifier within the local result mirror.",
    )
    parser.add_argument(
        "--stages",
        default="fastq.validate_reads,fastq.detect_adapters,fastq.profile_reads,fastq.profile_read_lengths,fastq.merge_pairs",
        help="Comma-separated stage ids to repair.",
    )
    return parser.parse_args()


def normalize_stage_csv(raw: str) -> list[str]:
    stages: list[str] = []
    seen: set[str] = set()
    for stage_id in (entry.strip() for entry in raw.split(",")):
        if not stage_id or stage_id in seen:
            continue
        seen.add(stage_id)
        stages.append(stage_id)
    if not stages:
        raise SystemExit("stage roster must not be empty")
    return stages


def sample_report_paths(run_root: Path, stage_id: str) -> list[Path]:
    stage_dir = run_root / "bench" / stage_run_dir_name(stage_id)
    return sorted(stage_dir.glob("sample_*/report.json"))


def required_path(parameters: dict, *keys: str) -> str:
    for key in keys:
        value = parameters.get(key)
        if value:
            return str(value)
    raise SystemExit(f"missing required parameter in report context: one of {keys!r}")


def sample_from_report(sample_id: str, sample_report: Path) -> tuple[dict, dict]:
    report = load_json(sample_report)
    records = report.get("records") or []
    if not records:
        raise SystemExit(f"missing records in mirrored sample report: {sample_report}")
    record = records[0]
    parameters = record.get("context", {}).get("parameters", {})
    r1 = required_path(parameters, "input_r1", "r1")
    r2 = parameters.get("input_r2") or parameters.get("r2")
    sample = {
        "sample_id": sample_id,
        "r1": Path(r1),
        "r2": Path(str(r2)) if r2 else None,
        "layout": "pe" if r2 else "se",
    }
    return sample, record


def assert_contract_tool_roster(sample_reports: list[Path], expected_tools: list[str]) -> None:
    observed_tools: set[str] = set()
    for sample_report in sample_reports:
        report = load_json(sample_report)
        for record in report.get("records") or []:
            context = record.get("context", {})
            tool = context.get("tool") or context.get("parameters", {}).get("tool")
            if tool:
                observed_tools.add(str(tool))
    if sorted(observed_tools) != sorted(expected_tools):
        raise SystemExit(
            "mirrored sample report tool coverage drift: "
            f"expected {expected_tools!r}, found {sorted(observed_tools)!r}"
        )


def repair_existing_manifest(run_root: Path, stage_id: str) -> dict:
    manifest_path = run_root / "run_manifest.json"
    if not manifest_path.is_file():
        return {
            "stage_id": stage_id,
            "status": "missing-stage-run-manifest",
            "path": str(manifest_path),
        }
    expected_tools = EXPECTED_STAGE_TOOLS.get(stage_id)
    if expected_tools is not None:
        try:
            assert_contract_tool_roster(sample_report_paths(run_root, stage_id), expected_tools)
        except SystemExit as exc:
            return {
                "stage_id": stage_id,
                "status": "tool-roster-incomplete",
                "detail": str(exc),
            }
    manifest = load_json(manifest_path)
    changed = False
    local_results_root = run_root.parents[2]
    for run in manifest.get("runs", []):
        sample_id = run.get("sample_id")
        localized_report = localize_results_path(str(run.get("report_json", "")), local_results_root)
        canonical_report = benchmark_sample_root(run_root, stage_id, sample_id) / "report.json"
        if localized_report.is_file():
            repaired_report = localized_report.resolve()
        elif canonical_report.is_file():
            repaired_report = canonical_report.resolve()
        else:
            raise SystemExit(
                f"unable to resolve mirrored report.json for {stage_id}/{sample_id}: "
                f"{run.get('report_json')!r}"
            )
        if run.get("report_json") != str(repaired_report):
            run["report_json"] = str(repaired_report)
            changed = True
    if changed:
        manifest_path.write_text(
            json.dumps(manifest, indent=2, sort_keys=True) + "\n",
            encoding="utf-8",
        )
    return {
        "stage_id": stage_id,
        "status": "updated-existing-manifest" if changed else "existing-manifest-already-canonical",
        "path": str(manifest_path),
    }


def build_validate_manifest(run_root: Path) -> dict:
    sample_reports = sample_report_paths(run_root, "fastq.validate_reads")
    if not sample_reports:
        raise SystemExit(f"no mirrored sample reports found under {run_root}")
    assert_contract_tool_roster(
        sample_reports,
        EXPECTED_STAGE_TOOLS["fastq.validate_reads"],
    )
    runs: list[dict] = []
    first_sample, first_record = sample_from_report(sample_reports[0].parent.name, sample_reports[0])
    first_parameters = first_record.get("context", {}).get("parameters", {})
    corpus_root = Path(required_path(first_parameters, "input_r1", "r1")).parents[1]
    remote_out_root = default_results_stage_root(corpus_root, "fastq.validate_reads")
    for sample_report in sample_reports:
        sample_id = sample_report.parent.name
        sample, _record = sample_from_report(sample_id, sample_report)
        runs.append(
            {
                "sample_id": sample_id,
                "r1": str(sample["r1"]),
                "r2": str(sample["r2"]) if sample["r2"] is not None else None,
                "layout": sample["layout"],
                "status": "completed",
                "exit_code": 0,
                "command": build_validate_reads_command(
                    out_root=remote_out_root,
                    platform=str(first_record.get("context", {}).get("platform", "lunarc-apptainer")),
                    tools=",".join(EXPECTED_STAGE_TOOLS["fastq.validate_reads"]),
                    threads=int(first_parameters.get("threads", 4) or 4),
                    jobs=1,
                    validation_mode=str(first_parameters.get("validation_mode", "strict")),
                    pair_sync_policy=str(
                        first_parameters.get("pair_sync_policy", "require_header_sync")
                    ),
                    sample=sample,
                ),
                "report_json": str(sample_report.resolve()),
            }
        )
    return {
        "schema_version": "bijux.fastq.validate_reads.corpus_run.v1",
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": "corpus-01",
        "stage_id": "fastq.validate_reads",
        "scenario_id": "validation_fairness",
        "tool_kind": "benchmark",
        "platform": str(first_record.get("context", {}).get("platform", "lunarc-apptainer")),
        "tools": EXPECTED_STAGE_TOOLS["fastq.validate_reads"],
        "threads": int(first_parameters.get("threads", 4) or 4),
        "jobs": 1,
        "validation_mode": str(first_parameters.get("validation_mode", "strict")),
        "pair_sync_policy": str(first_parameters.get("pair_sync_policy", "require_header_sync")),
        "sample_limit": None,
        "dry_run": False,
        "repo_root": str(benchmark_remote_repo_root()),
        "corpus_root": str(corpus_root),
        "out_root": str(remote_out_root),
        "samples_total": len(runs),
        "samples_failed": 0,
        "runs": runs,
    }


def build_single_tool_manifest(
    *,
    run_root: Path,
    contract,
    schema_version: str,
    runner_build_command,
    extra_fields: dict,
    command_extra_factory,
) -> dict:
    sample_reports = sample_report_paths(run_root, contract.stage_id)
    if not sample_reports:
        raise SystemExit(f"no mirrored sample reports found under {run_root}")
    assert_contract_tool_roster(sample_reports, contract.tools)
    runs: list[dict] = []
    first_sample, first_record = sample_from_report(sample_reports[0].parent.name, sample_reports[0])
    first_parameters = first_record.get("context", {}).get("parameters", {})
    corpus_root = Path(required_path(first_parameters, "input_r1", "r1")).parents[1]
    remote_out_root = default_results_stage_root(corpus_root, contract.stage_id)
    platform = str(first_record.get("context", {}).get("platform", "lunarc-apptainer"))
    threads = int(first_parameters.get("threads", 1) or 1)
    for sample_report in sample_reports:
        sample_id = sample_report.parent.name
        sample, _record = sample_from_report(sample_id, sample_report)
        command = runner_build_command(
            out_root=remote_out_root,
            platform=platform,
            tools=",".join(contract.tools),
            threads=threads,
            jobs=1,
            sample=sample,
            **command_extra_factory(first_parameters),
        )
        runs.append(
            {
                "sample_id": sample_id,
                "r1": str(sample["r1"]),
                "r2": str(sample["r2"]) if sample["r2"] is not None else None,
                "layout": sample["layout"],
                "status": "completed",
                "exit_code": 0,
                "command": command,
                "report_json": str(sample_report.resolve()),
            }
        )
    payload = {
        "schema_version": schema_version,
        "generated_at_utc": datetime.now(timezone.utc).isoformat(),
        "corpus_id": "corpus-01",
        "stage_id": contract.stage_id,
        "scenario_id": contract.scenario_id,
        "tool_kind": "benchmark",
        "platform": platform,
        "tools": contract.tools,
        "threads": threads,
        "jobs": 1,
        "sample_limit": None,
        "dry_run": False,
        "repo_root": str(benchmark_remote_repo_root()),
        "corpus_root": str(corpus_root),
        "out_root": str(remote_out_root),
        "samples_total": len(runs),
        "samples_failed": 0,
        "runs": runs,
    }
    payload.update(extra_fields)
    return payload


def write_manifest(run_root: Path, payload: dict) -> Path:
    manifest_path = run_root / "run_manifest.json"
    manifest_path.parent.mkdir(parents=True, exist_ok=True)
    manifest_path.write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )
    return manifest_path


def repair_stage(run_root: Path, stage_id: str) -> dict:
    if stage_id == "fastq.validate_reads":
        manifest_path = run_root / "run_manifest.json"
        if manifest_path.is_file():
            return repair_existing_manifest(run_root, stage_id)
        try:
            manifest = build_validate_manifest(run_root)
        except SystemExit as exc:
            return {
                "stage_id": stage_id,
                "status": "tool-roster-incomplete",
                "detail": str(exc),
            }
        return {
            "stage_id": stage_id,
            "status": "reconstructed-stage-run-manifest",
            "path": str(write_manifest(run_root, manifest)),
        }
    if stage_id == "fastq.detect_adapters":
        manifest_path = run_root / "run_manifest.json"
        if manifest_path.is_file():
            return repair_existing_manifest(run_root, stage_id)
        try:
            manifest = build_single_tool_manifest(
            run_root=run_root,
            contract=DETECT_ADAPTERS_BENCHMARK_CONTRACT,
            schema_version="bijux.fastq.detect_adapters.corpus_run.v1",
            runner_build_command=lambda **kwargs: build_fastq_stage_command(
                bench_subcommand="detect-adapters",
                **kwargs,
            ),
            extra_fields={
                "inspection_mode": "evidence_only",
                "report_only": True,
                "evidence_scope": "full_input",
                "evidence_format": "fastqc_summary",
            },
            command_extra_factory=lambda _params: {},
            )
        except SystemExit as exc:
            return {
                "stage_id": stage_id,
                "status": "tool-roster-incomplete",
                "detail": str(exc),
            }
        return {
            "stage_id": stage_id,
            "status": "reconstructed-stage-run-manifest",
            "path": str(write_manifest(run_root, manifest)),
        }
    if stage_id == "fastq.profile_reads":
        manifest_path = run_root / "run_manifest.json"
        if manifest_path.is_file():
            return repair_existing_manifest(run_root, stage_id)
        try:
            manifest = build_single_tool_manifest(
            run_root=run_root,
            contract=PROFILE_READS_BENCHMARK_CONTRACT,
            schema_version="bijux.fastq.profile_reads.corpus_run.v1",
            runner_build_command=lambda **kwargs: build_fastq_stage_command(
                bench_subcommand="profile-reads",
                **kwargs,
            ),
            extra_fields={
                "report_only": True,
                "mutates_fastq": False,
                "may_change_read_count": False,
                "raw_backend_report_format": "seqkit_stats_tsv",
                "length_histogram_source": "seqkit_fx2tab",
            },
            command_extra_factory=lambda _params: {},
            )
        except SystemExit as exc:
            return {
                "stage_id": stage_id,
                "status": "tool-roster-incomplete",
                "detail": str(exc),
            }
        return {
            "stage_id": stage_id,
            "status": "reconstructed-stage-run-manifest",
            "path": str(write_manifest(run_root, manifest)),
        }
    if stage_id == "fastq.profile_read_lengths":
        manifest_path = run_root / "run_manifest.json"
        if manifest_path.is_file():
            return repair_existing_manifest(run_root, stage_id)
        try:
            manifest = build_single_tool_manifest(
            run_root=run_root,
            contract=PROFILE_READ_LENGTHS_BENCHMARK_CONTRACT,
            schema_version="bijux.fastq.profile_read_lengths.corpus_run.v1",
            runner_build_command=lambda histogram_bins, **kwargs: build_fastq_stage_command(
                bench_subcommand="profile-read-lengths",
                extra_args=["--histogram-bins", str(histogram_bins)],
                **kwargs,
            ),
            extra_fields={
                "histogram_bins": 100,
                "report_only": True,
                "mutates_fastq": False,
                "may_change_read_count": False,
                "raw_backend_report_format": "seqkit_stats_length_histogram",
                "length_histogram_artifacts": [
                    "report_json",
                    "length_distribution_tsv",
                    "length_distribution_json",
                ],
            },
            command_extra_factory=lambda params: {
                "histogram_bins": int(params.get("histogram_bins", 100) or 100),
            },
            )
        except SystemExit as exc:
            return {
                "stage_id": stage_id,
                "status": "tool-roster-incomplete",
                "detail": str(exc),
            }
        return {
            "stage_id": stage_id,
            "status": "reconstructed-stage-run-manifest",
            "path": str(write_manifest(run_root, manifest)),
        }
    if stage_id == "fastq.merge_pairs":
        return repair_existing_manifest(run_root, stage_id)
    raise SystemExit(f"unsupported stage repair target: {stage_id}")


def main() -> int:
    args = parse_args()
    results_root = Path(args.results_root).expanduser().resolve()
    corpus_root = results_root / args.corpus_id
    report = {
        "results_root": str(results_root),
        "corpus_id": args.corpus_id,
        "stages": [],
    }
    for stage_id in normalize_stage_csv(args.stages):
        run_root = corpus_root / stage_id / "lunarc"
        report["stages"].append(repair_stage(run_root, stage_id))
    print(json.dumps(report, indent=2))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
