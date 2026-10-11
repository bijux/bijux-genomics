//! Semantic admission for illustrative failure evidence; no command execution is asserted.
use crate::{
    AllDomainFakeFailureRecord, AllDomainFakeFailuresManifest,
    ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION, ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION,
};
use anyhow::{ensure, Context, Result};
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

/// Admit a complete failure record with explicit nonzero outcome and bound output paths.
/// # Errors
/// Rejects schema, identity, path, output cardinality and failure-state contradictions.
pub fn validate_all_domain_failure_simulation_record(
    record: &AllDomainFakeFailureRecord,
) -> Result<()> {
    ensure!(
        record.schema_version == ALL_DOMAIN_FAKE_FAILURE_RECORD_SCHEMA_VERSION,
        "unsupported all-domain failure record schema"
    );
    ensure!(record.exit_code != 0, "failure simulation requires a nonzero exit code");
    for value in [
        &record.result_id,
        &record.domain,
        &record.stage_id,
        &record.tool_id,
        &record.corpus_id,
        &record.asset_profile_id,
        &record.command_source,
        &record.command,
    ] {
        ensure!(
            !value.trim().is_empty(),
            "failure record contains an empty required identity or command"
        );
    }
    for component in [
        &record.domain,
        &record.corpus_id,
        &record.stage_id,
        &record.asset_profile_id,
        &record.tool_id,
    ] {
        super::admit_path_component(component)?;
    }
    let parent = Path::new(&record.failure_record_path)
        .parent()
        .context("failure record has no result directory")?;
    for (path, suffix) in [
        (&record.command_script_path, "command.sh"),
        (&record.stderr_path, "stderr.txt"),
        (&record.failure_record_path, "failure.json"),
    ] {
        require_path(path)?;
        ensure!(
            Path::new(path) == parent.join(suffix),
            "failure record output paths belong to different result directories"
        );
    }
    ensure!(
        record.failed_output_count > 0 && record.failed_output_count == record.failed_outputs.len(),
        "failure record must enumerate every failed output"
    );
    let mut identities = BTreeSet::new();
    let mut paths = BTreeSet::new();
    for output in &record.failed_outputs {
        ensure!(
            output.role == crate::all_domain_simulation_output_role(&output.artifact_id)
                && output.declared_output == output.artifact_id,
            "failure record output identity is inconsistent"
        );
        ensure!(
            identities.insert(&output.artifact_id) && paths.insert(&output.expected_fake_run_path),
            "failure record repeats an output identity or path"
        );
        super::admit_path_component(&output.artifact_id)?;
        require_path(&output.expected_fake_run_path)?;
        ensure!(
            Path::new(&output.expected_fake_run_path)
                == parent
                    .join("declared-outputs")
                    .join(crate::all_domain_simulation_output_path(&output.artifact_id)),
            "failure output escapes its result directory"
        );
    }
    Ok(())
}

/// Admit a complete failure manifest without interpreting it as observed tool execution.
/// # Errors
/// Rejects inconsistent schema, paths, identities, outcomes, domain totals or output totals.
pub fn validate_all_domain_failure_simulation_manifest(
    report: &AllDomainFakeFailuresManifest,
) -> Result<()> {
    ensure!(
        report.schema_version == ALL_DOMAIN_FAKE_FAILURE_MANIFEST_SCHEMA_VERSION,
        "unsupported all-domain failure manifest schema"
    );
    require_path(&report.failure_root)?;
    require_path(&report.root_manifest_path)?;
    ensure!(
        Path::new(&report.root_manifest_path)
            == Path::new(&report.failure_root).join("manifest.json"),
        "failure manifest path differs from its root"
    );
    ensure!(
        report.exit_code != 0
            && report.result_count > 0
            && report.result_count == report.failures.len(),
        "failure manifest must contain every nonzero failure outcome"
    );
    let mut identities = BTreeSet::new();
    let mut paths = BTreeSet::new();
    let mut domains = BTreeMap::new();
    let mut count = 0_usize;
    for record in &report.failures {
        validate_all_domain_failure_simulation_record(record)?;
        ensure!(record.exit_code == report.exit_code, "failure outcome differs from its manifest");
        ensure!(
            identities.insert(&record.result_id) && paths.insert(&record.failure_record_path),
            "failure manifest repeats a result identity or path"
        );
        ensure!(
            Path::new(&record.failure_record_path)
                == Path::new(&report.failure_root)
                    .join(&record.domain)
                    .join(&record.corpus_id)
                    .join(&record.stage_id)
                    .join(&record.asset_profile_id)
                    .join(&record.tool_id)
                    .join("failure.json"),
            "failure record escapes its declared root"
        );
        *domains.entry(record.domain.clone()).or_insert(0_usize) += 1;
        count = count
            .checked_add(record.failed_output_count)
            .context("failure output total overflow")?;
    }
    ensure!(
        domains == report.domain_counts && count == report.failed_output_count,
        "failure manifest counters differ from its records"
    );
    Ok(())
}
fn require_path(value: &str) -> Result<()> {
    ensure!(
        !value.is_empty()
            && !Path::new(value).is_absolute()
            && !value.contains(['\\', '\0'])
            && value.split('/').all(|part| !part.is_empty() && part != "." && part != ".."),
        "failure evidence requires a normalized repository-relative path"
    );
    Ok(())
}
