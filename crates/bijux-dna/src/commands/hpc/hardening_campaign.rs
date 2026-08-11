use anyhow::Result;

pub use bijux_dna_bench_model::{
    HardeningBenchmarkCampaignReport, HardeningBenchmarkCampaignSummary,
    HardeningBenchmarkGoalEntry, HardeningMatrixRowRef, HardeningScenario,
};

use crate::commands::cli::{BenchmarkMatrixArgs, HardeningBenchmarkCampaignArgs};
use crate::commands::hpc::{
    appraise_matrix_report, benchmark_matrix, hardening_queue_from_appraisal,
};

pub fn hardening_benchmark_campaign(
    args: &HardeningBenchmarkCampaignArgs,
) -> Result<HardeningBenchmarkCampaignReport> {
    let matrix = benchmark_matrix(&BenchmarkMatrixArgs {
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_policies: args.user_policies.clone(),
        domain: "all".to_string(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })?;
    let appraisal = appraise_matrix_report(matrix.clone())?;
    let queue = hardening_queue_from_appraisal(appraisal.clone())?;
    let report = bijux_dna_bench::derive_hardening_benchmark_campaign(
        matrix,
        &appraisal,
        &queue,
        args.goals.as_deref(),
    )?;
    if let Some(path) = &args.out {
        bijux_dna_bench::publish_hardening_benchmark_campaign_report(path, &report)?;
    }
    Ok(report)
}
