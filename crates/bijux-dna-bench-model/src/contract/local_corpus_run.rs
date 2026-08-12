//! Validation for persisted FASTQ corpus benchmark run records.

use std::collections::BTreeSet;

use crate::{BenchError, BenchmarkCorpusRunManifest, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1};

use super::validate_utc_timestamp;

/// Validate one FASTQ corpus benchmark run manifest.
///
/// # Errors
/// Returns an error when identity, cardinality, resource, or sample-result invariants are invalid.
pub fn validate_benchmark_local_corpus_run(
    manifest: &BenchmarkCorpusRunManifest,
) -> Result<(), BenchError> {
    if manifest.schema_version != BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1 {
        return invalid("unsupported FASTQ corpus benchmark run schema");
    }
    for (value, field) in [
        (&manifest.corpus_id, "corpus_id"),
        (&manifest.stage_id, "stage_id"),
        (&manifest.scenario_id, "scenario_id"),
        (&manifest.sample_scope, "sample_scope"),
        (&manifest.tool_kind, "tool_kind"),
        (&manifest.platform, "platform"),
        (&manifest.config_path, "config_path"),
        (&manifest.publication_config_path, "publication_config_path"),
        (&manifest.repo_root, "repo_root"),
        (&manifest.corpus_root, "corpus_root"),
        (&manifest.out_root, "out_root"),
    ] {
        require_text(value, field)?;
    }
    validate_utc_timestamp(&manifest.generated_at_utc, "generated_at_utc")?;
    if manifest.threads == 0 || manifest.jobs == 0 || manifest.sample_jobs == 0 {
        return invalid("FASTQ corpus benchmark resources must be non-zero");
    }
    if manifest.tools.is_empty() || manifest.tools.iter().any(|tool| tool.trim().is_empty()) {
        return invalid("FASTQ corpus benchmark requires a non-empty tool roster");
    }
    let unique_tools = manifest.tools.iter().collect::<BTreeSet<_>>();
    if unique_tools.len() != manifest.tools.len() {
        return invalid("FASTQ corpus benchmark tool roster contains duplicates");
    }
    if manifest.samples_total != manifest.runs.len() {
        return invalid("FASTQ corpus benchmark samples_total does not match run rows");
    }
    if manifest.sample_limit.is_some_and(|limit| manifest.samples_total > limit) {
        return invalid("FASTQ corpus benchmark exceeds its declared sample_limit");
    }
    reject_reserved_extra_fields(
        manifest.extra_fields.keys().map(String::as_str),
        &[
            "schema_version",
            "generated_at_utc",
            "corpus_id",
            "stage_id",
            "scenario_id",
            "sample_scope",
            "tool_kind",
            "platform",
            "tools",
            "threads",
            "jobs",
            "sample_jobs",
            "sample_limit",
            "dry_run",
            "config_path",
            "publication_config_path",
            "repo_root",
            "corpus_root",
            "out_root",
            "stage_args",
            "samples_total",
            "samples_failed",
            "runs",
        ],
        "manifest",
    )?;
    let failures = manifest.runs.iter().filter(|run| run.exit_code != 0).count();
    if manifest.samples_failed != failures {
        return invalid("FASTQ corpus benchmark samples_failed does not match exit codes");
    }

    let mut samples = BTreeSet::new();
    for run in &manifest.runs {
        for (value, field) in [
            (&run.sample_id, "runs[].sample_id"),
            (&run.r1, "runs[].r1"),
            (&run.layout, "runs[].layout"),
            (&run.status, "runs[].status"),
            (&run.report_json, "runs[].report_json"),
        ] {
            require_text(value, field)?;
        }
        if !samples.insert(run.sample_id.as_str()) {
            return invalid(format!("FASTQ corpus benchmark repeats sample `{}`", run.sample_id));
        }
        let allowed_status = matches!(
            run.status.as_str(),
            "completed" | "failed" | "dry_run" | "skipped_existing_report"
        );
        if !allowed_status {
            return invalid(format!(
                "FASTQ corpus benchmark sample `{}` has unknown status `{}`",
                run.sample_id, run.status
            ));
        }
        if (run.command.is_empty() && run.status != "skipped_existing_report")
            || run.command.iter().any(String::is_empty)
        {
            return invalid(format!(
                "FASTQ corpus benchmark sample `{}` has an empty command",
                run.sample_id
            ));
        }
        if (run.exit_code != 0) != (run.status == "failed") {
            return invalid(format!(
                "FASTQ corpus benchmark sample `{}` status contradicts its exit code",
                run.sample_id
            ));
        }
        if manifest.dry_run != (run.status == "dry_run") {
            return invalid(format!(
                "FASTQ corpus benchmark sample `{}` dry-run status contradicts the manifest",
                run.sample_id
            ));
        }
        reject_reserved_extra_fields(
            run.extra_fields.keys().map(String::as_str),
            &["sample_id", "r1", "r2", "layout", "status", "exit_code", "command", "report_json"],
            "sample run",
        )?;
    }
    Ok(())
}

fn reject_reserved_extra_fields<'a>(
    fields: impl Iterator<Item = &'a str>,
    reserved: &[&str],
    context: &str,
) -> Result<(), BenchError> {
    for field in fields {
        if field.trim().is_empty() || reserved.contains(&field) {
            return invalid(format!(
                "FASTQ corpus benchmark {context} has invalid extra field `{field}`"
            ));
        }
    }
    Ok(())
}

fn require_text(value: &str, field: &str) -> Result<(), BenchError> {
    if value.trim().is_empty() {
        return invalid(format!("FASTQ corpus benchmark requires non-empty {field}"));
    }
    Ok(())
}

fn invalid<T>(message: impl Into<String>) -> Result<T, BenchError> {
    Err(BenchError::InvalidPolicy(message.into()))
}
