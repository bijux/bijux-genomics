use std::path::{Path, PathBuf};

use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    benchmark_corpus_spec_path_from_config, render_benchmark_config_section_json,
};

use super::load_benchmark_config;

pub(crate) fn benchmark_corpus_spec_path(
    cwd: &Path,
    explicit_path: Option<&Path>,
    corpus_id: &str,
) -> Result<PathBuf> {
    let config = load_benchmark_config(cwd, explicit_path)?;
    benchmark_corpus_spec_path_from_config(cwd, &config, corpus_id)
}

pub(crate) fn print_benchmark_config_json(
    cwd: &Path,
    args: &crate::commands::cli::BenchConfigJsonArgs,
) -> Result<()> {
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    println!("{}", render_benchmark_config_section_json(&config, &args.section)?);
    Ok(())
}
