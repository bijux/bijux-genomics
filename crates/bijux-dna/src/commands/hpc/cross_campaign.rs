use anyhow::Result;

pub use bijux_dna_api::v1::api::bench::{
    CrossBenchmarkCampaignReport, CrossBenchmarkCampaignSummary, CrossBenchmarkGoalEntry,
    CrossMatrixRowRef, CrossScenario,
};

use crate::commands::cli::{BenchmarkMatrixArgs, CrossBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal,
};

pub fn cross_benchmark_campaign(
    args: &CrossBenchmarkCampaignArgs,
) -> Result<CrossBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_policies: args.user_policies.clone(),
        domain: "cross".to_string(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })?;
    let appraisal = appraise_matrix_report(matrix.clone())?;
    let queue = hardening_queue_from_appraisal(appraisal.clone())?;
    let report = bijux_dna_api::v1::api::bench::derive_cross_benchmark_campaign(
        matrix,
        &appraisal,
        &queue,
        args.goals.as_deref(),
    )?;
    if let Some(path) = &args.out {
        bijux_dna_api::v1::api::bench::publish_cross_benchmark_campaign_report(path, &report)?;
    }
    Ok(report)
}
