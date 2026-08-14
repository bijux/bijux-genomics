use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    build_vcf_stage_matrix_rows, render_vcf_stage_matrix, validate_vcf_stage_matrix,
    LocalVcfStageMatrixConfig, VcfStageMatrixRow, DEFAULT_VCF_STAGE_MATRIX_PATH,
};

use crate::commands::cli::{parse, render};

pub(crate) fn run_render_vcf_stage_matrix(
    args: &parse::BenchLocalRenderVcfStageMatrixArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_stage_matrix(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn run_validate_vcf_stage_matrix(
    repo_root: &Path,
    args: &parse::BenchValidateMatrixArgs,
) -> Result<()> {
    let report = validate_vcf_stage_matrix(
        repo_root,
        args.matrix.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_MATRIX_PATH)),
        args.strict,
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{} {} rows validated", report.matrix_path, report.row_count);
    }
    Ok(())
}
