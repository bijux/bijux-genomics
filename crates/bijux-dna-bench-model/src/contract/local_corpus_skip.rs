//! Validation for persisted local benchmark corpus-skip reports.

use std::collections::{BTreeMap, BTreeSet};

use crate::{
    BenchError, BenchmarkLocalCorpusSkipReport, BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1,
};

/// Validate corpus identity, skip lineage, stage coverage, and report counters.
///
/// # Errors
/// Returns an error when report identity, references, uniqueness, or counters are inconsistent.
pub fn validate_benchmark_local_corpus_skip_report(
    report: &BenchmarkLocalCorpusSkipReport,
) -> Result<(), BenchError> {
    if report.schema_version != BENCHMARK_LOCAL_CORPUS_SKIP_REPORT_SCHEMA_V1 {
        return invalid(format!(
            "unsupported local corpus-skip report schema `{}`",
            report.schema_version
        ));
    }
    require_text(&report.matrix_path, "matrix_path")?;
    require_text(&report.output_path, "output_path")?;
    if report.fixture_count == 0 || report.fixture_count != report.fixtures.len() {
        return invalid("fixture_count must equal a non-empty fixtures array");
    }
    if report.stage_count == 0 {
        return invalid("stage_count must be positive");
    }
    if report.skip_count != report.skips.len()
        || report.asset_backed_stage_count != report.asset_backed_stages.len()
        || report.planner_only_stage_count != report.planner_only_stages.len()
    {
        return invalid("report counters must equal their persisted arrays");
    }

    let mut fixtures = BTreeMap::new();
    for fixture in &report.fixtures {
        require_text(&fixture.corpus_id, "fixtures[].corpus_id")?;
        require_text(&fixture.corpus_family_id, "fixtures[].corpus_family_id")?;
        if fixtures.insert(fixture.corpus_id.as_str(), fixture.corpus_family_id.as_str()).is_some()
        {
            return invalid(format!("duplicate fixture corpus `{}`", fixture.corpus_id));
        }
    }

    let mut stages = BTreeSet::new();
    let mut skips = BTreeSet::new();
    for skip in &report.skips {
        validate_stage_fields(&skip.stage_id, &skip.readiness_kind, &skip.reason)?;
        let Some(corpus_family_id) = fixtures.get(skip.corpus_id.as_str()) else {
            return invalid(format!("skip references unknown corpus `{}`", skip.corpus_id));
        };
        if *corpus_family_id != skip.corpus_family_id {
            return invalid(format!("skip corpus family drifted for `{}`", skip.corpus_id));
        }
        let Some(replacement_family_id) = fixtures.get(skip.replacement_corpus_id.as_str()) else {
            return invalid(format!(
                "skip references unknown replacement corpus `{}`",
                skip.replacement_corpus_id
            ));
        };
        if *replacement_family_id != skip.replacement_corpus_family_id {
            return invalid(format!(
                "replacement corpus family drifted for `{}`",
                skip.replacement_corpus_id
            ));
        }
        if skip.corpus_id == skip.replacement_corpus_id {
            return invalid("skip corpus and replacement corpus must differ");
        }
        if !skips.insert((skip.stage_id.as_str(), skip.corpus_id.as_str())) {
            return invalid(format!(
                "duplicate skip for stage `{}` and corpus `{}`",
                skip.stage_id, skip.corpus_id
            ));
        }
        stages.insert(skip.stage_id.as_str());
    }
    for stage in &report.asset_backed_stages {
        validate_stage_fields(&stage.stage_id, &stage.readiness_kind, &stage.reason)?;
        require_text(&stage.benchmark_scope_id, "asset_backed_stages[].benchmark_scope_id")?;
        if !stages.insert(stage.stage_id.as_str()) {
            return invalid(format!("stage `{}` has multiple corpus dispositions", stage.stage_id));
        }
    }
    for stage in &report.planner_only_stages {
        validate_stage_fields(&stage.stage_id, &stage.readiness_kind, &stage.reason)?;
        if !stages.insert(stage.stage_id.as_str()) {
            return invalid(format!("stage `{}` has multiple corpus dispositions", stage.stage_id));
        }
    }
    if report.stage_count != stages.len() {
        return invalid("stage_count must equal the distinct persisted stage dispositions");
    }
    Ok(())
}

fn validate_stage_fields(
    stage_id: &str,
    readiness_kind: &str,
    reason: &str,
) -> Result<(), BenchError> {
    require_text(stage_id, "stage_id")?;
    if !stage_id.contains('.') {
        return invalid(format!("stage `{stage_id}` is not domain-qualified"));
    }
    require_text(readiness_kind, "readiness_kind")?;
    if !matches!(readiness_kind, "dry_run" | "smoke" | "dry_or_smoke") {
        return invalid(format!(
            "stage `{stage_id}` has unsupported readiness kind `{readiness_kind}`"
        ));
    }
    require_text(reason, "reason")
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("local corpus-skip report requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
