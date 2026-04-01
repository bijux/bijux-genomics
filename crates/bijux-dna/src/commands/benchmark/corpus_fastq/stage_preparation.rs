use std::collections::BTreeMap;

use anyhow::Result;

use crate::commands::benchmark_stage_catalog::corpus_fastq_stage_catalog_entry;

#[derive(Debug, Clone, Copy)]
pub(super) struct StageCommandSpec {
    pub(super) bench_subcommand: &'static str,
    pub(super) report_dir: &'static str,
    pub(super) strict_resume_report: bool,
}

#[derive(Debug, Default)]
pub(super) struct StageSamplePreparation {
    pub(super) extra_stage_args: Vec<String>,
    pub(super) run_extra_fields: BTreeMap<String, serde_json::Value>,
}

pub(super) fn stage_command_spec(stage_id: &str) -> Result<StageCommandSpec> {
    let entry = corpus_fastq_stage_catalog_entry(stage_id)?;
    Ok(StageCommandSpec {
        bench_subcommand: entry.bench_subcommand,
        report_dir: entry.report_dir,
        strict_resume_report: entry.strict_resume_report,
    })
}
