//! Semantic validation for persisted benchmark dossier indexes.

use std::collections::BTreeSet;

use crate::{
    validate_utc_timestamp, BenchError, BenchmarkDossierIndex, BenchmarkDossierStageEntry,
    BENCHMARK_DOSSIER_INDEX_SCHEMA_V1,
};

/// Validate dossier identity, paths, publication state, metadata, and counters.
///
/// # Errors
/// Returns an error when any persisted dossier invariant is inconsistent.
pub fn validate_benchmark_dossier_index(index: &BenchmarkDossierIndex) -> Result<(), BenchError> {
    if index.schema_version != BENCHMARK_DOSSIER_INDEX_SCHEMA_V1 {
        return invalid("unsupported benchmark dossier index schema");
    }
    require_text(&index.corpus_id, "corpus_id")?;
    if index.stage_count == 0 || index.stage_count != index.stages.len() {
        return invalid("dossier stage_count must equal a non-empty stage index");
    }
    if index.published_stage_count + index.missing_stage_count != index.stage_count {
        return invalid("dossier publication counters do not partition the stage index");
    }

    let mut stage_ids = BTreeSet::new();
    let mut summary_paths = BTreeSet::new();
    let mut dossier_paths = BTreeSet::new();
    let mut published_count = 0usize;
    for stage in &index.stages {
        validate_stage_identity(stage, &mut stage_ids, &mut summary_paths, &mut dossier_paths)?;
        match stage.status.as_str() {
            "published" => {
                published_count += 1;
                validate_published_stage(stage)?;
            }
            "missing" => validate_missing_stage(stage)?,
            _ => {
                return invalid(format!(
                    "dossier stage `{}` has unknown status `{}`",
                    stage.stage_id, stage.status
                ));
            }
        }
    }
    if index.published_stage_count != published_count
        || index.missing_stage_count != index.stages.len() - published_count
    {
        return invalid("dossier counters do not match stage publication states");
    }
    Ok(())
}

fn validate_stage_identity<'a>(
    stage: &'a BenchmarkDossierStageEntry,
    stage_ids: &mut BTreeSet<&'a str>,
    summary_paths: &mut BTreeSet<&'a str>,
    dossier_paths: &mut BTreeSet<&'a str>,
) -> Result<(), BenchError> {
    for (value, field) in [
        (&stage.stage_id, "stages[].stage_id"),
        (&stage.sample_scope, "stages[].sample_scope"),
        (&stage.summary_path, "stages[].summary_path"),
        (&stage.dossier_path, "stages[].dossier_path"),
        (&stage.expected_remote_run_root, "stages[].expected_remote_run_root"),
        (
            &stage.expected_local_cache_mirror_run_root,
            "stages[].expected_local_cache_mirror_run_root",
        ),
        (&stage.expected_local_results_run_root, "stages[].expected_local_results_run_root"),
    ] {
        require_text(value, field)?;
    }
    if !stage_ids.insert(&stage.stage_id) {
        return invalid(format!("dossier repeats stage `{}`", stage.stage_id));
    }
    if !summary_paths.insert(&stage.summary_path) || !dossier_paths.insert(&stage.dossier_path) {
        return invalid(format!("dossier stage `{}` repeats a publication path", stage.stage_id));
    }
    Ok(())
}

fn validate_published_stage(stage: &BenchmarkDossierStageEntry) -> Result<(), BenchError> {
    let generated_at_utc = required_option(stage.generated_at_utc.as_deref(), "generated_at_utc")?;
    validate_utc_timestamp(generated_at_utc, "stages[].generated_at_utc")?;
    required_option(stage.platform.as_deref(), "platform")?;
    required_option(stage.corpus_root.as_deref(), "corpus_root")?;
    let run_root = required_option(stage.run_root.as_deref(), "run_root")?;
    let source = required_option(stage.run_root_source.as_deref(), "run_root_source")?;
    if !matches!(
        source,
        "local-cache-mirror"
            | "local-results-root"
            | "remote-results-root"
            | "remote-custom"
            | "custom"
    ) {
        return invalid(format!(
            "dossier stage `{}` has unknown run-root source `{source}`",
            stage.stage_id
        ));
    }
    let expected_root = match source {
        "local-cache-mirror" => Some(stage.expected_local_cache_mirror_run_root.as_str()),
        "local-results-root" => Some(stage.expected_local_results_run_root.as_str()),
        "remote-results-root" => Some(stage.expected_remote_run_root.as_str()),
        "remote-custom" | "custom" => None,
        _ => None,
    };
    if expected_root.is_some_and(|expected| run_root != expected) {
        return invalid(format!(
            "dossier stage `{}` run root contradicts source `{source}`",
            stage.stage_id
        ));
    }
    Ok(())
}

fn validate_missing_stage(stage: &BenchmarkDossierStageEntry) -> Result<(), BenchError> {
    if [
        stage.generated_at_utc.as_deref(),
        stage.platform.as_deref(),
        stage.corpus_root.as_deref(),
        stage.run_root.as_deref(),
        stage.run_root_source.as_deref(),
    ]
    .iter()
    .any(Option::is_some)
    {
        return invalid(format!(
            "dossier missing stage `{}` contains publication metadata",
            stage.stage_id
        ));
    }
    Ok(())
}

fn required_option<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, BenchError> {
    let value = value
        .ok_or_else(|| BenchError::InvalidPolicy(format!("published dossier requires {field}")))?;
    require_text(value, field)?;
    Ok(value)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("benchmark dossier index requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
