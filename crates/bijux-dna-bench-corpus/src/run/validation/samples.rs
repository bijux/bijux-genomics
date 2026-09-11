use super::super::{BenchmarkCorpusRunManifest, CorpusRunError};
use super::fields::{invalid, reject_reserved_extra_fields, require_text};
use std::collections::BTreeSet;
pub(super) fn validate_sample_runs(
    manifest: &BenchmarkCorpusRunManifest,
) -> Result<(), CorpusRunError> {
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
