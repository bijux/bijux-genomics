//! Semantic admission for persisted local stage fake-run and fake-failure records.

use std::collections::BTreeSet;
use std::path::{Component, Path};

use crate::{
    BenchError, BenchmarkLocalStageFakeFailureManifest, BenchmarkLocalStageFakeFailureRecord,
    BenchmarkLocalStageFakeRunManifest, BenchmarkLocalStageFakeRunResult,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1, BENCHMARK_STAGE_RESULT_SCHEMA_V2,
};

/// Validate one successful fake-run stage record.
///
/// # Errors
/// Returns an error when identity, output, counter, or path closure is invalid.
pub fn validate_benchmark_local_stage_fake_run_result(
    result: &BenchmarkLocalStageFakeRunResult,
) -> Result<(), BenchError> {
    if result.schema_version != BENCHMARK_STAGE_RESULT_SCHEMA_V2 {
        return invalid("unsupported local stage fake-run result schema");
    }
    require_text(&result.stage_id, "stage_id")?;
    require_text(&result.tool_id, "tool_id")?;
    require_text(&result.command, "command")?;
    require_path(&result.stage_manifest_path, "stage_manifest_path")?;
    let Some(stage_root) = Path::new(&result.stage_manifest_path).parent() else {
        return invalid("local stage fake-run stage_manifest_path has no parent");
    };
    if stage_root.as_os_str().is_empty() {
        return invalid("local stage fake-run stage_manifest_path requires a governed parent");
    }
    if result.outputs.is_empty()
        || result.declared_output_count != result.outputs.len()
        || result.created_output_count
            != result.outputs.iter().filter(|output| output.exists).count()
        || result.created_output_count != result.declared_output_count
    {
        return invalid("local stage fake-run output counts are inconsistent");
    }
    // One stage record may aggregate cases, so declared paths disambiguate case-local artifact IDs.
    let mut artifact_ids = BTreeSet::new();
    for output in &result.outputs {
        require_text(&output.artifact_id, "outputs[].artifact_id")?;
        require_path(&output.declared_path, "outputs[].declared_path")?;
        require_path(&output.fake_run_path, "outputs[].fake_run_path")?;
        require_descendant_path(stage_root, &output.fake_run_path, "outputs[].fake_run_path")?;
        require_text(&output.role, "outputs[].role")?;
        if !output.exists {
            return invalid("successful local stage fake-run output must exist");
        }
        if !artifact_ids.insert((output.artifact_id.as_str(), output.declared_path.as_str())) {
            return invalid("local stage fake-run repeats an artifact identity and declared path");
        }
    }
    Ok(())
}

/// Validate a successful local stage fake-run manifest.
///
/// # Errors
/// Returns an error when stage inventory, identity, counter, or path closure is invalid.
pub fn validate_benchmark_local_stage_fake_run_manifest(
    manifest: &BenchmarkLocalStageFakeRunManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_STAGE_FAKE_RUN_MANIFEST_SCHEMA_V1 {
        return invalid("unsupported local stage fake-run manifest schema");
    }
    require_path(&manifest.fake_run_root, "fake_run_root")?;
    require_path(
        &manifest.source_stage_command_manifest_path,
        "source_stage_command_manifest_path",
    )?;
    if manifest.stages.is_empty() || manifest.stage_count != manifest.stages.len() {
        return invalid("local stage fake-run stage_count must match a non-empty stage inventory");
    }
    let mut stage_ids = BTreeSet::new();
    for stage in &manifest.stages {
        validate_benchmark_local_stage_fake_run_result(stage)?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid("local stage fake-run repeats a stage identity");
        }
        require_descendant(
            &manifest.fake_run_root,
            &stage.stage_manifest_path,
            "stages[].stage_manifest_path",
        )?;
        for output in &stage.outputs {
            require_descendant(
                &manifest.fake_run_root,
                &output.fake_run_path,
                "stages[].outputs[].fake_run_path",
            )?;
        }
    }
    Ok(())
}

/// Validate one local stage fake-failure record.
///
/// # Errors
/// Returns an error when failure identity, output, counter, or path closure is invalid.
pub fn validate_benchmark_local_stage_fake_failure_record(
    record: &BenchmarkLocalStageFakeFailureRecord,
) -> Result<(), BenchError> {
    if record.schema_version != BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_RECORD_SCHEMA_V1 {
        return invalid("unsupported local stage fake-failure record schema");
    }
    require_text(&record.stage_id, "stage_id")?;
    require_text(&record.tool_id, "tool_id")?;
    require_text(&record.command, "command")?;
    if record.exit_code == 0 {
        return invalid("local stage fake-failure exit_code must be non-zero");
    }
    require_path(&record.stderr_path, "stderr_path")?;
    require_path(&record.failure_record_path, "failure_record_path")?;
    let Some(stage_root) = Path::new(&record.failure_record_path).parent() else {
        return invalid("local stage fake-failure failure_record_path has no parent");
    };
    if stage_root.as_os_str().is_empty() {
        return invalid("local stage fake-failure failure_record_path requires a governed parent");
    }
    require_descendant_path(stage_root, &record.stderr_path, "stderr_path")?;
    if record.failed_outputs.is_empty() || record.failed_output_count != record.failed_outputs.len()
    {
        return invalid("local stage fake-failure output count is inconsistent");
    }
    // One stage record may aggregate cases, so declared paths disambiguate case-local artifact IDs.
    let mut artifact_ids = BTreeSet::new();
    for output in &record.failed_outputs {
        require_text(&output.artifact_id, "failed_outputs[].artifact_id")?;
        require_path(&output.declared_path, "failed_outputs[].declared_path")?;
        require_path(&output.expected_fake_run_path, "failed_outputs[].expected_fake_run_path")?;
        require_descendant_path(
            stage_root,
            &output.expected_fake_run_path,
            "failed_outputs[].expected_fake_run_path",
        )?;
        require_text(&output.role, "failed_outputs[].role")?;
        if !artifact_ids.insert((output.artifact_id.as_str(), output.declared_path.as_str())) {
            return invalid(
                "local stage fake-failure repeats an artifact identity and declared path",
            );
        }
    }
    Ok(())
}

/// Validate a local stage fake-failure manifest.
///
/// # Errors
/// Returns an error when stage inventory, identity, counter, or path closure is invalid.
pub fn validate_benchmark_local_stage_fake_failure_manifest(
    manifest: &BenchmarkLocalStageFakeFailureManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_STAGE_FAKE_FAILURE_MANIFEST_SCHEMA_V1 {
        return invalid("unsupported local stage fake-failure manifest schema");
    }
    require_path(&manifest.failure_root, "failure_root")?;
    require_path(
        &manifest.source_stage_command_manifest_path,
        "source_stage_command_manifest_path",
    )?;
    if manifest.failures.is_empty() || manifest.stage_count != manifest.failures.len() {
        return invalid(
            "local stage fake-failure stage_count must match a non-empty failure inventory",
        );
    }
    let mut stage_ids = BTreeSet::new();
    for failure in &manifest.failures {
        validate_benchmark_local_stage_fake_failure_record(failure)?;
        if !stage_ids.insert(failure.stage_id.as_str()) {
            return invalid("local stage fake-failure repeats a stage identity");
        }
        for (path, field) in [
            (&failure.stderr_path, "failures[].stderr_path"),
            (&failure.failure_record_path, "failures[].failure_record_path"),
        ] {
            require_descendant(&manifest.failure_root, path, field)?;
        }
        for output in &failure.failed_outputs {
            require_descendant(
                &manifest.failure_root,
                &output.expected_fake_run_path,
                "failures[].failed_outputs[].expected_fake_run_path",
            )?;
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local stage fake-run evidence requires non-empty {field}"));
    }
    Ok(())
}

fn require_path(value: &str, field: &str) -> Result<(), BenchError> {
    require_text(value, field)?;
    let path = Path::new(value);
    if path.is_absolute()
        || path.components().any(|part| {
            matches!(
                part,
                Component::Prefix(_)
                    | Component::RootDir
                    | Component::ParentDir
                    | Component::CurDir
            )
        })
    {
        return invalid(format!(
            "local stage fake-run evidence requires safe repository-relative {field}"
        ));
    }
    Ok(())
}

fn require_descendant(root: &str, value: &str, field: &str) -> Result<(), BenchError> {
    require_descendant_path(Path::new(root), value, field)
}

fn require_descendant_path(root: &Path, value: &str, field: &str) -> Result<(), BenchError> {
    require_path(value, field)?;
    if !Path::new(value).starts_with(root) {
        return invalid(format!(
            "local stage fake-run evidence requires {field} below its governed root"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
