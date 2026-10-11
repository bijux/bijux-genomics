use anyhow::{anyhow, Result};
pub use bijux_dna_api::v1::api::bench::{
    appraise_matrix_report, hardening_queue_from_appraisal, AppraisalFinding, AppraisalReport,
    AppraisalSummary, HardeningQueueEntry, HardeningQueueReport,
};

use crate::commands::cli::{AppraiseMatrixArgs, BenchmarkMatrixArgs, HardeningQueueArgs};
use crate::commands::hpc::benchmark_matrix;

fn matrix_from_args(
    args: &AppraiseMatrixArgs,
) -> Result<bijux_dna_api::v1::api::bench::BenchmarkMatrixReport> {
    if let Some(path) = &args.matrix {
        return bijux_dna_api::v1::api::bench::load_benchmark_matrix_report(path);
    }
    let Some(config) = args.config.clone() else {
        return Err(anyhow!("appraise-matrix requires either --matrix or --config"));
    };
    benchmark_matrix(&BenchmarkMatrixArgs {
        config,
        env_file: args.env_file.clone(),
        user_policies: args.user_policies.clone(),
        domain: args.domain.clone(),
        out: None,
        fail_on_refuse: false,
        json: false,
    })
}

pub fn appraise_matrix(args: &AppraiseMatrixArgs) -> Result<AppraisalReport> {
    let report = appraise_matrix_report(matrix_from_args(args)?)?;
    if let Some(path) = &args.out {
        bijux_dna_api::v1::api::bench::publish_appraisal_report(path, &report)?;
    }
    Ok(report)
}

fn appraisal_from_args(args: &HardeningQueueArgs) -> Result<AppraisalReport> {
    if let Some(path) = &args.appraisal {
        return bijux_dna_api::v1::api::bench::load_appraisal_report(path);
    }
    appraise_matrix(&AppraiseMatrixArgs {
        matrix: args.matrix.clone(),
        config: args.config.clone(),
        env_file: args.env_file.clone(),
        user_policies: args.user_policies.clone(),
        domain: args.domain.clone(),
        out: None,
        json: false,
    })
}

pub fn generate_hardening_queue(args: &HardeningQueueArgs) -> Result<HardeningQueueReport> {
    let report = hardening_queue_from_appraisal(appraisal_from_args(args)?)?;
    if let Some(path) = &args.out {
        bijux_dna_api::v1::api::bench::publish_hardening_queue_report(path, &report)?;
    }
    Ok(report)
}
