use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};

use super::{config_paths::absolutize, load_benchmark_config, BenchmarkConfig};

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
    match args.section.as_str() {
        "full" => println!("{}", serde_json::to_string_pretty(&config)?),
        "workspace" => println!("{}", serde_json::to_string_pretty(&config.workspace)?),
        "publication" => println!("{}", serde_json::to_string_pretty(&config.publication)?),
        "corpora" => println!("{}", serde_json::to_string_pretty(&config.corpora)?),
        "stage_inputs" => println!("{}", serde_json::to_string_pretty(&config.stage_inputs)?),
        other => {
            return Err(anyhow!(
                "unsupported benchmark config section `{other}`; expected one of: full, workspace, publication, corpora, stage_inputs"
            ));
        }
    }
    Ok(())
}

fn benchmark_corpus_spec_path_from_config(
    cwd: &Path,
    config: &BenchmarkConfig,
    corpus_id: &str,
) -> Result<PathBuf> {
    if let Some(path) = config.corpora.get(corpus_id).and_then(|row| row.spec_path.as_deref()) {
        return Ok(absolutize(cwd, Path::new(path)));
    }
    Err(anyhow!("benchmark config is missing corpora.{corpus_id}.spec_path"))
}
