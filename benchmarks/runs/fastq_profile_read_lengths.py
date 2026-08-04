"""Prepare the exact FASTQ read-length benchmark campaign."""

from __future__ import annotations

import argparse
import datetime as dt
import gzip
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

RUN_ID = "run-2042"
STAGE_ID = "fastq.profile_read_lengths"
STAGE_SLUG = "fastq-profile-read-lengths"
CORPUS_ID = "human-short-read-diversity"
TOOLS = ("seqfu", "seqkit")
THREADS_BY_TOOL = {"seqfu": 1, "seqkit": 4}
PARAMETER_SCHEMA = "bijux.fastq.params.read_length_profile.v1"
DEFAULTS_PROVIDER = (
    "bijux_dna_domain_fastq::params::defaults::read_length_profile_defaults"
)
MANIFEST_HEADER = (
    "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\t"
    "mean_read_length\tmedian_read_length\tmaximum_read_length\t"
    "distinct_read_lengths\thistogram_artifact\thistogram_sha256"
)
SUMMARY_HEADER = (
    "sample_id\tinput_layout\tfile_count\tread_count\tminimum_read_length\t"
    "mean_read_length\tmedian_read_length\tmaximum_read_length\t"
    "distinct_read_lengths"
)


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
        "threads_by_tool": THREADS_BY_TOOL,
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


def open_fastq(path: Path):
    if path.suffix == ".gz":
        return gzip.open(path, "rt", encoding="utf-8")
    return path.open("r", encoding="utf-8")


def sample_inputs(catalog: dict[str, Any], admitted: list[dict[str, Any]]) -> list[dict[str, Any]]:
    files_by_sample: dict[str, list[dict[str, Any]]] = {}
    for item in admitted:
        accession = str(item.get("accession", ""))
        files_by_sample.setdefault(accession, []).append(item)
    rows = []
    for sample in sorted(catalog["samples"], key=lambda value: value["accession"]):
        accession = str(sample["accession"])
        layout = "paired_end" if sample.get("layout") == "pe" else "single_end"
        files = files_by_sample.get(accession, [])
        expected_file_count = 2 if layout == "paired_end" else 1
        if len(files) != expected_file_count:
            raise SpecificationError(
                f"{accession} file closure disagrees with its layout"
            )
        rows.append(
            {
                "sample_id": accession,
                "input_layout": layout,
                "files": files,
            }
        )
    return rows


def independent_sample_evidence(sample: dict[str, Any], study_root: Path) -> dict[str, Any]:
    histogram: dict[int, int] = {}
    total = 0
    observations = 0
    digest = hashlib.sha256()
    for item in sample["files"]:
        path = study_path(str(item["governed_path"]), study_root)
        with open_fastq(path) as stream:
            while True:
                header = stream.readline()
                if not header:
                    break
                sequence = stream.readline()
                plus = stream.readline()
                quality = stream.readline()
                if not sequence or not plus or not quality:
                    raise SpecificationError(f"FASTQ record is truncated: {path}")
                if not header.startswith("@") or not plus.startswith("+"):
                    raise SpecificationError(f"FASTQ framing is invalid: {path}")
                record_id = header[1:].strip().split()[0]
                length_value = len(sequence.rstrip("\n\r"))
                digest.update(f"{record_id}\t{length_value}\n".encode("utf-8"))
                histogram[length_value] = histogram.get(length_value, 0) + 1
                observations += 1
                total += length_value
    if not histogram:
        raise SpecificationError(
            f"independent exact read-length evidence is empty: {sample['sample_id']}"
        )
    ordered_lengths = sorted(histogram)
    lower_rank = (observations + 1) // 2
    upper_rank = (observations + 2) // 2
    cumulative = 0
    lower_value = ordered_lengths[0]
    upper_value = ordered_lengths[-1]
    lower_seen = False
    upper_seen = False
    for length_value in ordered_lengths:
        cumulative += histogram[length_value]
        if not lower_seen and cumulative >= lower_rank:
            lower_value = length_value
            lower_seen = True
        if not upper_seen and cumulative >= upper_rank:
            upper_value = length_value
            upper_seen = True
            break
    median = (lower_value + upper_value) / 2
    summary_line = (
        f"{sample['sample_id']}\t{sample['input_layout']}\t{len(sample['files'])}\t"
        f"{observations}\t{ordered_lengths[0]}\t{total / observations:.12g}\t"
        f"{median:.12g}\t{ordered_lengths[-1]}\t{len(histogram)}"
    )
    histogram_lines = ["sample_id\tread_length\tcount"] + [
        f"{sample['sample_id']}\t{length_value}\t{histogram[length_value]}"
        for length_value in ordered_lengths
    ]
    return {
        "sample_id": sample["sample_id"],
        "input_layout": sample["input_layout"],
        "file_count": len(sample["files"]),
        "read_count": observations,
        "minimum_read_length": ordered_lengths[0],
        "mean_read_length": total / observations,
        "median_read_length": median,
        "maximum_read_length": ordered_lengths[-1],
        "distinct_read_lengths": len(histogram),
        "summary_line": summary_line,
        "histogram_lines": histogram_lines,
        "exact_rows_sha256": digest.hexdigest(),
    }


def materialize_expected_evidence(
    roots: dict[str, Path | str],
    data_index_path: Path,
    catalog_path: Path,
    catalog: dict[str, Any],
    admitted: list[dict[str, Any]],
) -> tuple[Path, list[dict[str, Any]]]:
    identity = sha256_file(catalog_path)
    root = (
        Path(str(roots["data_root"]))
        / "evidence/benchmark-expectations"
        / identity
        / "fastq-profile-read-lengths"
    )
    summaries_root = root / "summaries"
    histograms_root = root / "histograms"
    summaries_root.mkdir(parents=True, exist_ok=True)
    histograms_root.mkdir(parents=True, exist_ok=True)
    samples = sample_inputs(catalog, admitted)
    study_root = Path(str(roots["study_root"]))
    manifest_rows = []
    for sample in samples:
        evidence = independent_sample_evidence(sample, study_root)
        summary_path = summaries_root / f"{sample['sample_id']}.summary.tsv"
        histogram_path = histograms_root / f"{sample['sample_id']}.histogram.tsv"
        summary_path.write_text(
            SUMMARY_HEADER + "\n" + evidence["summary_line"] + "\n",
            encoding="utf-8",
        )
        histogram_path.write_text(
            "\n".join(evidence["histogram_lines"]) + "\n", encoding="utf-8"
        )
        manifest_rows.append(
            {
                "sample_id": sample["sample_id"],
                "input_layout": sample["input_layout"],
                "file_count": evidence["file_count"],
                "read_count": evidence["read_count"],
                "minimum_read_length": evidence["minimum_read_length"],
                "mean_read_length": evidence["mean_read_length"],
                "median_read_length": evidence["median_read_length"],
                "maximum_read_length": evidence["maximum_read_length"],
                "distinct_read_lengths": evidence["distinct_read_lengths"],
                "summary_path": summary_path,
                "histogram_path": histogram_path,
                "histogram_sha256": sha256_file(histogram_path),
                "exact_rows_sha256": evidence["exact_rows_sha256"],
            }
        )
    manifest_path = root / "manifest.json"
    manifest = {
        "schema_version": "bijux.benchmark.fastq.profile_read_lengths.expectations.v1",
        "corpus_id": CORPUS_ID,
        "sample_count": len(manifest_rows),
        "file_count": len(admitted),
        "data_index_sha256": sha256_file(data_index_path),
        "catalog_sha256": identity,
        "samples": [
            {
                "sample_id": row["sample_id"],
                "input_layout": row["input_layout"],
                "file_count": row["file_count"],
                "read_count": row["read_count"],
                "minimum_read_length": row["minimum_read_length"],
                "mean_read_length": row["mean_read_length"],
                "median_read_length": row["median_read_length"],
                "maximum_read_length": row["maximum_read_length"],
                "distinct_read_lengths": row["distinct_read_lengths"],
                "summary_sha256": sha256_file(Path(row["summary_path"])),
                "histogram_sha256": row["histogram_sha256"],
                "exact_rows_sha256": row["exact_rows_sha256"],
            }
            for row in manifest_rows
        ],
    }
    manifest_path.write_text(
        json.dumps(manifest, indent=2, sort_keys=True) + "\n", encoding="utf-8"
    )
    return manifest_path, manifest_rows


def data_inputs(
    admitted: list[dict[str, Any]],
    study_root: Path,
    data_index_path: Path,
    catalog_path: Path,
    manifest_path: Path,
    manifest_rows: list[dict[str, Any]],
) -> tuple[list[dict[str, str]], list[dict[str, Any]]]:
    inputs: list[dict[str, str]] = []
    sample_mounts: dict[str, list[str]] = {}
    for item in admitted:
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
        sample_mounts.setdefault(str(item["accession"]), []).append(mount_path)
    support = (
        ("corpus_data_index", data_index_path, "/input/data-index.json"),
        ("corpus_catalog", catalog_path, "/input/corpus-catalog.json"),
        ("read_length_manifest", manifest_path, "/input/read-length-manifest.json"),
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
    samples = []
    for row in manifest_rows:
        summary_mount = f"/input/{row['sample_id']}.summary.tsv"
        histogram_mount = f"/input/{row['sample_id']}.histogram.tsv"
        for source, mount_path, label in (
            (Path(row["summary_path"]), summary_mount, "summary"),
            (Path(row["histogram_path"]), histogram_mount, "histogram"),
        ):
            inputs.append(
                {
                    "id": f"{row['sample_id'].lower()}_{label}",
                    "kind": "data",
                    "path": str(source.resolve()),
                    "mount_path": mount_path,
                    "sha256": sha256_file(source),
                }
            )
        samples.append(
            {
                "sample_id": row["sample_id"],
                "input_layout": row["input_layout"],
                "mounted_fastq": sample_mounts[row["sample_id"]],
                "expected_summary_mount": summary_mount,
                "expected_histogram_mount": histogram_mount,
                "expected_exact_sha256": row["exact_rows_sha256"],
            }
        )
    return inputs, samples


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
        or stage.get("overrides") != {"threads": 4, "histogram_bins": 100}
    ):
        raise SpecificationError(
            "parameter config differs from crate-owned defaults policy"
        )
    overrides_path = tool_root / "config-pipeline-parameters.yml"
    overrides_path.write_bytes(source.read_bytes())
    resolved = {
        sample["accession"]: {
            "schema_version": PARAMETER_SCHEMA,
            "paired_mode": "paired_end" if sample.get("layout") == "pe" else "single_end",
            "threads": 4,
            "histogram_bins": 100,
        }
        for sample in catalog["samples"]
    }
    effective_path = tool_root / "effective-parameters.json"
    effective = {
        "schema_version": "bijux.benchmark.effective_parameters.v1",
        "stage_id": STAGE_ID,
        "parameter_schema_version": PARAMETER_SCHEMA,
        "defaults_provider": DEFAULTS_PROVIDER,
        "overrides": {"threads": 4, "histogram_bins": 100},
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


def tool_command(tool: str, mounted_fastq: list[str]) -> str:
    if tool == "seqfu":
        commands = [
            "test -s "
            + shlex.quote(path)
            + " && seqfu tab "
            + shlex.quote(path)
            + " | awk -F '\\t' '{print $1 \"\\t\" length($3)}'"
            for path in mounted_fastq
        ]
        return "( " + "; ".join(commands) + " )"
    quoted = " ".join(shlex.quote(path) for path in mounted_fastq)
    return (
        "test -s "
        + " && test -s ".join(shlex.quote(path) for path in mounted_fastq)
        + " && seqkit fx2tab --name --only-id --length --threads 4 "
        + quoted
        + " | awk -F '\\t' 'NF < 2 { exit 2 } { print $1 \"\\t\" $NF }'"
    )


def shell_command(
    tool: str,
    samples: list[dict[str, Any]],
    input_bytes: int,
    *,
    warmup: bool,
) -> list[str]:
    lines = ["set -eu"]
    if warmup:
        for sample in samples:
            lines.append(tool_command(tool, sample["mounted_fastq"]) + " > /dev/null")
        return ["sh", "-c", "; ".join(lines)]

    lines.extend(
        [
            "mkdir -p /result/repetitions",
            'prefix="/result/repetitions/repetition-{repetition}"',
            'manifest="${prefix}.manifest.tsv"',
            f"printf '%s\\n' {shlex.quote(MANIFEST_HEADER)} > \"$manifest\"",
        ]
    )
    output_paths = ['"$manifest"']
    for sample in samples:
        sample_id = sample["sample_id"]
        raw = f'"${{prefix}}.{sample_id}.raw.tsv"'
        summary = f'"${{prefix}}.{sample_id}.summary.tsv"'
        histogram = f'"${{prefix}}.{sample_id}.histogram.tsv"'
        artifact = f"repetitions/repetition-{{repetition}}.{sample_id}.histogram.tsv"
        lines.extend(
            [
                f"{tool_command(tool, sample['mounted_fastq'])} > {raw}",
                f"observed_exact_sha=$(sha256sum {raw} | awk '{{print $1}}')",
                f"[ \"$observed_exact_sha\" = \"{sample['expected_exact_sha256']}\" ]",
                " ".join(
                    [
                        "awk",
                        "-v",
                        "mode=summary",
                        "-v",
                        f"sample_id={shlex.quote(sample_id)}",
                        "-v",
                        f"input_layout={sample['input_layout']}",
                        "-f",
                        "/repository/benchmarks/execution/fastq_read_lengths.awk",
                        raw,
                        ">",
                        summary,
                    ]
                ),
                f"diff -u {shlex.quote(sample['expected_summary_mount'])} {summary}",
                " ".join(
                    [
                        "awk",
                        "-v",
                        "mode=histogram",
                        "-v",
                        f"sample_id={shlex.quote(sample_id)}",
                        "-v",
                        f"input_layout={sample['input_layout']}",
                        "-f",
                        "/repository/benchmarks/execution/fastq_read_lengths.awk",
                        raw,
                        ">",
                        histogram,
                    ]
                ),
                f"diff -u {shlex.quote(sample['expected_histogram_mount'])} {histogram}",
                f"observed_histogram_sha=$(sha256sum {histogram} | awk '{{print $1}}')",
                f'tail -n +2 {summary} | awk -v OFS=\'\\t\' -v artifact="{artifact}" -v sha="$observed_histogram_sha" \'{{print $0, artifact, sha}}\' >> "$manifest"',
            ]
        )
        output_paths.extend([raw, summary, histogram])
    lines.extend(
        [
            " ".join(
                [
                    "awk",
                    "-v",
                    f"tool={tool}",
                    "-v",
                    "mode=bundle",
                    "-f",
                    "/repository/benchmarks/execution/fastq_read_lengths.awk",
                    '"$manifest"',
                    ">",
                    '"${prefix}.read-lengths.json"',
                ]
            ),
            " ".join(
                [
                    "awk",
                    "-v",
                    "mode=probe",
                    "-f",
                    "/repository/benchmarks/execution/fastq_read_lengths.awk",
                    '"$manifest"',
                    ">",
                    '"${prefix}.report-qc.json"',
                ]
            ),
            f"output_bytes=$(wc -c {' '.join(output_paths)} \"${{prefix}}.read-lengths.json\" \"${{prefix}}.report-qc.json\" | awk 'END {{print $1}}')",
            " ".join(
                [
                    "awk",
                    "-v",
                    "mode=metrics",
                    "-v",
                    f"input_bytes={input_bytes}",
                    "-v",
                    'output_bytes="$output_bytes"',
                    "-f",
                    "/repository/benchmarks/execution/fastq_read_lengths.awk",
                    '"$manifest"',
                    ">",
                    '"${prefix}.metrics.json"',
                ]
            ),
        ]
    )
    return ["sh", "-c", "\n".join(lines)]


def required_files(samples: list[dict[str, Any]]) -> list[str]:
    files: list[str] = []
    for repetition in range(1, 4):
        prefix = f"repetitions/repetition-{repetition}"
        files.append(f"{prefix}.manifest.tsv")
        for sample in samples:
            files.extend(
                (
                    f"{prefix}.{sample['sample_id']}.raw.tsv",
                    f"{prefix}.{sample['sample_id']}.summary.tsv",
                    f"{prefix}.{sample['sample_id']}.histogram.tsv",
                )
            )
        files.extend(
            (
                f"{prefix}.read-lengths.json",
                f"{prefix}.report-qc.json",
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
    samples: list[dict[str, Any]],
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
    negative_command = tool_command(tool, ["/input/missing.fastq.gz"])
    threads = THREADS_BY_TOOL[tool]
    return {
        "schema_version": "bijux.container.pipeline.v2",
        "run": {
            "id": RUN_ID,
            "kind": "benchmark",
            "campaign": "human-short-read-length-comparison",
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
            "warmup": shell_command(tool, samples, input_bytes, warmup=True),
            "real": shell_command(tool, samples, input_bytes, warmup=False),
            "negative": ["sh", "-c", negative_command],
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
            "required_files": required_files(samples),
            "collection_rule": "required_files_and_metrics",
        },
        "resources": {
            "account": roots["account"],
            "partition": roots["partition"],
            "cpus": threads,
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
            "threads": threads,
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
                    ("exact_observation_mismatch_count", "eq", 0, "samples"),
                    ("histogram_mismatch_count", "eq", 0, "samples"),
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
    manifest_path, manifest_rows = materialize_expected_evidence(
        roots, data_index_path, catalog_path, catalog, admitted
    )
    inputs, samples = data_inputs(
        admitted,
        study_root,
        data_index_path,
        catalog_path,
        manifest_path,
        manifest_rows,
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
                    samples,
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
        print(f"FASTQ read-length specification failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
