"""Prepare the real multi-tool FASTQ read-profile benchmark campaign."""

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
import shlex
import subprocess
import sys
from pathlib import Path
from typing import Any

import yaml

try:
    from benchmarks.runs.specification_support import (
        SpecificationError,
        current_commit,
        governed_roots,
        load_json,
        load_toml,
        load_yaml,
        runtime_contracts,
        selected_images,
        sha256_file,
        study_path,
    )
except ModuleNotFoundError:
    from specification_support import (  # type: ignore[no-redef]
        SpecificationError,
        current_commit,
        governed_roots,
        load_json,
        load_toml,
        load_yaml,
        runtime_contracts,
        selected_images,
        sha256_file,
        study_path,
    )

RUN_ID = "run-2043"
STAGE_ID = "fastq.profile_reads"
STAGE_SLUG = "fastq-profile-reads"
CORPUS_ID = "human-short-read-diversity"
TOOLS = ("seqfu", "seqkit", "seqkit_stats")
PARAMETER_SCHEMA = "bijux.fastq.params.stats.v1"
DEFAULTS_PROVIDER = "bijux_dna_domain_fastq::params::defaults::stats_defaults"
PROFILE_HEADER = (
    "sample_id\tinput_layout\tfile_count\tread_count\tbase_count\t"
    "minimum_read_length\tmaximum_read_length"
)
ADMISSION_HEADER = "sample_id\tinput_layout\tfile_count\tread_count\tbase_count"


def campaign_contract(repository: Path) -> dict[str, Any]:
    catalog = load_toml(repository / "benchmarks/contracts/stage-campaigns.toml")
    matches = [
        campaign
        for campaign in catalog.get("campaigns", [])
        if isinstance(campaign, dict) and campaign.get("run_id") == RUN_ID
    ]
    if len(matches) != 1:
        raise SpecificationError(f"{RUN_ID} campaign is not unique")
    campaign = matches[0]
    expected = {
        "stage_id": STAGE_ID,
        "tool_ids": list(TOOLS),
        "corpus_ids": [CORPUS_ID],
        "container_only": True,
        "threads": 4,
        "warmup_repetitions": 1,
        "measured_repetitions": 3,
    }
    for key, value in expected.items():
        if campaign.get(key) != value:
            raise SpecificationError(f"campaign {key} differs from executable scope")
    return campaign


def interface_contract(repository: Path) -> dict[str, Any]:
    catalog = load_toml(repository / "benchmarks/contracts/stage-interfaces.toml")
    matches = [
        interface
        for interface in catalog.get("interfaces", [])
        if isinstance(interface, dict) and interface.get("stage_id") == STAGE_ID
    ]
    if len(matches) != 1:
        raise SpecificationError(f"{STAGE_ID} interface is not unique")
    interface = matches[0]
    if interface.get("tool_ids") != list(TOOLS):
        raise SpecificationError(
            "stage interface differs from the retained tool closure"
        )
    return interface


def corpus_record(index: dict[str, Any]) -> dict[str, Any]:
    if index.get("schema_version") != "bijux.data_index.v1":
        raise SpecificationError("data index schema is unsupported")
    matches = [
        record
        for record in index.get("records", [])
        if isinstance(record, dict) and record.get("id") == CORPUS_ID
    ]
    if len(matches) != 1 or matches[0].get("benchmark_eligibility") != "eligible":
        raise SpecificationError(f"{CORPUS_ID} is not uniquely benchmark-eligible")
    if matches[0].get("integrity_status") != "verified":
        raise SpecificationError(f"{CORPUS_ID} lacks verified integrity")
    return matches[0]


def admitted_corpus(
    record: dict[str, Any], study_root: Path
) -> tuple[Path, dict[str, Any], list[dict[str, Any]]]:
    catalog_path = study_path(str(record["catalog_path"]), study_root)
    if sha256_file(catalog_path) != record.get("catalog_sha256"):
        raise SpecificationError("corpus catalog digest differs from the data index")
    catalog = load_json(catalog_path)
    if (
        catalog.get("schema_version") != "bijux.data_catalog.v1"
        or catalog.get("id") != CORPUS_ID
    ):
        raise SpecificationError("corpus catalog identity is unsupported")
    samples = catalog.get("samples")
    files = catalog.get("files")
    if not isinstance(samples, list) or not isinstance(files, list):
        raise SpecificationError("corpus catalog lacks sample and file records")
    if len(samples) != 20 or len(files) != 30:
        raise SpecificationError("real FASTQ campaign requires 20 runs and 30 files")

    indexed = {
        Path(str(item["path"])).name: item
        for item in record.get("files", [])
        if isinstance(item, dict)
    }
    admitted: list[dict[str, Any]] = []
    for item in files:
        if not isinstance(item, dict):
            raise SpecificationError("corpus file record must be a mapping")
        name = Path(str(item["path"])).name
        index_item = indexed.get(name)
        if not isinstance(index_item, dict):
            raise SpecificationError(f"data index lacks admitted file {name}")
        if item.get("sha256") != index_item.get("sha256") or item.get(
            "size_bytes"
        ) != index_item.get("size_bytes"):
            raise SpecificationError(f"catalog and data index disagree for {name}")
        admitted.append({**item, "governed_path": index_item["path"]})
    if len(indexed) != len(admitted):
        raise SpecificationError("data index and catalog FASTQ closures differ")
    return catalog_path, catalog, admitted


def expected_admission_rows(catalog: dict[str, Any]) -> list[str]:
    files = catalog["files"]
    rows = []
    for sample in sorted(catalog["samples"], key=lambda value: value["accession"]):
        accession = str(sample["accession"])
        sample_files = [item for item in files if item.get("accession") == accession]
        layout = "paired_end" if sample.get("layout") == "pe" else "single_end"
        expected_file_count = 2 if layout == "paired_end" else 1
        if len(sample_files) != expected_file_count:
            raise SpecificationError(
                f"{accession} file closure disagrees with its layout"
            )
        rows.append(
            "\t".join(
                (
                    accession,
                    layout,
                    str(len(sample_files)),
                    str(sum(int(item["read_count"]) for item in sample_files)),
                    str(sum(int(item["base_count"]) for item in sample_files)),
                )
            )
        )
    return rows


def materialize_admission_evidence(
    roots: dict[str, Path | str],
    data_index_path: Path,
    catalog_path: Path,
    catalog: dict[str, Any],
) -> tuple[Path, Path]:
    identity = sha256_file(catalog_path)
    root = Path(str(roots["data_root"])) / "evidence/benchmark-expectations" / identity
    root.mkdir(parents=True, exist_ok=True)
    expected_path = root / "fastq-profile-reads-admission.tsv"
    provenance_path = root / "fastq-profile-reads-admission.json"
    expected_content = (
        ADMISSION_HEADER + "\n" + "\n".join(expected_admission_rows(catalog)) + "\n"
    )
    provenance = {
        "schema_version": "bijux.benchmark.fastq.profile_reads.admission.v1",
        "corpus_id": CORPUS_ID,
        "sample_count": len(catalog["samples"]),
        "file_count": len(catalog["files"]),
        "data_index_sha256": sha256_file(data_index_path),
        "catalog_sha256": identity,
        "expected_profile_sha256": hashlib.sha256(
            expected_content.encode()
        ).hexdigest(),
    }
    expected_provenance = json.dumps(provenance, indent=2, sort_keys=True) + "\n"
    for path, content in (
        (expected_path, expected_content),
        (provenance_path, expected_provenance),
    ):
        if path.exists() and path.read_text(encoding="utf-8") != content:
            raise SpecificationError(f"refusing to replace admission evidence: {path}")
        path.write_text(content, encoding="utf-8")
    return expected_path, provenance_path


def data_inputs(
    admitted: list[dict[str, Any]],
    study_root: Path,
    data_index_path: Path,
    catalog_path: Path,
    expected_path: Path,
    provenance_path: Path,
) -> tuple[list[dict[str, str]], list[str]]:
    inputs: list[dict[str, str]] = []
    mounted_fastq: list[str] = []
    for item in sorted(admitted, key=lambda value: Path(value["governed_path"]).name):
        source = study_path(str(item["governed_path"]), study_root)
        name = source.name
        mount_path = f"/input/{name}"
        identifier = "fastq_" + name.removesuffix(".fastq.gz").lower()
        inputs.append(
            {
                "id": identifier,
                "kind": "data",
                "path": str(source),
                "mount_path": mount_path,
                "sha256": str(item["sha256"]),
            }
        )
        mounted_fastq.append(mount_path)
    support = (
        ("corpus_data_index", data_index_path, "/input/data-index.json"),
        ("corpus_catalog", catalog_path, "/input/corpus-catalog.json"),
        ("admission_profile", expected_path, "/input/expected-profile.tsv"),
        ("admission_provenance", provenance_path, "/input/expected-profile.json"),
    )
    for identifier, source, mount_path in support:
        inputs.append(
            {
                "id": identifier,
                "kind": "data",
                "path": str(source.resolve()),
                "mount_path": mount_path,
                "sha256": sha256_file(source),
            }
        )
    return inputs, mounted_fastq


def parameter_evidence(
    tool_root: Path,
    repository: Path,
    catalog: dict[str, Any],
) -> dict[str, str]:
    source = repository / "benchmarks/contracts/config-pipeline-parameters.yml"
    configured = load_yaml(source)
    stage = configured.get("stages", {}).get(STAGE_ID)
    if not isinstance(stage, dict):
        raise SpecificationError(f"parameter config lacks {STAGE_ID}")
    if (
        stage.get("parameter_schema_version") != PARAMETER_SCHEMA
        or stage.get("defaults_provider") != DEFAULTS_PROVIDER
        or stage.get("overrides") != {"threads": 4}
    ):
        raise SpecificationError(
            "parameter config differs from crate-owned defaults policy"
        )
    overrides_path = tool_root / "config-pipeline-parameters.yml"
    overrides_path.write_bytes(source.read_bytes())
    resolved = {
        str(sample["accession"]): {
            "schema_version": PARAMETER_SCHEMA,
            "paired_mode": (
                "paired_end" if sample.get("layout") == "pe" else "single_end"
            ),
            "threads": 4,
        }
        for sample in catalog["samples"]
    }
    effective_path = tool_root / "effective-parameters.json"
    effective = {
        "schema_version": "bijux.benchmark.effective_parameters.v1",
        "stage_id": STAGE_ID,
        "parameter_schema_version": PARAMETER_SCHEMA,
        "defaults_provider": DEFAULTS_PROVIDER,
        "overrides": {"threads": 4},
        "resolved_by_sample": dict(sorted(resolved.items())),
    }
    effective_path.write_text(
        json.dumps(effective, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return {
        "schema_version": "bijux.benchmark.parameter_evidence.v1",
        "parameter_schema_version": PARAMETER_SCHEMA,
        "defaults_provider": DEFAULTS_PROVIDER,
        "overrides_file": overrides_path.name,
        "overrides_sha256": sha256_file(overrides_path),
        "effective_file": effective_path.name,
        "effective_sha256": sha256_file(effective_path),
    }


def tool_command(tool: str, mounted_fastq: list[str]) -> list[str]:
    if tool == "seqfu":
        return ["seqfu", "stats", "--threads", "4", "--gc", *mounted_fastq]
    return [tool, "stats", "-a", "-T", "-j", "4", *mounted_fastq]


def shell_command(
    tool: str,
    mounted_fastq: list[str],
    input_bytes: int,
    *,
    warmup: bool,
) -> list[str]:
    command = shlex.join(tool_command(tool, mounted_fastq))
    if warmup:
        return ["sh", "-c", f"set -eu; {command} > /scratch/fastq-profile-warmup.tsv"]
    script = " ".join(
        (
            "set -eu;",
            "mkdir -p /result/repetitions;",
            'prefix="/result/repetitions/repetition-{repetition}";',
            'raw="${prefix}.raw.tsv";',
            'profile="${prefix}.read-profile.tsv";',
            f'{command} > "$raw";',
            f"printf '%s\\n' {shlex.quote(PROFILE_HEADER)} > \"$profile\";",
            (
                "awk -v mode=normalize -v tool="
                f"{shlex.quote(tool)} -f /repository/benchmarks/execution/fastq_profile_reads.awk "
                '"$raw" | sort >> "$profile";'
            ),
            'cut -f1-5 "$profile" > "${{prefix}}.admission-observed.tsv";'.replace(
                "${{prefix}}", "${prefix}"
            ),
            'diff -u /input/expected-profile.tsv "${prefix}.admission-observed.tsv";',
            (
                "awk -v mode=bundle -v tool="
                f"{shlex.quote(tool)} -f /repository/benchmarks/execution/fastq_profile_reads.awk "
                '"$profile" > "${prefix}.read-profile.json";'
            ),
            (
                "awk -v mode=probe -f "
                "/repository/benchmarks/execution/fastq_profile_reads.awk "
                '"$profile" > "${prefix}.report-qc.json";'
            ),
            'output_bytes=$(wc -c "$raw" "$profile" "${prefix}.read-profile.json" "${prefix}.report-qc.json" | awk \'END {print $1}\');',
            (
                f"awk -v mode=metrics -v input_bytes={input_bytes} "
                '-v output_bytes="$output_bytes" -f '
                "/repository/benchmarks/execution/fastq_profile_reads.awk "
                '"$profile" > "${prefix}.metrics.json";'
            ),
        )
    )
    return ["sh", "-c", script]


def required_files() -> list[str]:
    files = []
    for repetition in range(1, 4):
        prefix = f"repetitions/repetition-{repetition}"
        files.extend(
            (
                f"{prefix}.raw.tsv",
                f"{prefix}.read-profile.tsv",
                f"{prefix}.read-profile.json",
                f"{prefix}.report-qc.json",
                f"{prefix}.admission-observed.tsv",
                f"{prefix}.metrics.json",
                f"{prefix}.resource.tsv",
            )
        )
    return files


def interface_evidence(interface: dict[str, Any], tool: str) -> dict[str, Any]:
    return {
        "schema_version": "bijux.benchmark.stage_interface_evidence.v1",
        "accepted_input_contracts": interface["accepted_input_contracts"],
        "output_contract": interface["output_contract"],
        "normalized_result_contract": interface["normalized_result_contract"],
        "adapter": interface["adapters"][tool],
        "downstream_probe": {
            "stage_id": interface["downstream_probe"]["consumer_stage_id"],
            "input_contract": interface["downstream_probe"]["input_contract"],
            "source": interface["downstream_probe"]["source"],
            "artifact": "repetitions/repetition-1.report-qc.json",
        },
    }


def pipeline(
    tool: str,
    roots: dict[str, Path | str],
    commit: str,
    image: dict[str, str],
    inputs: list[dict[str, str]],
    mounted_fastq: list[str],
    input_bytes: int,
    submitted_at: str,
    runtime: dict[str, Any],
    parameters: dict[str, str],
    interface: dict[str, Any],
) -> dict[str, Any]:
    result_root = (
        Path(str(roots["results_root"]))
        / "benchmarks"
        / f"{RUN_ID}-{STAGE_SLUG}"
        / tool
    )
    missing_command = tool_command(tool, ["/input/missing.fastq.gz"])
    return {
        "schema_version": "bijux.container.pipeline.v2",
        "run": {
            "id": RUN_ID,
            "kind": "benchmark",
            "campaign": "human-short-read-profile-comparison",
            "corpus": CORPUS_ID,
            "sample": "twenty-run-cohort",
            "domain": "fastq",
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
            "version": runtime["version"],
            "help": runtime["help"],
            "warmup": shell_command(tool, mounted_fastq, input_bytes, warmup=True),
            "real": shell_command(tool, mounted_fastq, input_bytes, warmup=False),
            "negative": missing_command,
        },
        "parameters": parameters,
        "interface": interface_evidence(interface, tool),
        "measurement": {
            "schema_version": "bijux.benchmark.repeated_measurement.v1",
            "warmup_repetitions": 1,
            "measured_repetitions": 3,
            "resource_file_template": (
                "repetitions/repetition-{repetition}.resource.tsv"
            ),
            "runtime_scope": "containerized_stage_end_to_end",
        },
        "outputs": {
            "result_root": str(result_root),
            "mount_path": "/result",
            "required_files": required_files(),
            "collection_rule": "required_files_and_metrics",
        },
        "resources": {
            "account": roots["account"],
            "partition": roots["partition"],
            "cpus": 4,
            "memory_mb": 8192,
            "walltime": "01:00:00",
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
            "threads": 4,
            "random_seed": None,
            "cache_policy": "warm",
            "determinism": "deterministic",
        },
        "dependencies": [],
        "acceptance": {
            "version_regex": runtime["version_regex"],
            "negative_exit_nonzero": True,
            "metrics": [
                {
                    "id": metric,
                    "path": "repetitions/repetition-1.metrics.json",
                    "json_pointer": f"/{metric}",
                    "operator": operator,
                    "expected": expected,
                    "unit": unit,
                }
                for metric, operator, expected, unit in (
                    ("sample_count", "eq", 20, "samples"),
                    ("file_count", "eq", 30, "files"),
                    ("read_count", "gt", 0, "reads"),
                    ("base_count", "gt", 0, "bases"),
                    ("admission_mismatch_count", "eq", 0, "samples"),
                )
            ],
        },
    }


def prepare(
    governance_path: Path,
    data_index_path: Path,
    image_inventory_path: Path,
    container_catalog_path: Path,
    output_root: Path | None,
    submitted_at: str | None = None,
) -> Path:
    roots = governed_roots(load_yaml(governance_path))
    study_root = Path(str(roots["study_root"]))
    repository = Path(str(roots["active_repository"]))
    commit = current_commit(repository)
    campaign_contract(repository)
    interface = interface_contract(repository)
    record = corpus_record(load_json(data_index_path))
    catalog_path, catalog, admitted = admitted_corpus(record, study_root)
    expected_path, provenance_path = materialize_admission_evidence(
        roots, data_index_path, catalog_path, catalog
    )
    inputs, mounted_fastq = data_inputs(
        admitted,
        study_root,
        data_index_path,
        catalog_path,
        expected_path,
        provenance_path,
    )
    input_bytes = sum(int(item["size_bytes"]) for item in admitted)
    images = selected_images(load_json(image_inventory_path), study_root, TOOLS)
    runtimes = runtime_contracts(load_toml(container_catalog_path), TOOLS)
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
    for tool, tool_root in tool_roots.items():
        tool_root.mkdir(parents=True)
        parameters = parameter_evidence(tool_root, repository, catalog)
        (tool_root / "config-pipeline.yml").write_text(
            yaml.safe_dump(
                pipeline(
                    tool,
                    roots,
                    commit,
                    images[tool],
                    inputs,
                    mounted_fastq,
                    input_bytes,
                    timestamp,
                    runtimes[tool],
                    parameters,
                    interface,
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
    root.add_argument("--container-catalog", type=Path, required=True)
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
                args.container_catalog,
                args.output_root,
            )
        )
        return 0
    except (SpecificationError, subprocess.CalledProcessError) as exc:
        print(f"FASTQ profile specification failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
