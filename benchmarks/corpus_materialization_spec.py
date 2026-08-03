#!/usr/bin/env python3
"""Prepare and submit a governed Slurm FASTQ corpus materialization."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

PIPELINE_SCHEMA = "bijux.benchmark.corpus_materialization.v1"
CREDENTIAL_SCHEMA = "bijux.container.credential_reference.v1"
PLAN_SCHEMA = "bijux.benchmark.corpus_acquisition.v1"
COMMIT = re.compile(r"[0-9a-f]{40}")
ACTIVE_STATES = {
    "CONFIGURING",
    "COMPLETING",
    "PENDING",
    "REQUEUED",
    "RESIZING",
    "RUNNING",
    "SUSPENDED",
}


class SpecificationError(RuntimeError):
    """Raised when data transfer cannot be frozen into a governed Slurm unit."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def git_output(repository: Path, *arguments: str) -> str:
    completed = subprocess.run(
        ["git", "-C", str(repository), *arguments],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise SpecificationError(
            f"git {' '.join(arguments)} failed for {repository}: {completed.stderr.strip()}"
        )
    return completed.stdout.strip()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SpecificationError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"JSON root must be an object: {path}")
    return value


def governed_child(path: Path, root: Path, label: str) -> Path:
    path = path.resolve()
    try:
        path.relative_to(root.resolve())
    except ValueError as exc:
        raise SpecificationError(f"{label} escapes governed root {root}: {path}") from exc
    return path


def validate_repository(repository: Path, expected_commit: str | None = None) -> str:
    commit = git_output(repository, "rev-parse", "HEAD")
    if COMMIT.fullmatch(commit) is None:
        raise SpecificationError(f"repository has no full commit identity: {repository}")
    if expected_commit is not None and commit != expected_commit:
        raise SpecificationError(f"frozen repository commit disagrees: {repository}")
    if git_output(repository, "status", "--porcelain"):
        raise SpecificationError(f"repository must be clean: {repository}")
    return commit


def build_specification(
    governance: dict[str, Any],
    plan_path: Path,
    repository: Path,
    frozen_repository: Path,
    output: Path,
    jobs: int,
) -> tuple[dict[str, Any], dict[str, Any], str]:
    if jobs < 1:
        raise SpecificationError("download job count must be positive")
    data_root = Path(str(governance["data_root"])).resolve()
    plan_path = governed_child(plan_path, data_root, "acquisition plan")
    output = governed_child(output, data_root / "evidence" / "materializations", "specification")
    plan = load_json(plan_path)
    if plan.get("schema_version") != PLAN_SCHEMA:
        raise SpecificationError("acquisition plan schema is unsupported")
    corpus_id = str(plan.get("portfolio_id", ""))
    if not corpus_id:
        raise SpecificationError("acquisition plan lacks a portfolio identity")
    commit = validate_repository(repository)
    validate_repository(frozen_repository, commit)
    materializer = frozen_repository / "benchmarks" / "corpus_materialization.py"
    if not materializer.is_file():
        raise SpecificationError("frozen repository lacks the corpus materializer")
    command = [
        "python3",
        "-B",
        str(materializer),
        "--plan",
        str(plan_path),
        "--data-root",
        str(data_root),
        "--jobs",
        str(jobs),
    ]
    resources = {
        "cpus": jobs,
        "memory_mb": max(4096, jobs * 1024),
        "walltime": "04:00:00",
    }
    pipeline = {
        "schema_version": PIPELINE_SCHEMA,
        "corpus": {
            "id": corpus_id,
            "legacy_id": plan.get("corpus_id"),
            "sample_count": plan.get("sample_count"),
            "file_count": plan.get("file_count"),
            "source_size_bytes": plan.get("total_size_bytes"),
        },
        "repository": {
            "commit": commit,
            "clean": True,
            "frozen_path": str(frozen_repository.resolve()),
            "materializer_sha256": sha256_file(materializer),
        },
        "input": {
            "acquisition_plan": str(plan_path),
            "acquisition_plan_sha256": sha256_file(plan_path),
        },
        "execution": {
            "boundary": "site-python-data-transfer",
            "scientific_binaries": [],
            "command": command,
            "network_purpose": "retrieve checksum-pinned public ENA FASTQ files",
        },
        "scheduler": {
            "site": governance["site_id"],
            "account": governance["slurm_account"],
            "partition": governance["slurm_partitions"][0],
            "resources": resources,
        },
        "outputs": {
            "corpus_root": str(data_root / "corpora" / corpus_id),
            "catalog": str(data_root / "catalog" / f"{corpus_id}.json"),
            "specification": str(output),
        },
        "acceptance": {
            "source_size_and_md5": "required",
            "gzip_integrity": "required",
            "fastq_structure": "required",
            "read_and_base_counts": "required",
            "local_sha256": "required",
            "catalog_publication": "after-complete-cohort-only",
        },
    }
    credentials = {
        "schema_version": CREDENTIAL_SCHEMA,
        "credentials_required": False,
        "references": [],
        "redaction": {"publish_values": False},
    }
    job_name = f"corpus-{corpus_id}"[:64]
    stdout = output / "slurm-%j.stdout.log"
    stderr = output / "slurm-%j.stderr.log"
    slurm = "\n".join(
        (
            "#!/usr/bin/env bash",
            f"#SBATCH --job-name={job_name}",
            f"#SBATCH --account={governance['slurm_account']}",
            f"#SBATCH --partition={governance['slurm_partitions'][0]}",
            f"#SBATCH --cpus-per-task={resources['cpus']}",
            f"#SBATCH --mem={resources['memory_mb']}M",
            f"#SBATCH --time={resources['walltime']}",
            f"#SBATCH --output={stdout}",
            f"#SBATCH --error={stderr}",
            "set -euo pipefail",
            "umask 027",
            "export LC_ALL=C",
            "export TZ=UTC",
            " ".join(shlex.quote(value) for value in command),
            "",
        )
    )
    return pipeline, credentials, slurm


def write_specification(
    governance_path: Path,
    plan_path: Path,
    repository: Path,
    frozen_repository: Path,
    output: Path,
    jobs: int,
) -> None:
    repository = repository.resolve()
    contract_modules = repository / "containers" / "contracts"
    sys.path.insert(0, str(repository))
    sys.path.insert(0, str(contract_modules))
    try:
        from containers.contracts.container_validation import load_governance
        import yaml
    except ImportError as exc:
        raise SpecificationError("PyYAML and container governance are required") from exc
    finally:
        sys.path.remove(str(contract_modules))
        sys.path.remove(str(repository))
    governance = load_governance(governance_path)
    pipeline, credentials, slurm = build_specification(
        governance, plan_path, repository, frozen_repository, output, jobs
    )
    if output.exists() or output.is_symlink():
        raise SpecificationError(f"refusing to replace specification: {output}")
    output.mkdir(parents=True, mode=0o750)
    (output / "config-pipeline.yml").write_text(
        yaml.safe_dump(pipeline, sort_keys=False), encoding="utf-8"
    )
    (output / "config-credential.yml").write_text(
        yaml.safe_dump(credentials, sort_keys=False), encoding="utf-8"
    )
    script = output / "submit.sbatch"
    script.write_text(slurm, encoding="utf-8")
    script.chmod(0o750)


def submit(specification: Path) -> str:
    specification = specification.resolve()
    script = specification / "submit.sbatch"
    job_record = specification / "slurm-job-id.txt"
    required = (
        specification / "config-pipeline.yml",
        specification / "config-credential.yml",
        script,
    )
    if not all(path.is_file() for path in required):
        raise SpecificationError("materialization specification is incomplete")
    if job_record.exists() or job_record.is_symlink():
        raise SpecificationError(f"refusing duplicate submission: {job_record}")
    completed = subprocess.run(
        ["sbatch", "--parsable", str(script)],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise SpecificationError(f"sbatch failed: {completed.stderr.strip()}")
    job_id = completed.stdout.strip().split(";", 1)[0]
    if not job_id.isdigit():
        raise SpecificationError(f"sbatch returned an invalid job identity: {completed.stdout}")
    job_record.write_text(job_id + "\n", encoding="utf-8")
    return job_id


def verify_materialization(pipeline: dict[str, Any]) -> dict[str, Any]:
    if pipeline.get("schema_version") != PIPELINE_SCHEMA:
        raise SpecificationError("materialization pipeline schema is unsupported")
    corpus = pipeline.get("corpus")
    outputs = pipeline.get("outputs")
    if not isinstance(corpus, dict) or not isinstance(outputs, dict):
        raise SpecificationError("materialization pipeline identity is incomplete")
    catalog_path = Path(str(outputs.get("catalog", ""))).resolve()
    corpus_root = Path(str(outputs.get("corpus_root", ""))).resolve()
    catalog = load_json(catalog_path)
    files = catalog.get("files")
    if catalog.get("id") != corpus.get("id") or not isinstance(files, list):
        raise SpecificationError("materialized catalog identity disagrees")
    if len(files) != corpus.get("file_count"):
        raise SpecificationError("materialized catalog file count disagrees")
    observed_bytes = 0
    observed_reads = 0
    for record in files:
        if not isinstance(record, dict):
            raise SpecificationError("materialized catalog file is invalid")
        path = governed_child(Path(str(record.get("path", ""))), corpus_root, "FASTQ")
        digest = str(record.get("sha256", ""))
        size = record.get("size_bytes")
        reads = record.get("read_count")
        if (
            not path.is_file()
            or not isinstance(size, int)
            or isinstance(size, bool)
            or path.stat().st_size != size
            or sha256_file(path) != digest
            or not isinstance(reads, int)
            or isinstance(reads, bool)
            or reads < 1
        ):
            raise SpecificationError(f"materialized FASTQ identity is invalid: {path}")
        observed_bytes += size
        observed_reads += reads
    if observed_bytes != corpus.get("source_size_bytes"):
        raise SpecificationError("materialized catalog byte total disagrees")
    return {
        "catalog": str(catalog_path),
        "catalog_sha256": sha256_file(catalog_path),
        "file_count": len(files),
        "size_bytes": observed_bytes,
        "read_count": observed_reads,
    }


def collect(specification: Path) -> Path:
    try:
        import yaml
        import data_catalog
    except ImportError as exc:
        raise SpecificationError("PyYAML and benchmark data catalog are required") from exc
    specification = specification.resolve()
    evidence_path = specification / "materialization-evidence.json"
    accounting_path = specification / "slurm-accounting.json"
    if evidence_path.exists() or accounting_path.exists():
        raise SpecificationError("refusing to replace materialization evidence")
    pipeline_path = specification / "config-pipeline.yml"
    credential_path = specification / "config-credential.yml"
    job_path = specification / "slurm-job-id.txt"
    if not all(path.is_file() for path in (pipeline_path, credential_path, job_path)):
        raise SpecificationError("submitted materialization specification is incomplete")
    pipeline = yaml.safe_load(pipeline_path.read_text(encoding="utf-8"))
    if not isinstance(pipeline, dict):
        raise SpecificationError("materialization pipeline YAML is invalid")
    job_id = job_path.read_text(encoding="utf-8").strip()
    fields = ("JobID", "State", "ExitCode", "Elapsed", "MaxRSS", "AllocCPUS")
    completed = subprocess.run(
        ["sacct", "-X", "-n", "-P", "-j", job_id, "-o", ",".join(fields)],
        check=False,
        capture_output=True,
        text=True,
    )
    if completed.returncode != 0:
        raise SpecificationError(f"cannot collect Slurm accounting: {completed.stderr.strip()}")
    rows = []
    for line in completed.stdout.splitlines():
        if not line.strip():
            continue
        values = line.split("|")
        if len(values) != len(fields):
            raise SpecificationError(f"invalid Slurm accounting row: {line}")
        rows.append(dict(zip(fields, values)))
    if not rows:
        raise SpecificationError(f"Slurm accounting is unavailable for {job_id}")
    states = {row["State"].split("+", 1)[0] for row in rows}
    active = sorted(states & ACTIVE_STATES)
    if active:
        raise SpecificationError(f"Slurm job is not terminal: {active}")
    accounting = {
        "schema_version": "bijux.benchmark.slurm_accounting.v1",
        "job_id": job_id,
        "rows": rows,
    }
    primary = next((row for row in rows if row["JobID"] == job_id), rows[0])
    passed = primary["State"].split("+", 1)[0] == "COMPLETED" and primary[
        "ExitCode"
    ].startswith("0:")
    failure = None if passed else f"Slurm terminal state {primary['State']}"
    verified = None
    if passed:
        try:
            verified = verify_materialization(pipeline)
        except SpecificationError as exc:
            passed = False
            failure = str(exc)
    repository_commit = str(pipeline["repository"]["commit"])
    if verified is not None:
        data_root = Path(str(pipeline["outputs"]["catalog"])).resolve().parents[1]
        index_root = data_root / "evidence" / "indexes" / repository_commit
        index_path = index_root / "data-index.json"
        report_path = index_root / "data-index.md"
        if index_path.exists() or report_path.exists():
            raise SpecificationError("refusing to replace governed data index")
        try:
            index = data_catalog.build_index(data_root / "catalog", data_root)
            data_catalog.write_json(index_path, index)
            report_path.parent.mkdir(parents=True, exist_ok=True)
            report_path.write_text(data_catalog.render_report(index), encoding="utf-8")
        except data_catalog.DataCatalogError as exc:
            passed = False
            failure = str(exc)
            verified = None
    accounting_path.write_text(json.dumps(accounting, indent=2, sort_keys=True) + "\n")
    evidence = {
        "schema_version": "bijux.benchmark.corpus_materialization_evidence.v1",
        "status": "passed" if passed else "failed",
        "repository_commit": repository_commit,
        "job_id": job_id,
        "pipeline_sha256": sha256_file(pipeline_path),
        "credential_reference_sha256": sha256_file(credential_path),
        "accounting_sha256": sha256_file(accounting_path),
        "verified_corpus": verified,
        "failure": failure,
    }
    evidence_path.write_text(json.dumps(evidence, indent=2, sort_keys=True) + "\n")
    return evidence_path


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    commands = root.add_subparsers(dest="command", required=True)
    prepare = commands.add_parser("prepare")
    prepare.add_argument("--governance", type=Path, required=True)
    prepare.add_argument("--plan", type=Path, required=True)
    prepare.add_argument("--repository", type=Path, required=True)
    prepare.add_argument("--frozen-repository", type=Path, required=True)
    prepare.add_argument("--output", type=Path, required=True)
    prepare.add_argument("--jobs", type=int, default=4)
    submit_command = commands.add_parser("submit")
    submit_command.add_argument("--specification", type=Path, required=True)
    collect_command = commands.add_parser("collect")
    collect_command.add_argument("--specification", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        if args.command == "prepare":
            write_specification(
                args.governance,
                args.plan,
                args.repository,
                args.frozen_repository,
                args.output,
                args.jobs,
            )
            print(args.output.resolve())
        elif args.command == "submit":
            print(submit(args.specification))
        else:
            print(collect(args.specification))
        return 0
    except SpecificationError as exc:
        print(f"corpus materialization specification failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
