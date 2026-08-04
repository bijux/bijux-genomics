#!/usr/bin/env python3
"""Validate LUNARC stage-benchmark evidence and render a scientific report."""

from __future__ import annotations

import argparse
import hashlib
import json
import re
import statistics
import sys
from pathlib import Path
from typing import Any

from benchmark_registry import load_toml, threads_by_tool, validate_registry

REPORT_SCHEMA = "bijux.benchmark.stage_report.v1"
EVIDENCE_SCHEMA = "bijux.container.evidence.v2"
ACCOUNTING_SCHEMA = "bijux.slurm.accounting.v1"
SHA256 = re.compile(r"[0-9a-f]{64}")
COMMIT = re.compile(r"[0-9a-f]{40}")
LEGACY_RESOURCE_KEYS = (
    "wall_seconds",
    "user_seconds",
    "system_seconds",
    "peak_rss_kib",
)
RESOURCE_KEYS = LEGACY_RESOURCE_KEYS + (
    "filesystem_input_operations",
    "filesystem_output_operations",
)
RESOURCE_UNITS = {
    "wall_seconds": "seconds",
    "user_seconds": "seconds",
    "system_seconds": "seconds",
    "peak_rss_kib": "kibibytes",
    "filesystem_input_operations": "operations",
    "filesystem_output_operations": "operations",
}


class BenchmarkEvidenceError(RuntimeError):
    """Raised when benchmark evidence is incomplete or internally inconsistent."""


def sha256_file(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as stream:
        for chunk in iter(lambda: stream.read(1024 * 1024), b""):
            digest.update(chunk)
    return digest.hexdigest()


def load_json(path: Path) -> dict[str, Any]:
    try:
        value = json.loads(path.read_text(encoding="utf-8"))
    except (OSError, json.JSONDecodeError) as exc:
        raise BenchmarkEvidenceError(
            f"cannot read benchmark JSON {path}: {exc}"
        ) from exc
    if not isinstance(value, dict):
        raise BenchmarkEvidenceError(f"benchmark JSON must be an object: {path}")
    return value


def read_fastq_histogram(path: Path, sample_id: str) -> dict[int, int]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines or lines[0] != "sample_id\tread_length\tcount":
        raise BenchmarkEvidenceError(f"invalid histogram TSV header: {path}")
    histogram: dict[int, int] = {}
    for line in lines[1:]:
        fields = line.split("\t")
        if len(fields) != 3 or fields[0] != sample_id:
            raise BenchmarkEvidenceError(f"invalid histogram TSV row: {path}")
        try:
            length_value = int(fields[1])
            count_value = int(fields[2])
        except ValueError as exc:
            raise BenchmarkEvidenceError(
                f"non-integer histogram TSV value: {path}"
            ) from exc
        if (
            length_value < 1
            or count_value < 1
            or length_value in histogram
        ):
            raise BenchmarkEvidenceError(f"invalid histogram TSV row: {path}")
        histogram[length_value] = count_value
    if not histogram:
        raise BenchmarkEvidenceError(f"histogram TSV is empty: {path}")
    return histogram


def histogram_summary(histogram: dict[int, int]) -> dict[str, Any]:
    ordered_lengths = sorted(histogram)
    total_reads = sum(histogram.values())
    total_bases = sum(length * count for length, count in histogram.items())
    lower_rank = (total_reads + 1) // 2
    upper_rank = (total_reads + 2) // 2
    cumulative = 0
    lower_value = ordered_lengths[0]
    upper_value = ordered_lengths[-1]
    lower_seen = False
    for length_value in ordered_lengths:
        cumulative += histogram[length_value]
        if not lower_seen and cumulative >= lower_rank:
            lower_value = length_value
            lower_seen = True
        if cumulative >= upper_rank:
            upper_value = length_value
            break
    return {
        "read_count": total_reads,
        "minimum_read_length": ordered_lengths[0],
        "mean_read_length": total_bases / total_reads,
        "median_read_length": (lower_value + upper_value) / 2,
        "maximum_read_length": ordered_lengths[-1],
        "distinct_read_lengths": len(histogram),
    }


def scientific_metrics_from_result_artifact(path: Path) -> dict[str, Any]:
    artifact = load_json(path)
    if (
        artifact.get("schema_version")
        != "bijux.benchmark.fastq.profile_read_lengths.bundle.v1"
        or artifact.get("stage_id") != "fastq.profile_read_lengths"
    ):
        return {}
    results = artifact.get("results")
    if not isinstance(results, list) or not results:
        raise BenchmarkEvidenceError(f"benchmark bundle omits sample results: {path}")
    aggregate_histogram: dict[int, int] = {}
    histogram_digest = hashlib.sha256()
    total_files = 0
    seen_samples: set[str] = set()
    outputs_root = path.parent.parent if path.parent.name == "repetitions" else path.parent
    for row in results:
        if not isinstance(row, dict):
            raise BenchmarkEvidenceError(f"benchmark bundle row must be an object: {path}")
        sample_id = str(row.get("sample_id", ""))
        if not sample_id or sample_id in seen_samples:
            raise BenchmarkEvidenceError(f"benchmark bundle sample id is invalid: {path}")
        seen_samples.add(sample_id)
        histogram_relative = Path(str(row.get("histogram_artifact", "")))
        if (
            histogram_relative.is_absolute()
            or ".." in histogram_relative.parts
            or histogram_relative.parts[:1] != ("repetitions",)
        ):
            raise BenchmarkEvidenceError(f"benchmark histogram path is unsafe: {path}")
        histogram_path = outputs_root / histogram_relative
        recorded_digest = str(row.get("histogram_sha256", ""))
        actual_digest = sha256_file(histogram_path)
        if recorded_digest != actual_digest or SHA256.fullmatch(actual_digest) is None:
            raise BenchmarkEvidenceError(
                f"benchmark histogram digest drifted: {histogram_relative}"
            )
        histogram_digest.update(f"{sample_id}\t{actual_digest}\n".encode("utf-8"))
        histogram = read_fastq_histogram(histogram_path, sample_id)
        summary = histogram_summary(histogram)
        for key in (
            "read_count",
            "minimum_read_length",
            "maximum_read_length",
            "distinct_read_lengths",
        ):
            if row.get(key) != summary[key]:
                raise BenchmarkEvidenceError(
                    f"benchmark bundle {sample_id} disagrees on {key}"
                )
        for key in ("mean_read_length", "median_read_length"):
            if abs(float(row.get(key, -1)) - float(summary[key])) > 1e-12:
                raise BenchmarkEvidenceError(
                    f"benchmark bundle {sample_id} disagrees on {key}"
                )
        total_files += int(row.get("file_count", 0))
        for length_value, count_value in histogram.items():
            aggregate_histogram[length_value] = (
                aggregate_histogram.get(length_value, 0) + count_value
            )
    aggregate = histogram_summary(aggregate_histogram)
    aggregate["file_count"] = total_files
    aggregate["histogram_sha256"] = histogram_digest.hexdigest()
    return aggregate


def repetition_science_metrics(
    repetition_prefix: Path,
    campaign: dict[str, Any],
) -> dict[str, Any]:
    metrics = load_json(repetition_prefix.with_suffix(".metrics.json"))
    artifact_metrics = scientific_metrics_from_result_artifact(
        repetition_prefix.with_name(
            f"{repetition_prefix.name}.{campaign['result_artifact']}"
        )
    )
    merged = dict(metrics)
    for key in campaign["scientific_metrics"]:
        if key in artifact_metrics:
            merged[key] = artifact_metrics[key]
    return merged


def verify_recorded_outputs(result_root: Path, evidence: dict[str, Any]) -> None:
    records = evidence.get("outputs")
    if not isinstance(records, list) or not records:
        raise BenchmarkEvidenceError("benchmark evidence contains no recorded outputs")
    recorded: set[str] = set()
    for record in records:
        if not isinstance(record, dict):
            raise BenchmarkEvidenceError("benchmark output record must be an object")
        relative = Path(str(record.get("path", "")))
        if (
            relative.is_absolute()
            or ".." in relative.parts
            or relative.parts[:1] != ("outputs",)
        ):
            raise BenchmarkEvidenceError(f"benchmark output path is unsafe: {relative}")
        path = result_root / relative
        if not path.is_file():
            raise BenchmarkEvidenceError(
                f"recorded benchmark output is absent: {relative}"
            )
        if path.stat().st_size != record.get("size_bytes"):
            raise BenchmarkEvidenceError(f"benchmark output size drifted: {relative}")
        if sha256_file(path) != record.get("sha256"):
            raise BenchmarkEvidenceError(f"benchmark output digest drifted: {relative}")
        recorded.add(relative.as_posix())
    actual = {
        path.relative_to(result_root).as_posix()
        for path in (result_root / "outputs").rglob("*")
        if path.is_file()
    }
    if recorded != actual:
        raise BenchmarkEvidenceError(
            f"benchmark output inventory is incomplete: missing={sorted(actual - recorded)}, "
            f"extra={sorted(recorded - actual)}"
        )


def read_resources(path: Path) -> dict[str, float]:
    lines = path.read_text(encoding="utf-8").splitlines()
    if not lines:
        raise BenchmarkEvidenceError(f"resource TSV is empty: {path}")
    governed = lines[0] == "metric\tvalue\tunit"
    rows = lines[1:] if governed else lines
    values: dict[str, float] = {}
    for line in rows:
        fields = line.split("\t")
        expected_fields = 3 if governed else 2
        if len(fields) != expected_fields or fields[0] in values:
            raise BenchmarkEvidenceError(f"invalid resource TSV row: {path}")
        if governed and RESOURCE_UNITS.get(fields[0]) != fields[2]:
            raise BenchmarkEvidenceError(f"invalid resource TSV unit: {path}")
        try:
            values[fields[0]] = float(fields[1])
        except ValueError as exc:
            raise BenchmarkEvidenceError(f"non-numeric resource value: {path}") from exc
    expected_keys = RESOURCE_KEYS if governed else LEGACY_RESOURCE_KEYS
    if tuple(values) != expected_keys or any(value < 0 for value in values.values()):
        raise BenchmarkEvidenceError(
            f"resource TSV has an invalid metric surface: {path}"
        )
    return values


def benchmark_context(
    result_root: Path,
    evidence: dict[str, Any],
    campaign: dict[str, Any],
) -> dict[str, Any]:
    """Resolve benchmark context without duplicating the archived run contract."""
    context_path = result_root / "outputs/benchmark-context.json"
    if context_path.is_file():
        return load_json(context_path)
    measurement = evidence.get("measurement")
    execution = evidence.get("execution")
    if not isinstance(measurement, dict) or not isinstance(execution, dict):
        raise BenchmarkEvidenceError("benchmark evidence omits execution context")
    context = {
        "warmup_repetitions": measurement.get("warmup_repetitions"),
        "measured_repetitions": measurement.get("measured_repetitions"),
        "cache_policy": execution.get("cache_policy"),
        "threads": execution.get("threads"),
    }
    input_keys = campaign["input_size_keys"]
    if len(input_keys) != 1:
        raise BenchmarkEvidenceError(
            "multi-input benchmark requires an explicit benchmark context artifact"
        )
    metrics = load_json(result_root / "outputs/repetitions/repetition-1.metrics.json")
    input_bytes = metrics.get("input_bytes")
    if (
        not isinstance(input_bytes, int)
        or isinstance(input_bytes, bool)
        or input_bytes < 1
    ):
        raise BenchmarkEvidenceError("benchmark metrics omit governed input bytes")
    context[input_keys[0]] = input_bytes
    return context


def distribution(values: list[float]) -> dict[str, float]:
    return {
        "mean": round(statistics.mean(values), 6),
        "sample_standard_deviation": round(statistics.stdev(values), 6),
        "minimum": round(min(values), 6),
        "maximum": round(max(values), 6),
    }


def validate_execution_identity(
    result_root: Path,
    evidence: dict[str, Any],
    run_id: str,
    tool: str,
) -> dict[str, Any]:
    if (
        evidence.get("schema_version") != EVIDENCE_SCHEMA
        or evidence.get("evidence_kind") != "benchmark"
        or evidence.get("status") != "passed"
        or evidence.get("run_id") != run_id
    ):
        raise BenchmarkEvidenceError(f"{tool} does not have passed benchmark evidence")
    commit = str(evidence.get("repository_commit", ""))
    if COMMIT.fullmatch(commit) is None:
        raise BenchmarkEvidenceError(f"{tool} benchmark omits a full repository commit")
    image = evidence.get("image")
    if (
        not isinstance(image, dict)
        or SHA256.fullmatch(str(image.get("sha256", ""))) is None
    ):
        raise BenchmarkEvidenceError(
            f"{tool} benchmark omits an immutable image digest"
        )
    checks = {
        "config-pipeline.yml": "pipeline_sha256",
        "config-governance.yml": "governance_sha256",
        "config-credential.yml": "credential_reference_sha256",
    }
    for filename, key in checks.items():
        if sha256_file(result_root / filename) != evidence.get(key):
            raise BenchmarkEvidenceError(
                f"{tool} archived contract digest drifted: {filename}"
            )
    accounting = load_json(result_root / "slurm-accounting.json")
    job_id = str(evidence.get("runtime", {}).get("slurm_job_id", ""))
    if (
        accounting.get("schema_version") != ACCOUNTING_SCHEMA
        or accounting.get("run_id") != run_id
        or accounting.get("slurm_job_id") != job_id
        or not job_id.isdigit()
    ):
        raise BenchmarkEvidenceError(f"{tool} Slurm accounting is inconsistent")
    rows = accounting.get("rows")
    if (
        not isinstance(rows, list)
        or len(rows) != 1
        or rows[0].get("JobID") != job_id
        or rows[0].get("State") != "COMPLETED"
        or rows[0].get("ExitCode") != "0:0"
    ):
        raise BenchmarkEvidenceError(f"{tool} Slurm job did not complete cleanly")
    verify_recorded_outputs(result_root, evidence)
    return accounting


def tool_report(
    campaign: dict[str, Any],
    tool: str,
    result_root: Path,
) -> dict[str, Any]:
    evidence_path = result_root / "evidence.json"
    evidence = load_json(evidence_path)
    accounting = validate_execution_identity(
        result_root, evidence, str(campaign["run_id"]), tool
    )
    context = benchmark_context(result_root, evidence, campaign)
    measured = int(campaign["measured_repetitions"])
    expected_context = {
        "warmup_repetitions": int(campaign["warmup_repetitions"]),
        "measured_repetitions": measured,
        "cache_policy": campaign["cache_policy"],
        "threads": threads_by_tool(campaign)[tool],
    }
    for key, expected in expected_context.items():
        if context.get(key) != expected:
            raise BenchmarkEvidenceError(f"{tool} benchmark context disagrees on {key}")
    input_keys = campaign["input_size_keys"]
    if any(
        not isinstance(context.get(key), int) or context[key] <= 0 for key in input_keys
    ):
        raise BenchmarkEvidenceError(f"{tool} benchmark lacks governed input sizes")

    resources: list[dict[str, float]] = []
    science: list[dict[str, Any]] = []
    result_digests: list[str] = []
    for repetition in range(1, measured + 1):
        root = result_root / "outputs/repetitions"
        resources.append(read_resources(root / f"repetition-{repetition}.resource.tsv"))
        metrics = repetition_science_metrics(
            root / f"repetition-{repetition}", campaign
        )
        missing_metrics = sorted(set(campaign["scientific_metrics"]) - set(metrics))
        if missing_metrics:
            raise BenchmarkEvidenceError(
                f"{tool} repetition {repetition} lacks science metrics: {missing_metrics}"
            )
        if (
            not isinstance(metrics.get("output_bytes"), (int, float))
            or isinstance(metrics["output_bytes"], bool)
            or metrics["output_bytes"] < 0
        ):
            raise BenchmarkEvidenceError(
                f"{tool} repetition {repetition} contains invalid output_bytes"
            )
        for key in campaign["scientific_metrics"]:
            value = metrics[key]
            if key.endswith("_sha256"):
                if not isinstance(value, str) or SHA256.fullmatch(value) is None:
                    raise BenchmarkEvidenceError(
                        f"{tool} repetition {repetition} contains invalid science metrics"
                    )
                continue
            if (
                not isinstance(value, (int, float))
                or isinstance(value, bool)
                or value < 0
            ):
                raise BenchmarkEvidenceError(
                    f"{tool} repetition {repetition} contains invalid science metrics"
                )
        science.append(metrics)
        result_digests.append(
            sha256_file(root / f"repetition-{repetition}.{campaign['result_artifact']}")
        )
    if len(set(result_digests)) != 1:
        raise BenchmarkEvidenceError(
            f"{tool} result artifacts differ across repetitions"
        )

    wall = [row["wall_seconds"] for row in resources]
    cpu = [row["user_seconds"] + row["system_seconds"] for row in resources]
    rss = [row["peak_rss_kib"] for row in resources]
    input_operations = [row.get("filesystem_input_operations") for row in resources]
    output_operations = [row.get("filesystem_output_operations") for row in resources]
    output_bytes = [float(row["output_bytes"]) for row in science]
    scientific_observations = {
        metric: [row[metric] for row in science]
        for metric in campaign["scientific_metrics"]
    }
    inputs = evidence.get("inputs")
    if not isinstance(inputs, list) or not inputs:
        raise BenchmarkEvidenceError(f"{tool} benchmark contains no immutable inputs")
    input_locks = []
    input_ids: set[str] = set()
    for record in inputs:
        if not isinstance(record, dict):
            raise BenchmarkEvidenceError(f"{tool} benchmark input lock is invalid")
        input_id = str(record.get("id", ""))
        digest = str(record.get("sha256", ""))
        if not input_id or input_id in input_ids or SHA256.fullmatch(digest) is None:
            raise BenchmarkEvidenceError(f"{tool} benchmark input identity is invalid")
        input_ids.add(input_id)
        input_locks.append(
            {
                "id": input_id,
                "kind": record.get("kind"),
                "mount_path": record.get("mount_path"),
                "sha256": digest,
            }
        )
    total_input_bytes = sum(context[key] for key in input_keys)
    performance = {
        "wall_seconds": distribution(wall),
        "cpu_seconds": distribution(cpu),
        "cpu_utilization_percent": distribution(
            [cpu_value / wall_value * 100.0 for cpu_value, wall_value in zip(cpu, wall)]
        ),
        "peak_rss_kib": distribution(rss),
        "output_bytes": distribution(output_bytes),
        "throughput_input_bytes_per_second": distribution(
            [total_input_bytes / wall_value for wall_value in wall]
        ),
    }
    if all(value is not None for value in input_operations + output_operations):
        performance["filesystem_input_operations"] = distribution(
            [float(value) for value in input_operations if value is not None]
        )
        performance["filesystem_output_operations"] = distribution(
            [float(value) for value in output_operations if value is not None]
        )
    return {
        "tool": tool,
        "verdict": "passed",
        "repository_commit": evidence["repository_commit"],
        "image": {
            "name": Path(str(evidence["image"]["path"])).name,
            "sha256": evidence["image"]["sha256"],
            "size_bytes": evidence["image"]["size_bytes"],
        },
        "slurm": {
            "job_id": evidence["runtime"]["slurm_job_id"],
            "host": evidence["runtime"]["host"],
            "elapsed": accounting["rows"][0]["Elapsed"],
            "allocated_cpus": evidence["runtime"]["cpus"],
        },
        "source_evidence_sha256": sha256_file(evidence_path),
        "input_locks": input_locks,
        "input_bytes": {
            key.removesuffix("_input_bytes"): context[key] for key in input_keys
        }
        | {
            "total": total_input_bytes,
        },
        "performance": performance,
        "parameters": evidence.get("parameters"),
        "interface": evidence.get("interface"),
        "scientific_observations": scientific_observations,
        "determinism": {
            "artifact": campaign["result_artifact"],
            "sha256": result_digests[0],
            "identical_across_repetitions": True,
        },
    }


def validate_comparability(tools: list[dict[str, Any]]) -> dict[str, Any]:
    """Require all tools to share one immutable scientific comparison context."""
    commits = {tool["repository_commit"] for tool in tools}
    input_locks = {
        json.dumps(tool["input_locks"], sort_keys=True, separators=(",", ":"))
        for tool in tools
    }
    parameter_locks = {
        json.dumps(tool["parameters"], sort_keys=True, separators=(",", ":"))
        for tool in tools
    }
    declared_interfaces = [tool.get("interface") for tool in tools]
    interface_contracts = {
        json.dumps(
            {
                key: tool["interface"].get(key)
                for key in (
                    "accepted_input_contracts",
                    "output_contract",
                    "normalized_result_contract",
                    "downstream_probe",
                )
            },
            sort_keys=True,
            separators=(",", ":"),
        )
        for tool in tools
        if isinstance(tool.get("interface"), dict)
    }
    if len(commits) != 1:
        raise BenchmarkEvidenceError(
            "benchmark tools do not share one repository commit"
        )
    if len(input_locks) != 1:
        raise BenchmarkEvidenceError(
            "benchmark tools do not share one immutable input closure"
        )
    if len(parameter_locks) != 1:
        raise BenchmarkEvidenceError(
            "benchmark tools do not share one parameter closure"
        )
    if any(interface is not None for interface in declared_interfaces) and (
        len(interface_contracts) != 1
        or any(not isinstance(interface, dict) for interface in declared_interfaces)
    ):
        raise BenchmarkEvidenceError("benchmark tools do not share one stage interface")
    closure = {
        "repository_commit": tools[0]["repository_commit"],
        "input_lock_sha256": hashlib.sha256(
            next(iter(input_locks)).encode()
        ).hexdigest(),
        "parameter_lock_sha256": hashlib.sha256(
            next(iter(parameter_locks)).encode()
        ).hexdigest(),
    }
    if interface_contracts:
        interface = tools[0]["interface"]
        closure["stage_interface"] = {
            "accepted_input_contracts": interface["accepted_input_contracts"],
            "output_contract": interface["output_contract"],
            "normalized_result_contract": interface["normalized_result_contract"],
            "downstream_probe": interface["downstream_probe"],
        }
    return closure


def compare_tools(
    campaign: dict[str, Any], tools: list[dict[str, Any]]
) -> dict[str, Any]:
    comparisons = []
    for contract in campaign["comparison_metrics"]:
        metric = contract["id"]
        observations = {
            tool["tool"]: statistics.mean(tool["scientific_observations"][metric])
            for tool in tools
        }
        minimum = min(observations.values())
        maximum = max(observations.values())
        absolute_range = maximum - minimum
        scale = max(abs(value) for value in observations.values())
        allowed_range = (
            float(contract["absolute_tolerance"])
            + float(contract["relative_tolerance"]) * scale
        )
        comparisons.append(
            {
                "metric": metric,
                "observations": observations,
                "minimum": minimum,
                "maximum": maximum,
                "absolute_range": absolute_range,
                "allowed_range": allowed_range,
                "within_tolerance": absolute_range <= allowed_range,
                "absolute_tolerance": contract["absolute_tolerance"],
                "relative_tolerance": contract["relative_tolerance"],
            }
        )
    return {
        "status": (
            "not_applicable"
            if not comparisons
            else (
                "within_tolerance"
                if all(row["within_tolerance"] for row in comparisons)
                else "outside_tolerance"
            )
        ),
        "metrics": comparisons,
    }


def comparison_verdict(comparison: dict[str, Any]) -> str:
    """Admit only complete comparisons that satisfy their declared tolerances."""
    status = comparison.get("status")
    if status in {"not_applicable", "within_tolerance"}:
        return "passed"
    if status == "outside_tolerance":
        return "failed"
    raise BenchmarkEvidenceError(f"benchmark comparison status is invalid: {status}")


def create_report(
    registry_path: Path,
    catalog_path: Path,
    run_id: str,
    stage_result_root: Path,
) -> dict[str, Any]:
    campaigns = validate_registry(load_toml(registry_path), load_toml(catalog_path))
    matches = [campaign for campaign in campaigns if campaign["run_id"] == run_id]
    if len(matches) != 1:
        raise BenchmarkEvidenceError(f"benchmark registry does not define {run_id}")
    campaign = matches[0]
    declared = set(campaign["tool_ids"])
    observed = {path.name for path in stage_result_root.iterdir() if path.is_dir()}
    if observed != declared:
        raise BenchmarkEvidenceError(
            f"benchmark evidence lacks complete tool closure: "
            f"missing={sorted(declared - observed)}, extra={sorted(observed - declared)}"
        )
    tools = [
        tool_report(campaign, tool, stage_result_root / tool)
        for tool in campaign["tool_ids"]
    ]
    comparability = validate_comparability(tools)
    for tool in tools:
        tool.pop("input_locks")
        tool.pop("parameters")
        tool.pop("interface")
    comparison = compare_tools(campaign, tools)
    return {
        "schema_version": REPORT_SCHEMA,
        "verdict": comparison_verdict(comparison),
        "run_id": run_id,
        "stage_id": campaign["stage_id"],
        "evidence_tier": campaign["evidence_tier"],
        "benchmark_profile_id": campaign["profile_id"],
        "scientific_question": campaign["scientific_question"],
        "comparison_scope": campaign["comparison_scope"],
        "accuracy_claim": campaign["accuracy_claim"],
        "design": {
            "runtime": campaign["runtime"],
            "scheduler": campaign["scheduler"],
            "container_only": campaign["container_only"],
            "threads_by_tool": threads_by_tool(campaign),
            "cache_policy": campaign["cache_policy"],
            "warmup_repetitions": campaign["warmup_repetitions"],
            "measured_repetitions": campaign["measured_repetitions"],
            "tool_closure": campaign["tool_ids"],
            "corpus_ids": campaign["corpus_ids"],
            "corpus_catalog_ids": campaign["corpus_catalog_ids"],
            "reference_catalog_ids": campaign["reference_catalog_ids"],
            "database_catalog_ids": campaign["database_catalog_ids"],
            "input_size_keys": campaign["input_size_keys"],
            "result_artifact": campaign["result_artifact"],
        },
        "tools": tools,
        "comparability": comparability,
        "cross_tool_comparison": comparison,
        "limitations": campaign["limitations"],
        "interpretation": campaign["interpretation"],
    }


def render_markdown(report: dict[str, Any]) -> str:
    design = report["design"]
    lines = [
        "<!-- GENERATED FILE - DO NOT EDIT -->",
        "<!-- source: benchmarks/benchmark_evidence.py -->",
        f"# {report['stage_id']} {report['evidence_tier']}",
        "",
        f"**Verdict: {report['verdict']}.** {report['interpretation']}",
        "",
        (
            "This is qualification evidence. It proves the declared execution and "
            "metric contract on the named data, but it is not a comparative scientific "
            "benchmark and must not be used to rank tools."
            if report["evidence_tier"] == "qualification"
            else "This is comparative evidence governed by the declared scientific profile."
        ),
        "",
        "## Scientific scope",
        "",
        report["scientific_question"],
        "",
        f"Comparison scope: {report['comparison_scope']}.",
        "",
        f"Accuracy claim: `{report['accuracy_claim']}`.",
        "",
        "## Design",
        "",
        f"- Campaign: `{report['run_id']}`",
        f"- Evidence tier: `{report['evidence_tier']}`",
        f"- Scientific profile: `{report['benchmark_profile_id']}`",
        f"- Tool closure: {', '.join(f'`{tool}`' for tool in design['tool_closure'])}",
        f"- Execution: {design['runtime']} under {design['scheduler']}, container-only",
        "- Threads by tool: "
        + ", ".join(
            f"`{tool}`={threads}" for tool, threads in design["threads_by_tool"].items()
        ),
        f"- Cache policy: {design['cache_policy']}",
        f"- Repetitions: {design['warmup_repetitions']} warmup and {design['measured_repetitions']} measured",
        f"- Corpus: {', '.join(f'`{item}`' for item in design['corpus_catalog_ids'])}",
        f"- Corpus portfolio: {', '.join(f'`{item}`' for item in design['corpus_ids'])}",
        "- Reference: "
        + (
            ", ".join(f"`{item}`" for item in design["reference_catalog_ids"])
            if design["reference_catalog_ids"]
            else "none"
        ),
        "- Databases: "
        + (
            ", ".join(f"`{item}`" for item in design["database_catalog_ids"])
            if design["database_catalog_ids"]
            else "none"
        ),
        f"- Repeated result artifact: `{design['result_artifact']}`",
        "",
    ]
    comparability = report.get("comparability")
    if isinstance(comparability, dict):
        lines.extend(
            [
                "## Replay and comparison interface",
                "",
                f"- Repository commit: `{comparability['repository_commit']}`",
                f"- Immutable input closure: `{comparability['input_lock_sha256']}`",
                f"- Effective parameter closure: `{comparability['parameter_lock_sha256']}`",
            ]
        )
        stage_interface = comparability.get("stage_interface")
        if isinstance(stage_interface, dict):
            probe = stage_interface["downstream_probe"]
            lines.extend(
                [
                    "- Accepted inputs: "
                    + ", ".join(
                        f"`{contract}`"
                        for contract in stage_interface["accepted_input_contracts"]
                    ),
                    f"- Normalized output: `{stage_interface['output_contract']}`",
                    f"- Output schema: `{stage_interface['normalized_result_contract']}`",
                    f"- Tool-independent consumer: `{probe['stage_id']}`",
                    "- Substitution scope: normalized benchmark comparison only; this run does not claim operational downstream pipeline substitution",
                ]
            )
        lines.append("")
    lines.extend(
        [
            "## Performance",
            "",
            "| Tool | Wall mean (range), s | CPU mean, s | CPU use, % | Peak RSS mean (range), MiB | Throughput, MiB/s | Filesystem input/output operations | Output mean, bytes |",
            "|---|---:|---:|---:|---:|---:|---:|---:|",
        ]
    )
    for tool in report["tools"]:
        performance = tool["performance"]
        wall = performance["wall_seconds"]
        cpu = performance["cpu_seconds"]
        rss = performance["peak_rss_kib"]
        output = performance["output_bytes"]
        utilization = performance.get("cpu_utilization_percent")
        throughput = performance.get("throughput_input_bytes_per_second")
        input_operations = performance.get("filesystem_input_operations")
        output_operations = performance.get("filesystem_output_operations")
        utilization_value = (
            f"{utilization['mean']:.1f}" if utilization else "not recorded"
        )
        throughput_value = (
            f"{throughput['mean'] / 1048576:.2f}" if throughput else "not recorded"
        )
        operations_value = (
            f"{input_operations['mean']:.1f}/{output_operations['mean']:.1f}"
            if input_operations and output_operations
            else "not recorded"
        )
        lines.append(
            f"| `{tool['tool']}` | {wall['mean']:.3f} ({wall['minimum']:.3f}–{wall['maximum']:.3f}) "
            f"| {cpu['mean']:.3f} | {utilization_value} | {rss['mean'] / 1024:.1f} "
            f"({rss['minimum'] / 1024:.1f}–{rss['maximum'] / 1024:.1f}) "
            f"| {throughput_value} | {operations_value} | {output['mean']:.1f} |"
        )
    lines.extend(["", "## Scientific observations", ""])
    for tool in report["tools"]:
        lines.extend(
            [
                f"### {tool['tool']}",
                "",
                "| Metric | Measured repetitions |",
                "|---|---|",
            ]
        )
        for metric, values in tool["scientific_observations"].items():
            lines.append(
                f"| `{metric}` | {', '.join(str(value) for value in values)} |"
            )
        lines.extend(
            [
                "",
                f"The `{tool['determinism']['artifact']}` result was byte-identical across all repetitions: `{tool['determinism']['sha256']}`.",
                "",
                f"Evidence: Slurm job `{tool['slurm']['job_id']}`, repository `{tool['repository_commit']}`, image `{tool['image']['sha256']}`.",
                "",
            ]
        )
    comparison = report["cross_tool_comparison"]
    lines.extend(["## Cross-tool agreement", ""])
    if comparison["status"] == "not_applicable":
        lines.extend(
            [
                "This campaign has one tool, so cross-tool agreement is not applicable.",
                "",
            ]
        )
    else:
        lines.extend(
            [
                f"Agreement status: **{comparison['status'].replace('_', ' ')}**.",
                "",
                "| Metric | Tool means | Observed range | Allowed range | Status |",
                "|---|---|---:|---:|---|",
            ]
        )
        for row in comparison["metrics"]:
            observations = ", ".join(
                f"`{tool}`={value:.12g}" for tool, value in row["observations"].items()
            )
            lines.append(
                f"| `{row['metric']}` | {observations} | {row['absolute_range']:.12g} "
                f"| {row['allowed_range']:.12g} | "
                f"{'within' if row['within_tolerance'] else 'outside'} |"
            )
        lines.append("")
    lines.extend(["## Limitations", ""])
    lines.extend(f"- {limitation}" for limitation in report["limitations"])
    return "\n".join(lines) + "\n"


def parser() -> argparse.ArgumentParser:
    root = argparse.ArgumentParser(description=__doc__)
    root.add_argument("--registry", type=Path, required=True)
    root.add_argument("--catalog", type=Path, required=True)
    root.add_argument("--run-id", required=True)
    root.add_argument("--stage-result-root", type=Path, required=True)
    root.add_argument("--json-output", type=Path, required=True)
    root.add_argument("--markdown-output", type=Path, required=True)
    return root


def main() -> int:
    args = parser().parse_args()
    try:
        report = create_report(
            args.registry, args.catalog, args.run_id, args.stage_result_root
        )
        args.json_output.parent.mkdir(parents=True, exist_ok=True)
        args.markdown_output.parent.mkdir(parents=True, exist_ok=True)
        args.json_output.write_text(
            json.dumps(report, indent=2, sort_keys=True) + "\n", encoding="utf-8"
        )
        args.markdown_output.write_text(render_markdown(report), encoding="utf-8")
        print(f"stage benchmark evidence: {args.run_id} passed")
        return 0
    except (BenchmarkEvidenceError, OSError) as exc:
        print(f"stage benchmark evidence failed: {exc}", file=sys.stderr)
        return 2


if __name__ == "__main__":
    raise SystemExit(main())
