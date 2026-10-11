//! Semantic validation for executable benchmark specifications.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use chrono::{DateTime, FixedOffset};

use crate::diagnostics::BenchError;
use crate::model::{
    BenchmarkAcceptanceExpectedValue, BenchmarkAcceptanceMetric, BenchmarkExecutionCommands,
    BenchmarkExecutionSpec,
};

use super::{validate_attempt_identity, BENCHMARK_EXECUTION_SPEC_SCHEMA_V1};

const REQUIRED_SOURCE_LOCKS: &[&str] =
    &["campaign_registry", "container_catalog", "image_inventory", "stage_interface_registry"];
const DISPOSABLE_PRODUCTION_ROOTS: &[&str] = &[
    "assets/toy",
    "benchmarks/tests/fixtures",
    "configs/bench/local",
    "runs/bench/local-fake-runs",
    "runs/bench/readiness-probes",
    "artifacts/benchmark-smoke",
];

/// Validate a complete immutable executable benchmark specification.
///
/// # Errors
/// Returns an error when identity, source locks, paths, commands, resources, or acceptance rules
/// are incomplete or internally inconsistent.
pub fn validate_execution_spec(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    if spec.schema_version != BENCHMARK_EXECUTION_SPEC_SCHEMA_V1 {
        return Err(invalid("benchmark execution specification schema mismatch"));
    }
    validate_attempt_identity(&spec.identity)?;
    require_text(&spec.operator, "operator")?;
    parse_timestamp(&spec.prepared_at_utc)?;
    validate_source(spec)?;
    validate_image(spec)?;
    validate_inputs(spec)?;
    validate_commands(&spec.commands)?;
    validate_parameters(spec)?;
    validate_interface(spec)?;
    validate_measurement(spec)?;
    validate_outputs(spec)?;
    validate_resources(spec)?;
    validate_runtime(spec)?;
    validate_acceptance(spec)
}

fn validate_source(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    validate_commit(&spec.source.repository_commit)?;
    require_absolute_path(&spec.source.repository_path, "repository path")?;
    for required in REQUIRED_SOURCE_LOCKS {
        if !spec.source.contract_sha256.contains_key(*required) {
            return Err(invalid("benchmark execution source lock is incomplete"));
        }
    }
    if spec.source.contract_sha256.is_empty() {
        return Err(invalid("benchmark execution has no contract source locks"));
    }
    for (name, digest) in &spec.source.contract_sha256 {
        validate_component(name, "source lock name")?;
        validate_digest(digest, "source lock digest")?;
    }
    Ok(())
}

fn validate_image(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    require_absolute_path(&spec.image.path, "image path")?;
    validate_digest(&spec.image.sha256, "image digest")
}

fn validate_inputs(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    if spec.inputs.is_empty() {
        return Err(invalid("benchmark execution has no inputs"));
    }
    let mut ids = BTreeSet::new();
    let mut mounts = BTreeSet::new();
    for input in &spec.inputs {
        validate_component(&input.id, "input id")?;
        if !ids.insert(input.id.as_str()) {
            return Err(invalid("benchmark execution duplicates an input id"));
        }
        require_absolute_path(&input.path, "input path")?;
        if !input.mount_path.starts_with("/input/")
            || input.mount_path.ends_with('/')
            || !mounts.insert(input.mount_path.as_str())
            || input.mount_path.split('/').any(|part| part == "." || part == "..")
        {
            return Err(invalid("benchmark execution input mount path is invalid or duplicated"));
        }
        validate_digest(&input.sha256, "input digest")?;
    }
    Ok(())
}

fn validate_commands(commands: &BenchmarkExecutionCommands) -> Result<(), BenchError> {
    for (label, command) in [
        ("version", &commands.version),
        ("help", &commands.help),
        ("warmup", &commands.warmup),
        ("measured", &commands.measured),
        ("negative", &commands.negative),
    ] {
        if command.is_empty() || command.iter().any(String::is_empty) {
            return Err(invalid(&format!("benchmark execution {label} command is empty")));
        }
        for argument in command {
            reject_disposable_production_root(argument, &format!("{label} command"))?;
        }
    }
    Ok(())
}

fn validate_parameters(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    require_text(&spec.parameters.parameter_schema_version, "parameter schema version")?;
    require_text(&spec.parameters.defaults_provider, "parameter defaults provider")?;
    validate_relative_path(&spec.parameters.overrides_path, "parameter overrides path")?;
    validate_digest(&spec.parameters.overrides_sha256, "parameter overrides digest")?;
    validate_relative_path(&spec.parameters.effective_path, "effective parameter path")?;
    validate_digest(&spec.parameters.effective_sha256, "effective parameter digest")
}

fn validate_interface(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    validate_text_list(&spec.interface.accepted_input_contracts, "accepted input contracts")?;
    require_text(&spec.interface.output_contract, "output contract")?;
    validate_relative_path(
        &spec.interface.normalized_result_contract,
        "normalized result contract",
    )?;
    require_text(&spec.interface.adapter, "tool adapter")?;
    require_text(&spec.interface.downstream_probe.stage_id, "downstream probe stage")?;
    require_text(&spec.interface.downstream_probe.input_contract, "downstream input contract")?;
    validate_relative_path(&spec.interface.downstream_probe.source, "downstream probe source")?;
    validate_relative_path(&spec.interface.downstream_probe.artifact, "downstream probe artifact")
}

fn validate_measurement(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    if spec.measurement.warmup_repetitions < 1 || spec.measurement.measured_repetitions < 3 {
        return Err(invalid("benchmark execution repetition policy is insufficient"));
    }
    if !spec.measurement.resource_file_template.contains("{repetition}") {
        return Err(invalid("benchmark resource template lacks repetition identity"));
    }
    validate_relative_path(&spec.measurement.resource_file_template, "resource file template")?;
    if spec.measurement.runtime_scope != "containerized_stage_end_to_end" {
        return Err(invalid("benchmark runtime measurement scope is unsupported"));
    }
    Ok(())
}

fn validate_outputs(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    require_absolute_path(&spec.outputs.result_root, "result root")?;
    let expected_suffix = format!(
        "/{}/{}/attempts/{}/{}",
        crate::contract::benchmark_run_directory_name(&spec.identity.run)?,
        spec.identity.tool_id,
        spec.identity.campaign_id,
        spec.identity.attempt_id,
    );
    if !spec.outputs.result_root.ends_with(&expected_suffix) {
        return Err(invalid(
            "benchmark result root does not encode the immutable attempt identity",
        ));
    }
    if spec.outputs.mount_path != "/result" {
        return Err(invalid("benchmark result mount must be /result"));
    }
    validate_relative_paths(&spec.outputs.required_files, "required output")
}

fn validate_resources(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    require_text(&spec.resources.account, "Slurm account")?;
    require_text(&spec.resources.partition, "Slurm partition")?;
    if spec.resources.cpus == 0 || spec.resources.memory_mb == 0 || spec.resources.scratch_mb == 0 {
        return Err(invalid("benchmark resources must be positive"));
    }
    validate_walltime(&spec.resources.walltime)?;
    if spec.resources.retry.max_attempts != 1
        || spec.resources.retry.backoff_seconds != 0
        || !spec.resources.retry.retryable_exit_codes.is_empty()
    {
        return Err(invalid("immutable benchmark attempts cannot retry in place"));
    }
    Ok(())
}

fn validate_runtime(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    if spec.runtime.locale != "C"
        || spec.runtime.timezone != "UTC"
        || spec.runtime.runtime_network
        || spec.runtime.threads == 0
        || spec.runtime.threads != spec.resources.cpus
    {
        return Err(invalid("benchmark runtime policy is invalid"));
    }
    if matches!(spec.runtime.determinism, crate::model::BenchmarkExecutionDeterminism::Seeded)
        != spec.runtime.random_seed.is_some()
    {
        return Err(invalid("benchmark random seed and determinism policy disagree"));
    }
    Ok(())
}

fn validate_acceptance(spec: &BenchmarkExecutionSpec) -> Result<(), BenchError> {
    require_text(&spec.acceptance.version_regex, "version acceptance regex")?;
    if !spec.acceptance.negative_exit_nonzero || spec.acceptance.metrics.is_empty() {
        return Err(invalid("benchmark acceptance policy is incomplete"));
    }
    let mut ids = BTreeSet::new();
    for metric in &spec.acceptance.metrics {
        validate_metric(metric)?;
        if !ids.insert(metric.id.as_str()) {
            return Err(invalid("benchmark acceptance duplicates a metric id"));
        }
        if !spec.outputs.required_files.contains(&metric.path) {
            return Err(invalid("benchmark acceptance metric references an undeclared output"));
        }
    }
    Ok(())
}

fn validate_metric(metric: &BenchmarkAcceptanceMetric) -> Result<(), BenchError> {
    validate_component(&metric.id, "acceptance metric id")?;
    validate_relative_path(&metric.path, "acceptance metric path")?;
    if !metric.json_pointer.starts_with('/') || metric.json_pointer.len() == 1 {
        return Err(invalid("benchmark acceptance JSON pointer is invalid"));
    }
    match &metric.expected {
        BenchmarkAcceptanceExpectedValue::Number(value) if !value.is_finite() => {
            return Err(invalid("benchmark acceptance numeric threshold is not finite"));
        }
        BenchmarkAcceptanceExpectedValue::Text(value) => {
            require_text(value, "acceptance expected text")?;
        }
        _ => {}
    }
    require_text(&metric.unit, "acceptance metric unit")
}

fn validate_text_list(values: &[String], label: &str) -> Result<(), BenchError> {
    if values.is_empty() {
        return Err(invalid(&format!("benchmark {label} is empty")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        require_text(value, label)?;
        if !seen.insert(value.as_str()) {
            return Err(invalid(&format!("benchmark {label} contains duplicates")));
        }
    }
    Ok(())
}

fn validate_relative_paths(values: &[String], label: &str) -> Result<(), BenchError> {
    if values.is_empty() {
        return Err(invalid(&format!("benchmark {label} list is empty")));
    }
    let mut seen = BTreeSet::new();
    for value in values {
        validate_relative_path(value, label)?;
        if !seen.insert(value.as_str()) {
            return Err(invalid(&format!("benchmark {label} is duplicated")));
        }
    }
    Ok(())
}

fn validate_relative_path(value: &str, label: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if value.is_empty()
        || path.is_absolute()
        || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid(&format!("benchmark {label} is not a safe relative path")));
    }
    reject_disposable_production_root(value, label)
}

fn require_absolute_path(value: &str, label: &str) -> Result<(), BenchError> {
    let path = Path::new(value);
    if !path.is_absolute()
        || path.components().any(|part| matches!(part, Component::ParentDir | Component::CurDir))
    {
        return Err(invalid(&format!("benchmark {label} is not an absolute normalized path")));
    }
    reject_disposable_production_root(value, label)
}

fn reject_disposable_production_root(value: &str, label: &str) -> Result<(), BenchError> {
    let normalized = value.replace('\\', "/");
    for root in DISPOSABLE_PRODUCTION_ROOTS {
        if normalized.match_indices(root).any(|(start, _)| {
            let before = normalized[..start].chars().next_back();
            let after = normalized[start + root.len()..].chars().next();
            is_path_segment_boundary(before) && is_path_segment_boundary(after)
        }) {
            return Err(invalid(&format!(
                "benchmark {label} references disposable production root {root}"
            )));
        }
    }
    Ok(())
}

fn is_path_segment_boundary(character: Option<char>) -> bool {
    character
        .is_none_or(|value| !value.is_ascii_alphanumeric() && !matches!(value, '_' | '-' | '.'))
}

fn validate_component(value: &str, label: &str) -> Result<(), BenchError> {
    if value.is_empty()
        || !value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'_' | b'-' | b'.')
        })
    {
        return Err(invalid(&format!("benchmark {label} is malformed")));
    }
    Ok(())
}

fn require_text(value: &str, label: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() || value != value.trim() {
        return Err(invalid(&format!("benchmark {label} is empty or malformed")));
    }
    Ok(())
}

fn validate_commit(value: &str) -> Result<(), BenchError> {
    if valid_lower_hex(value, 40) {
        Ok(())
    } else {
        Err(invalid("benchmark repository commit is malformed"))
    }
}

fn validate_digest(value: &str, label: &str) -> Result<(), BenchError> {
    if valid_lower_hex(value, 64) {
        Ok(())
    } else {
        Err(invalid(&format!("benchmark {label} is malformed")))
    }
}

fn valid_lower_hex(value: &str, length: usize) -> bool {
    value.len() == length
        && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
        && value.bytes().any(|byte| byte != b'0')
}

fn parse_timestamp(value: &str) -> Result<DateTime<FixedOffset>, BenchError> {
    DateTime::parse_from_rfc3339(value)
        .map_err(|_| invalid("benchmark prepared_at_utc is not RFC 3339"))
}

fn validate_walltime(value: &str) -> Result<(), BenchError> {
    let fields = value.split(':').collect::<Vec<_>>();
    if fields.len() != 3
        || fields[0].len() < 2
        || fields.iter().any(|field| !field.bytes().all(|byte| byte.is_ascii_digit()))
        || fields[1].parse::<u8>().map_or(true, |part| part > 59)
        || fields[2].parse::<u8>().map_or(true, |part| part > 59)
    {
        return Err(invalid("benchmark Slurm walltime is malformed"));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
