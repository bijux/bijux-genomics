//! Admission of declared completion observations; no filesystem or process effects.
use crate::model::simulation::completion::{
    AllDomainCompletionCheckReport, AllDomainCompletionCheckRow, AllDomainCompletionStatus,
    ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION, MUTATION_EXECUTION_NOT_SUCCESSFUL,
    MUTATION_MISSING_DECLARED_OUTPUT, MUTATION_MISSING_MANIFEST,
    MUTATION_MISSING_NORMALIZED_METRICS, MUTATION_REQUIRED_FILE_EMPTY,
    REASON_EXECUTION_NOT_SUCCESSFUL, REASON_INVALID_MANIFEST, REASON_MANIFEST_OUTPUT_MISMATCH,
    REASON_MISSING_DECLARED_OUTPUTS, REASON_MISSING_MANIFEST, REASON_MISSING_NORMALIZED_METRICS,
    REASON_REQUIRED_FILES_INCOMPLETE,
};
use anyhow::{ensure, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Component, Path};

/// Compute the illustrative behavior verdict after checking every declared row.
/// # Errors
/// Rejects inconsistent identities, evidence flags, counters, paths, or seeded omissions.
pub fn admit_all_domain_completion_check(
    mut report: AllDomainCompletionCheckReport,
) -> Result<AllDomainCompletionCheckReport> {
    validate_content(&report)?;
    report.passes_behavior_test = true;
    Ok(report)
}

/// Validate persisted observations without treating their verdict as fresh execution evidence.
/// # Errors
/// Rejects an unsuccessful verdict or internally contradictory report.
pub fn validate_all_domain_completion_check(report: &AllDomainCompletionCheckReport) -> Result<()> {
    ensure!(report.passes_behavior_test, "completion behavior test has not passed");
    validate_content(report)
}

fn validate_content(report: &AllDomainCompletionCheckReport) -> Result<()> {
    ensure!(
        report.schema_version == ALL_DOMAIN_COMPLETION_CHECK_SCHEMA_VERSION,
        "unsupported completion schema"
    );
    admit_path(&report.output_path)?;
    admit_path(&report.fixture_root)?;
    ensure!(
        !report.rows.is_empty() && report.row_count == report.rows.len(),
        "completion row total differs from observations"
    );
    let mut rows = BTreeMap::new();
    let mut domains = BTreeMap::new();
    let mut reasons = BTreeMap::new();
    let mut complete = 0;
    for row in &report.rows {
        validate_row(row, &report.fixture_root)?;
        ensure!(
            rows.insert(row.result_id.as_str(), row).is_none(),
            "duplicate completion result identity"
        );
        *domains.entry(row.domain.clone()).or_insert(0_usize) += 1;
        for reason in &row.failure_reasons {
            *reasons.entry(reason.clone()).or_insert(0_usize) += 1;
        }
        complete += usize::from(row.completion_status == AllDomainCompletionStatus::Complete);
    }
    ensure!(
        report.complete_row_count == complete
            && report.incomplete_row_count == report.rows.len() - complete,
        "completion status totals differ from rows"
    );
    ensure!(
        report.domain_counts == domains && report.failure_reason_counts == reasons,
        "completion summary counts differ from rows"
    );
    validate_mutations(report, &rows)
}

fn validate_mutations(
    report: &AllDomainCompletionCheckReport,
    rows: &BTreeMap<&str, &AllDomainCompletionCheckRow>,
) -> Result<()> {
    let governed = [
        (
            MUTATION_MISSING_DECLARED_OUTPUT,
            "fastq:corpus-01-mini:fastq.profile_reads:sample-set:seqkit_stats",
            REASON_MISSING_DECLARED_OUTPUTS,
        ),
        (
            MUTATION_MISSING_NORMALIZED_METRICS,
            "fastq:corpus-02-edna-mini:fastq.screen_taxonomy:sample-set:kraken2",
            REASON_MISSING_NORMALIZED_METRICS,
        ),
        (
            MUTATION_MISSING_MANIFEST,
            "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools",
            REASON_MISSING_MANIFEST,
        ),
        (
            MUTATION_REQUIRED_FILE_EMPTY,
            "bam:corpus-01-bam-mini:bam.coverage:sample-set:samtools",
            REASON_REQUIRED_FILES_INCOMPLETE,
        ),
        (
            MUTATION_EXECUTION_NOT_SUCCESSFUL,
            "bam:corpus-01-bam-mini:bam.qc_pre:sample-set:multiqc",
            REASON_EXECUTION_NOT_SUCCESSFUL,
        ),
    ];
    ensure!(
        report.seeded_mutations.len() == governed.len()
            && report.incomplete_row_count == governed.len(),
        "completion omissions differ from the five governed mutations"
    );
    let mut mutations = BTreeMap::new();
    let mut evidence = BTreeSet::new();
    for mutation in &report.seeded_mutations {
        ensure!(
            mutations.insert(mutation.mutation_id.as_str(), mutation).is_none(),
            "duplicate completion mutation identity"
        );
        admit_owned_path(&mutation.evidence_path, &report.fixture_root)?;
        ensure!(
            evidence.insert(&mutation.evidence_path),
            "completion mutation evidence paths collide"
        );
    }
    for (id, result_id, reason) in governed {
        let mutation =
            mutations.get(id).ok_or_else(|| anyhow::anyhow!("missing completion mutation {id}"))?;
        ensure!(mutation.result_id == result_id, "completion mutation targets the wrong result");
        let row = rows
            .get(result_id)
            .ok_or_else(|| anyhow::anyhow!("missing completion result {result_id}"))?;
        ensure!(
            row.completion_status == AllDomainCompletionStatus::Incomplete
                && row.failure_reasons.iter().any(|value| value == reason),
            "completion mutation does not demonstrate its required failure"
        );
        let row_root = Path::new(&row.stage_result_path)
            .parent()
            .ok_or_else(|| anyhow::anyhow!("completion manifest path has no parent"))?;
        ensure!(
            Path::new(&mutation.evidence_path).starts_with(row_root),
            "completion mutation evidence belongs to a different result"
        );
        if id == MUTATION_MISSING_MANIFEST || id == MUTATION_EXECUTION_NOT_SUCCESSFUL {
            ensure!(
                mutation.evidence_path == row.stage_result_path,
                "completion manifest mutation points at different evidence"
            );
        }
        if id == MUTATION_REQUIRED_FILE_EMPTY {
            ensure!(
                mutation.evidence_path == row.command_script_path,
                "completion empty-file mutation points at different evidence"
            );
        }
    }
    Ok(())
}

fn validate_row(row: &AllDomainCompletionCheckRow, root: &str) -> Result<()> {
    validate_identity(row, root)?;
    let ready = validate_required_files(row)?;
    validate_artifact_counts(row)?;
    let successful = validate_manifest_observation(row)?;
    let mut reasons = Vec::new();
    if !row.manifest_exists {
        reasons.push(REASON_MISSING_MANIFEST);
    } else if !row.manifest_valid {
        reasons.push(REASON_INVALID_MANIFEST);
    }
    if row.manifest_exists && row.manifest_valid && !successful {
        reasons.push(REASON_EXECUTION_NOT_SUCCESSFUL);
    }
    if ready != row.required_file_count {
        reasons.push(REASON_REQUIRED_FILES_INCOMPLETE);
    }
    if !row.missing_declared_output_ids.is_empty() {
        reasons.push(REASON_MISSING_DECLARED_OUTPUTS);
    }
    if !row.missing_normalized_metric_ids.is_empty() {
        reasons.push(REASON_MISSING_NORMALIZED_METRICS);
    }
    if row.manifest_exists && row.manifest_valid && !row.manifest_output_match {
        reasons.push(REASON_MANIFEST_OUTPUT_MISMATCH);
    }
    ensure!(
        row.failure_reasons.iter().map(String::as_str).eq(reasons.iter().copied()),
        "completion failure reasons differ from observations"
    );
    ensure!(
        (row.completion_status == AllDomainCompletionStatus::Complete) == reasons.is_empty(),
        "completion status contradicts observations"
    );
    Ok(())
}
fn validate_identity(row: &AllDomainCompletionCheckRow, root: &str) -> Result<()> {
    for value in [&row.domain, &row.stage_id, &row.tool_id, &row.corpus_id, &row.asset_profile_id] {
        ensure!(
            !value.trim().is_empty() && !value.contains(['/', '\\', '\0', ':']),
            "invalid completion identity component"
        );
    }
    ensure!(
        matches!(row.domain.as_str(), "fastq" | "bam" | "vcf")
            && row.stage_id.starts_with(&format!("{}.", row.domain)),
        "completion stage and domain differ"
    );
    let identity = bijux_dna_core::ids::parse_benchmark_result_id(&row.result_id)?;
    ensure!(
        identity.domain.as_str() == row.domain
            && identity.corpus_id == row.corpus_id
            && identity.stage_id.as_str() == row.stage_id
            && identity.tool_id.as_str() == row.tool_id,
        "completion result identity differs from its fields"
    );
    // FASTQ/BAM identities retain sample scopes; VCF identities carry the asset profile.
    ensure!(
        !identity.scope_id.trim().is_empty()
            && !identity.scope_id.contains(['/', '\\', '\0', '\n', '\r'])
            && (identity.scope_kind != bijux_dna_core::ids::BenchmarkResultScopeKind::AssetProfile
                || identity.scope_id == row.asset_profile_id),
        "completion result scope contradicts its domain contract"
    );
    for path in [
        &row.command_script_path,
        &row.stdout_path,
        &row.stderr_path,
        &row.metrics_path,
        &row.stage_result_path,
    ] {
        admit_owned_path(path, root)?;
    }
    Ok(())
}

fn validate_required_files(row: &AllDomainCompletionCheckRow) -> Result<usize> {
    let required = [
        ("command_script", &row.command_script_path),
        ("stdout", &row.stdout_path),
        ("stderr", &row.stderr_path),
        ("metrics", &row.metrics_path),
    ]
    .into_iter()
    .collect::<BTreeMap<_, _>>();
    ensure!(
        row.required_files.len() == required.len() && row.required_file_count == required.len(),
        "completion required file inventory differs"
    );
    let mut seen = BTreeSet::new();
    let mut ready = 0;
    for file in &row.required_files {
        ensure!(
            seen.insert(file.file_id.as_str())
                && required.get(file.file_id.as_str()).is_some_and(|path| *path == &file.path),
            "completion required file identity or path differs"
        );
        ensure!(!file.non_empty || file.exists, "absent completion file cannot contain bytes");
        ready += usize::from(file.exists && file.non_empty);
    }
    ensure!(row.ready_required_file_count == ready, "completion ready-file count differs");
    Ok(ready)
}

fn validate_artifact_counts(row: &AllDomainCompletionCheckRow) -> Result<()> {
    for (total, present, missing) in [
        (
            row.declared_output_count,
            row.present_declared_output_count,
            &row.missing_declared_output_ids,
        ),
        (
            row.normalized_metrics_count,
            row.present_normalized_metrics_count,
            &row.missing_normalized_metric_ids,
        ),
    ] {
        ensure!(
            present <= total && missing.len() == total - present,
            "completion artifact counts differ"
        );
        let unique = missing.iter().collect::<BTreeSet<_>>();
        ensure!(
            unique.len() == missing.len() && missing.iter().all(|id| !id.trim().is_empty()),
            "completion missing-artifact identities repeat or are empty"
        );
    }
    Ok(())
}

fn validate_manifest_observation(row: &AllDomainCompletionCheckRow) -> Result<bool> {
    ensure!(row.exit_code_zero == (row.exit_code == Some(0)), "completion exit-code flag differs");
    ensure!(
        row.runtime_status.as_deref() != Some("succeeded") || row.exit_code_zero,
        "successful completion runtime has a nonzero exit code"
    );
    ensure!(
        row.runtime_status.is_some() == row.exit_code.is_some(),
        "completion runtime observation is partial"
    );
    ensure!(
        row.runtime_status.as_deref().is_none_or(|status| matches!(status, "succeeded" | "failed")),
        "unknown completion runtime status"
    );
    ensure!(
        row.manifest_exists
            || (!row.manifest_valid && !row.manifest_output_match && row.runtime_status.is_none()),
        "absent completion manifest has observations"
    );
    ensure!(
        !row.manifest_valid || row.runtime_status.is_some(),
        "valid completion manifest omits runtime observation"
    );
    ensure!(
        !row.manifest_output_match || row.manifest_valid,
        "invalid completion manifest cannot match outputs"
    );
    Ok(row.exit_code_zero && row.runtime_status.as_deref() == Some("succeeded"))
}

fn admit_path(value: &str) -> Result<()> {
    ensure!(
        !value.trim().is_empty()
            && !value.contains(['\0', '\n', '\r'])
            && !Path::new(value).components().any(|part| matches!(part, Component::ParentDir)),
        "invalid completion evidence path"
    );
    Ok(())
}
fn admit_owned_path(value: &str, root: &str) -> Result<()> {
    admit_path(value)?;
    ensure!(
        Path::new(value).starts_with(root) && Path::new(value) != Path::new(root),
        "completion evidence path is outside its fixture root"
    );
    Ok(())
}
