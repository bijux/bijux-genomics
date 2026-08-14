use std::path::PathBuf;

use anyhow::{Context, Result};
pub(crate) use bijux_dna_api::v1::api::bench::{
    build_vcf_stage_catalog_rows, render_vcf_stage_catalog, VcfStageCatalogRow,
    DEFAULT_VCF_STAGE_CATALOG_PATH,
};

use crate::commands::cli::{parse, render};

pub(crate) fn run_render_vcf_stage_catalog(
    args: &parse::BenchLocalRenderVcfStageCatalogArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_stage_catalog(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_PATH)),
    )?;
    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}
