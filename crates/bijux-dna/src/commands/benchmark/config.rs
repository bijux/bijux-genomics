//! Thin CLI adapter for benchmark configuration admission owned by bijux-dna-bench.

use std::path::Path;

use anyhow::Result;
use bijux_dna_api::v1::api::bench::{
    validate_benchmark_config_contract, BenchmarkConfigValidationRequest,
};

use crate::commands::benchmark_workspace::{benchmark_config_path, load_benchmark_config};
use crate::commands::cli::BenchConfigValidateArgs;

pub(crate) fn validate_benchmark_config(cwd: &Path, args: &BenchConfigValidateArgs) -> Result<()> {
    let path = benchmark_config_path(cwd, args.config.as_deref());
    let config = load_benchmark_config(cwd, args.config.as_deref())?;
    let summary = validate_benchmark_config_contract(&BenchmarkConfigValidationRequest {
        cwd,
        config_path: &path,
        config: &config,
        check_paths: args.check_paths,
    })?;

    println!("benchmark_config={}", path.display());
    println!("workspace=ok");
    println!("publication=ok");
    println!("corpora={}", summary.corpus_count);
    println!("paths_checked={}", summary.paths_checked);
    Ok(())
}
