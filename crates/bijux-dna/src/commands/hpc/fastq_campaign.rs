use anyhow::Result;

pub use bijux_dna_api::v1::api::bench::{
    FastqBenchmarkCampaignReport, FastqBenchmarkCampaignSummary, FastqBenchmarkGoalEntry,
    FastqMatrixRowRef, FastqScenario,
};

use crate::commands::cli::{BenchmarkMatrixArgs, FastqBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal,
};

pub fn fastq_benchmark_campaign(
    args: &FastqBenchmarkCampaignArgs,
) -> Result<FastqBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_policies: args.user_policies.clone(),
        domain: "fastq".to_string(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })?;
    let appraisal = appraise_matrix_report(matrix.clone())?;
    let queue = hardening_queue_from_appraisal(appraisal.clone())?;
    let report = bijux_dna_api::v1::api::bench::derive_fastq_benchmark_campaign(
        matrix,
        &appraisal,
        &queue,
        args.goals.as_deref(),
    )?;
    if let Some(path) = &args.out {
        bijux_dna_api::v1::api::bench::publish_fastq_benchmark_campaign_report(path, &report)?;
    }
    Ok(report)
}
