//! Validation for persisted local corpus-to-stage compatibility reports.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, BenchmarkLocalCorpusStageCompatibilityEntry,
    BenchmarkLocalCorpusStageCompatibilityReport,
    BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1,
};

/// Validate fixture lineage, stage dispositions, counters, and report identity.
///
/// # Errors
/// Returns an error when a persisted compatibility report is incomplete or inconsistent.
pub fn validate_benchmark_local_corpus_stage_compatibility_report(
    report: &BenchmarkLocalCorpusStageCompatibilityReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_CORPUS_STAGE_COMPATIBILITY_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local corpus-stage compatibility schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.matrix_path, "matrix_path")?;
    if !report.valid {
        return invalid("local corpus-stage compatibility report must be valid");
    }
    if report.fixture_count == 0 || report.fixture_count != report.fixtures.len() {
        return invalid("fixture_count must equal a non-empty fixtures array");
    }
    if report.stage_count == 0 || report.stage_count != report.stages.len() {
        return invalid("stage_count must equal a non-empty stages array");
    }

    let mut fixtures = BTreeMap::new();
    for fixture in &report.fixtures {
        require_text(&fixture.corpus_family_id, "fixtures[].corpus_family_id")?;
        require_text(&fixture.fixture_id, "fixtures[].fixture_id")?;
        require_text(&fixture.fixture_manifest, "fixtures[].fixture_manifest")?;
        require_text(&fixture.summary, "fixtures[].summary")?;
        if fixtures.insert(fixture.fixture_id.as_str(), fixture.corpus_family_id.as_str()).is_some()
        {
            return invalid(format!("duplicate fixture `{}`", fixture.fixture_id));
        }
    }

    let mut stage_ids = BTreeSet::new();
    let mut fixture_backed = 0usize;
    let mut asset_backed = 0usize;
    let mut planner_only = 0usize;
    let mut corpus_family_counts = BTreeMap::<String, usize>::new();
    for stage in &report.stages {
        validate_stage_identity(stage)?;
        if !stage_ids.insert(stage.stage_id.as_str()) {
            return invalid(format!("duplicate stage `{}`", stage.stage_id));
        }
        match stage.compatibility_kind.as_str() {
            "fixture" => {
                fixture_backed += 1;
                let fixture_id = required_optional_text(stage.fixture_id.as_deref(), "fixture_id")?;
                let family_id =
                    required_optional_text(stage.corpus_family_id.as_deref(), "corpus_family_id")?;
                if stage.benchmark_scope_id.is_some() {
                    return invalid(format!(
                        "fixture-backed stage `{}` cannot declare benchmark_scope_id",
                        stage.stage_id
                    ));
                }
                let Some(expected_family) = fixtures.get(fixture_id) else {
                    return invalid(format!(
                        "stage `{}` references unknown fixture `{fixture_id}`",
                        stage.stage_id
                    ));
                };
                if *expected_family != family_id {
                    return invalid(format!(
                        "stage `{}` fixture family drifted for `{fixture_id}`",
                        stage.stage_id
                    ));
                }
                *corpus_family_counts.entry(family_id.to_string()).or_default() += 1;
            }
            "asset_backed" => {
                asset_backed += 1;
                require_absent_fixture(stage)?;
                required_optional_text(stage.benchmark_scope_id.as_deref(), "benchmark_scope_id")?;
            }
            "planner_only" => {
                planner_only += 1;
                require_absent_fixture(stage)?;
                if stage.benchmark_scope_id.is_some() {
                    return invalid(format!(
                        "planner-only stage `{}` cannot declare benchmark_scope_id",
                        stage.stage_id
                    ));
                }
            }
            kind => {
                return invalid(format!(
                    "stage `{}` has unsupported compatibility kind `{kind}`",
                    stage.stage_id
                ));
            }
        }
    }

    if report.fixture_backed_stage_count != fixture_backed
        || report.asset_backed_stage_count != asset_backed
        || report.planner_only_stage_count != planner_only
        || fixture_backed + asset_backed + planner_only != report.stage_count
    {
        return invalid("stage disposition counters must equal persisted stage entries");
    }
    if report.corpus_family_counts != corpus_family_counts {
        return invalid("corpus_family_counts drifted from fixture-backed stages");
    }
    Ok(())
}

fn validate_stage_identity(
    stage: &BenchmarkLocalCorpusStageCompatibilityEntry,
) -> Result<(), BenchError> {
    require_text(&stage.stage_id, "stages[].stage_id")?;
    if !stage.stage_id.contains('.') {
        return invalid(format!("stage `{}` is not domain-qualified", stage.stage_id));
    }
    require_text(&stage.readiness_kind, "stages[].readiness_kind")?;
    if !matches!(stage.readiness_kind.as_str(), "dry_run" | "smoke" | "dry_or_smoke") {
        return invalid(format!(
            "stage `{}` has unsupported readiness kind `{}`",
            stage.stage_id, stage.readiness_kind
        ));
    }
    require_text(&stage.compatibility_note, "stages[].compatibility_note")
}

fn require_absent_fixture(
    stage: &BenchmarkLocalCorpusStageCompatibilityEntry,
) -> Result<(), BenchError> {
    if stage.corpus_family_id.is_some() || stage.fixture_id.is_some() {
        return invalid(format!(
            "stage `{}` cannot declare fixture lineage for `{}` compatibility",
            stage.stage_id, stage.compatibility_kind
        ));
    }
    Ok(())
}

fn required_optional_text<'a>(value: Option<&'a str>, field: &str) -> Result<&'a str, BenchError> {
    let value = value.ok_or_else(|| {
        BenchError::InvalidPolicy(format!(
            "local corpus-stage compatibility report requires {field}"
        ))
    })?;
    require_text(value, field)?;
    Ok(value)
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!(
            "local corpus-stage compatibility report requires non-empty {field}"
        ));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
