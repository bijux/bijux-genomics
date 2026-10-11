use super::super::{time::validate_utc_timestamp, BENCHMARK_LOCAL_CORPUS_RUN_SCHEMA_V1};
use super::super::{BenchmarkCorpusRunManifest, CorpusRunError};
use super::fields::{invalid, reject_reserved_extra_fields, require_text};
use std::collections::BTreeSet;
pub(super) fn validate_manifest_contract(
    manifest: &BenchmarkCorpusRunManifest,
) -> Result<(), CorpusRunError> {
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
    Ok(())
}
