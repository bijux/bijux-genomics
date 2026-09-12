//! Semantic admission for persisted local stage-command rendering records.

use std::collections::BTreeSet;

use crate::{
    BenchmarkLocalStageArtifactEntry, BenchmarkLocalStageCommandManifest, StageContractError,
    BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
};

/// Validate a persisted local stage-command manifest and its aggregate closure.
///
/// # Errors
/// Returns an error when identities, paths, resources, arguments, or aggregates drift.
pub fn validate_benchmark_local_stage_commands(
    manifest: &BenchmarkLocalStageCommandManifest,
) -> Result<(), StageContractError> {
    if manifest.schema_version != BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3 {
        return invalid("unsupported local stage-command schema");
    }
    for (field, value, suffix) in [
        ("script_output_path", manifest.script_output_path.as_str(), ".sh"),
        ("manifest_output_path", manifest.manifest_output_path.as_str(), ".json"),
        ("argv_output_path", manifest.argv_output_path.as_str(), ".jsonl"),
    ] {
        require_text(value, field)?;
        if !value.ends_with(suffix) {
            return invalid(format!("local stage-command {field} must end with {suffix}"));
        }
    }
    if manifest.commands.is_empty() || manifest.command_count != manifest.commands.len() {
        return invalid("local stage-command aggregate does not close");
    }

    let mut stage_ids = BTreeSet::new();
    for command in &manifest.commands {
        for (field, value) in [
            ("commands[].stage_id", command.stage_id.as_str()),
            ("commands[].tool_id", command.tool_id.as_str()),
            ("commands[].command", command.command.as_str()),
        ] {
            require_text(value, field)?;
        }
        let domain = command.stage_id.split_once('.').map(|(domain, _)| domain);
        if !matches!(domain, Some("fastq" | "bam")) {
            return invalid("local stage-command stage lies outside the governed domains");
        }
        if !stage_ids.insert(command.stage_id.as_str()) {
            return invalid("local stage-command repeats a stage identity");
        }
        if command.threads == 0 || command.memory_mb == 0 || command.argv.is_empty() {
            return invalid("local stage-command requires resources and argv");
        }
        if command.argv.iter().any(String::is_empty)
            || command.argv.last().map(String::as_str) != Some(command.stage_id.as_str())
        {
            return invalid("local stage-command argv does not close over its stage identity");
        }
        validate_artifacts(&command.inputs, "commands[].inputs")?;
        validate_artifacts(&command.outputs, "commands[].outputs")?;
    }
    Ok(())
}

fn validate_artifacts(
    artifacts: &[BenchmarkLocalStageArtifactEntry],
    field: &str,
) -> Result<(), StageContractError> {
    if artifacts.is_empty() {
        return invalid(format!("local stage-command requires non-empty {field}"));
    }
    let mut identities = BTreeSet::new();
    for artifact in artifacts {
        require_text(&artifact.artifact_id, field)?;
        require_text(&artifact.path, field)?;
        require_text(&artifact.role, field)?;
        if !identities.insert((artifact.artifact_id.as_str(), artifact.path.as_str())) {
            return invalid(format!("local stage-command repeats an artifact in {field}"));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), StageContractError> {
    if value.trim().is_empty() {
        return invalid(format!("local stage-command requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, StageContractError> {
    Err(StageContractError::invalid(message.into()))
}
