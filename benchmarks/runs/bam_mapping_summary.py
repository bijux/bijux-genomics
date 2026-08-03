"""Prepare replayable Picard and Samtools mapping-summary benchmark contracts."""

from __future__ import annotations

import argparse
import datetime as dt
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

import yaml

try:
    import tomllib
except ModuleNotFoundError:
    import tomli as tomllib

RUN_ID = "run-2004"
STAGE_ID = "bam.mapping_summary"
STAGE_SLUG = "bam-mapping-summary"
CORPUS_ID = "ena-prjeb22390-err2112767-bam"
TOOLS = ("picard", "samtools")
IMAGE_SCHEMA = "bijux.container.image_inventory.v1"
DATA_SCHEMA = "bijux.data_index.v1"
GOVERNANCE_SCHEMA = "bijux.container.governance.v1"
CATALOG_SCHEMA = "bijux.container.catalog.v1"


class SpecificationError(RuntimeError):
    """Raised when replay inputs cannot support an exact benchmark contract."""


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise SpecificationError(f"cannot read JSON {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"JSON root must be an object: {path}")
    return value


def load_yaml(path: Path) -> dict[str, Any]:
    try:
        value = yaml.safe_load(path.read_text(encoding="utf-8"))
    except (OSError, yaml.YAMLError) as exc:
        raise SpecificationError(f"cannot read YAML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"YAML root must be a mapping: {path}")
    return value


def load_toml(path: Path) -> dict[str, Any]:
    try:
        value = tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as exc:
        raise SpecificationError(f"cannot read TOML {path}: {exc}") from exc
    if not isinstance(value, dict):
        raise SpecificationError(f"TOML root must be a mapping: {path}")
    return value


def runtime_contracts(catalog: dict[str, Any]) -> dict[str, dict[str, Any]]:
    if catalog.get("schema_version") != CATALOG_SCHEMA:
        raise SpecificationError("container catalog schema is unsupported")
    contracts: dict[str, dict[str, Any]] = {}
    for record in catalog.get("tools", []):
        if not isinstance(record, dict) or record.get("id") not in TOOLS:
            continue
        if record.get("support_disposition") != "retained":
            raise SpecificationError(f"{record.get('id')} is not retained")
        tool = str(record["id"])
        fields = ("version_command", "help_command", "expected_version_regex")
        if any(not isinstance(record.get(field), str) or not record[field] for field in fields):
            raise SpecificationError(f"{tool} runtime contract is incomplete")
        contracts[tool] = {
            "version": shlex.split(str(record["version_command"])),
            "help": shlex.split(str(record["help_command"])),
            "version_regex": str(record["expected_version_regex"]),
        }
    if set(contracts) != set(TOOLS):
        raise SpecificationError("container catalog lacks the complete tool closure")
    return contracts


def study_path(uri: str, study_root: Path) -> Path:
    prefix = "study://"
    value = Path(uri)
    if value.is_absolute():
        path = value.resolve()
    elif uri.startswith(prefix):
        path = (study_root / uri.removeprefix(prefix)).resolve()
    else:
        raise SpecificationError(f"governed path is not study-relative: {uri}")
    try:
        path.relative_to(study_root.resolve())
    except ValueError as exc:
        raise SpecificationError(f"governed path escapes the study root: {uri}") from exc
    return path


def current_commit(repository: Path) -> str:
    status = subprocess.run(
        ["git", "-C", str(repository), "status", "--porcelain"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout
    if status:
        raise SpecificationError("active repository must be clean")
    commit = subprocess.run(
        ["git", "-C", str(repository), "rev-parse", "HEAD"],
        check=True,
        capture_output=True,
        text=True,
    ).stdout.strip()
    if len(commit) != 40:
        raise SpecificationError("repository HEAD is not a full commit identity")
    return commit


def governed_roots(governance: dict[str, Any]) -> dict[str, Path | str]:
    if governance.get("schema_version") != GOVERNANCE_SCHEMA:
        raise SpecificationError("governance schema is unsupported")
    paths = governance.get("paths")
    slurm = governance.get("slurm")
    if not isinstance(paths, dict) or not isinstance(slurm, dict):
        raise SpecificationError("governance paths and Slurm policy are required")
    required = {
        "study_root",
        "containers_root",
        "data_root",
        "results_root",
        "active_repository",
        "frozen_repositories",
    }
    if not required.issubset(paths):
        raise SpecificationError("governance paths are incomplete")
    partitions = slurm.get("partitions")
    if not isinstance(partitions, list) or not partitions:
        raise SpecificationError("governance must provide a Slurm partition")
    roots: dict[str, Path | str] = {key: Path(str(paths[key])) for key in required}
    roots["account"] = str(slurm.get("account", ""))
    roots["partition"] = str(partitions[0])
    if not roots["account"]:
        raise SpecificationError("governance must provide a Slurm account")
    return roots


def selected_images(
    inventory: dict[str, Any], study_root: Path
) -> dict[str, dict[str, str]]:
    if inventory.get("schema_version") != IMAGE_SCHEMA:
        raise SpecificationError("image inventory schema is unsupported")
    selected: dict[str, dict[str, str]] = {}
    for record in inventory.get("tools", []):
        if not isinstance(record, dict) or record.get("tool") not in TOOLS:
            continue
        if record.get("support_disposition") != "retained" or record.get("state") != "current":
            raise SpecificationError(f"{record.get('tool')} lacks a current retained image")
        tool = str(record["tool"])
        selected[tool] = {
            "path": str(study_path(str(record["selected_image"]), study_root)),
            "sha256": str(record["selected_image_sha256"]),
        }
    if set(selected) != set(TOOLS):
        raise SpecificationError("image inventory lacks the complete tool closure")
    return selected


def corpus_inputs(
    index: dict[str, Any], study_root: Path
) -> list[dict[str, str]]:
    if index.get("schema_version") != DATA_SCHEMA:
        raise SpecificationError("data index schema is unsupported")
    records = [
        record
        for record in index.get("records", [])
        if isinstance(record, dict) and record.get("id") == CORPUS_ID
    ]
    if len(records) != 1 or records[0].get("benchmark_eligibility") != "eligible":
        raise SpecificationError(f"{CORPUS_ID} is not uniquely benchmark-eligible")
    record = records[0]
    files = record.get("files", [])
    wanted = {
        ".sorted.bam": ("err2112767_bam", "/input/ERR2112767.sorted.bam"),
        ".sorted.bam.bai": (
            "err2112767_bam_index",
            "/input/ERR2112767.sorted.bam.bai",
        ),
    }
    inputs = []
    for suffix, (input_id, mount_path) in wanted.items():
        matches = [
            item
            for item in files
            if isinstance(item, dict) and str(item.get("path", "")).endswith(suffix)
        ]
        if len(matches) != 1:
            raise SpecificationError(f"data index lacks one governed {suffix} input")
        item = matches[0]
        inputs.append(
            {
                "id": input_id,
                "kind": "data",
                "path": str(study_path(str(item["path"]), study_root)),
                "mount_path": mount_path,
                "sha256": str(item["sha256"]),
            }
        )
    inputs.append(
        {
            "id": "err2112767_catalog",
            "kind": "data",
            "path": str(study_path(str(record["catalog_path"]), study_root)),
            "mount_path": "/input/ERR2112767.catalog.json",
            "sha256": str(record["catalog_sha256"]),
        }
    )
    return inputs


def resource_conversion() -> str:
    return (
        "awk '{for (field=1; field<=NF; field++) {split($field, parts, \"=\"); "
        "printf \"%s\\t%s\\n\", parts[1], parts[2]}}' \"$resource_raw\" > \"$resource\";"
    )


def picard_command() -> str:
    return " ".join(
        (
            "set -eu;",
            "mkdir -p /result/repetitions;",
            "mkdir -p /scratch/picard-warmup;",
            "picard CollectAlignmentSummaryMetrics I=/input/ERR2112767.sorted.bam O=/scratch/warmup.tsv VALIDATION_STRINGENCY=SILENT TMP_DIR=/scratch/picard-warmup;",
            "for repetition in 1 2 3; do",
            'raw="/result/repetitions/repetition-${repetition}.picard.tsv";',
            'summary="/result/repetitions/repetition-${repetition}.mapping-summary.tsv";',
            'metrics="/result/repetitions/repetition-${repetition}.metrics.json";',
            'resource="/result/repetitions/repetition-${repetition}.resource.tsv";',
            'resource_raw="/scratch/repetition-${repetition}.resource.txt";',
            'mkdir -p "/scratch/picard-${repetition}";',
            "/usr/bin/time -f 'wall_seconds=%e user_seconds=%U system_seconds=%S peak_rss_kib=%M' -o \"$resource_raw\" picard CollectAlignmentSummaryMetrics I=/input/ERR2112767.sorted.bam O=\"$raw\" VALIDATION_STRINGENCY=SILENT TMP_DIR=\"/scratch/picard-${repetition}\";",
            resource_conversion(),
            "awk -F '\\t' '$1==\"CATEGORY\" {for (field=1; field<=NF; field++) column[$field]=field} $1==\"PAIR\" || $1==\"UNPAIRED\" {total += $(column[\"TOTAL_READS\"]); mapped += $(column[\"PF_READS_ALIGNED\"]); if ($1==\"PAIR\") paired += $(column[\"TOTAL_READS\"])} END {if (total <= 0) exit 2; printf \"metric\\tvalue\\nprimary_reads\\t%.0f\\nmapped_primary_reads\\t%.0f\\nmapped_primary_fraction\\t%.12f\\npaired_primary_reads\\t%.0f\\n\", total, mapped, mapped/total, paired}' \"$raw\" > \"$summary\";",
            'output_bytes=$(wc -c "$raw" "$summary" | awk \'END {print $1}\');',
            "awk -F '\\t' -v bytes=\"$output_bytes\" 'NR > 1 {value[$1]=$2} END {printf \"{\\\"primary_reads\\\":%s,\\\"mapped_primary_reads\\\":%s,\\\"mapped_primary_fraction\\\":%s,\\\"paired_primary_reads\\\":%s,\\\"output_bytes\\\":%s}\\n\", value[\"primary_reads\"], value[\"mapped_primary_reads\"], value[\"mapped_primary_fraction\"], value[\"paired_primary_reads\"], bytes}' \"$summary\" > \"$metrics\";",
            "done;",
            "write_context",
        )
    )


def samtools_command() -> str:
    return " ".join(
        (
            "set -eu;",
            "mkdir -p /result/repetitions;",
            "samtools flagstat /input/ERR2112767.sorted.bam > /scratch/warmup.txt;",
            "for repetition in 1 2 3; do",
            'raw="/result/repetitions/repetition-${repetition}.flagstat.txt";',
            'summary="/result/repetitions/repetition-${repetition}.mapping-summary.tsv";',
            'metrics="/result/repetitions/repetition-${repetition}.metrics.json";',
            'resource="/result/repetitions/repetition-${repetition}.resource.tsv";',
            'resource_raw="/scratch/repetition-${repetition}.resource.txt";',
            "/usr/bin/time -f 'wall_seconds=%e user_seconds=%U system_seconds=%S peak_rss_kib=%M' -o \"$resource_raw\" sh -c 'samtools flagstat /input/ERR2112767.sorted.bam > \"$1\"' sh \"$raw\";",
            resource_conversion(),
            "awk '$4==\"primary\" && NF==4 {total=$1+$3} $4==\"primary\" && $5==\"mapped\" {mapped=$1+$3} $4==\"paired\" && $5==\"in\" && $6==\"sequencing\" {paired=$1+$3} END {if (total <= 0) exit 2; printf \"metric\\tvalue\\nprimary_reads\\t%.0f\\nmapped_primary_reads\\t%.0f\\nmapped_primary_fraction\\t%.12f\\npaired_primary_reads\\t%.0f\\n\", total, mapped, mapped/total, paired}' \"$raw\" > \"$summary\";",
            'output_bytes=$(wc -c "$raw" "$summary" | awk \'END {print $1}\');',
            "awk -F '\\t' -v bytes=\"$output_bytes\" 'NR > 1 {value[$1]=$2} END {printf \"{\\\"primary_reads\\\":%s,\\\"mapped_primary_reads\\\":%s,\\\"mapped_primary_fraction\\\":%s,\\\"paired_primary_reads\\\":%s,\\\"output_bytes\\\":%s}\\n\", value[\"primary_reads\"], value[\"mapped_primary_reads\"], value[\"mapped_primary_fraction\"], value[\"paired_primary_reads\"], bytes}' \"$summary\" > \"$metrics\";",
            "done;",
            "write_context",
        )
    )


def complete_command(command: str) -> str:
    context = (
        "bam_bytes=$(wc -c < /input/ERR2112767.sorted.bam | tr -d ' '); "
        "bam_index_bytes=$(wc -c < /input/ERR2112767.sorted.bam.bai | tr -d ' '); "
        "printf '{\"warmup_repetitions\":1,\"measured_repetitions\":3,"
        "\"cache_policy\":\"warm\",\"threads\":1,\"bam_input_bytes\":%s,"
        "\"bam_index_input_bytes\":%s}\\n' \"$bam_bytes\" \"$bam_index_bytes\" "
        "> /result/benchmark-context.json"
    )
    return command.replace("write_context", context)


def required_files(tool: str) -> list[str]:
    raw_suffix = "picard.tsv" if tool == "picard" else "flagstat.txt"
    files = ["benchmark-context.json"]
    for repetition in range(1, 4):
        prefix = f"repetitions/repetition-{repetition}"
        files.extend(
            (
                f"{prefix}.{raw_suffix}",
                f"{prefix}.mapping-summary.tsv",
                f"{prefix}.resource.tsv",
                f"{prefix}.metrics.json",
            )
        )
    return files


def pipeline(
    tool: str,
    roots: dict[str, Path | str],
    commit: str,
    image: dict[str, str],
    inputs: list[dict[str, str]],
    submitted_at: str,
    runtime_contract: dict[str, Any],
) -> dict[str, Any]:
    command = picard_command() if tool == "picard" else samtools_command()
    negative = ["picard", "CollectAlignmentSummaryMetrics", "I=/input/missing.bam", "O=/scratch/missing.tsv"] if tool == "picard" else ["samtools", "flagstat", "/input/missing.bam"]
    result_root = Path(str(roots["results_root"])) / "benchmarks" / f"{RUN_ID}-{STAGE_SLUG}" / tool
    metrics = [
        {
            "id": metric,
            "path": "repetitions/repetition-1.metrics.json",
            "json_pointer": f"/{metric}",
            "operator": "gt",
            "expected": 0,
            "unit": unit,
        }
        for metric, unit in (
            ("primary_reads", "reads"),
            ("mapped_primary_reads", "reads"),
            ("mapped_primary_fraction", "fraction"),
            ("paired_primary_reads", "reads"),
        )
    ]
    return {
        "schema_version": "bijux.container.pipeline.v2",
        "run": {
            "id": RUN_ID,
            "kind": "benchmark",
            "campaign": "ancient-horse-bam-mapping-summary-benchmark",
            "corpus": CORPUS_ID,
            "sample": "ERR2112767",
            "domain": "bam",
            "stage": STAGE_ID,
            "tool": tool,
            "operator": "bijan",
            "submitted_at_utc": submitted_at,
        },
        "repository": {
            "path": str(Path(str(roots["frozen_repositories"])) / commit),
            "commit": commit,
            "clean": True,
        },
        "image": image,
        "data": {"inputs": inputs},
        "commands": {
            "version": runtime_contract["version"],
            "help": runtime_contract["help"],
            "real": ["sh", "-c", complete_command(command)],
            "negative": negative,
        },
        "outputs": {
            "result_root": str(result_root),
            "mount_path": "/result",
            "required_files": required_files(tool),
            "collection_rule": "required_files_and_metrics",
        },
        "resources": {
            "account": roots["account"],
            "partition": roots["partition"],
            "cpus": 1,
            "memory_mb": 8192,
            "walltime": "00:30:00",
            "scratch_mb": 4096,
            "retry": {
                "max_attempts": 1,
                "backoff_seconds": 0,
                "retryable_exit_codes": [],
            },
        },
        "execution": {
            "locale": "C",
            "timezone": "UTC",
            "runtime_network": False,
            "threads": 1,
            "random_seed": None,
            "cache_policy": "warm",
            "determinism": "deterministic",
        },
        "dependencies": [],
        "acceptance": {
            "version_regex": runtime_contract["version_regex"],
            "negative_exit_nonzero": True,
            "metrics": metrics,
        },
    }


def prepare(
    governance_path: Path,
    data_index_path: Path,
    image_inventory_path: Path,
    catalog_path: Path,
    output_root: Path | None,
    submitted_at: str | None = None,
) -> Path:
    governance = load_yaml(governance_path)
    roots = governed_roots(governance)
    study_root = Path(str(roots["study_root"]))
    repository = Path(str(roots["active_repository"]))
    commit = current_commit(repository)
    if data_index_path.parent.name != commit:
        raise SpecificationError(
            "data index directory must identify the exact repository commit"
        )
    images = selected_images(load_json(image_inventory_path), study_root)
    inputs = corpus_inputs(load_json(data_index_path), study_root)
    contracts = runtime_contracts(load_toml(catalog_path))
    destination = output_root or (
        Path(str(roots["results_root"])) / "benchmark-specifications"
    )
    tool_roots = {tool: destination / f"{RUN_ID}-{tool}" for tool in TOOLS}
    existing = [path for path in tool_roots.values() if path.exists()]
    if existing:
        raise SpecificationError(
            f"refusing to replace benchmark specifications: {existing[0]}"
        )
    timestamp = submitted_at or dt.datetime.now(dt.timezone.utc).isoformat()
    credentials = {
        "schema_version": "bijux.container.credential_reference.v1",
        "credentials_required": False,
        "references": [],
        "redaction": {"publish_values": False},
    }
    for tool in TOOLS:
        tool_root = tool_roots[tool]
        tool_root.mkdir(parents=True)
        (tool_root / "config-pipeline.yml").write_text(
            yaml.safe_dump(
                pipeline(
                    tool,
                    roots,
                    commit,
                    images[tool],
                    inputs,
                    timestamp,
                    contracts[tool],
                ),
                sort_keys=False,
            ),
            encoding="utf-8",
        )
        (tool_root / "config-credential.yml").write_text(
            yaml.safe_dump(credentials, sort_keys=False), encoding="utf-8"
        )
    return destination


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--governance", type=Path, required=True)
    root.add_argument("--data-index", type=Path, required=True)
    root.add_argument("--image-inventory", type=Path, required=True)
    root.add_argument("--catalog", type=Path, required=True)
    root.add_argument("--output-root", type=Path)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        print(
            prepare(
                args.governance,
                args.data_index,
                args.image_inventory,
                args.catalog,
                args.output_root,
            )
        )
        return 0
    except (SpecificationError, subprocess.CalledProcessError) as exc:
        print(f"mapping-summary specification failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
