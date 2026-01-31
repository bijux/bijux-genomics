//! Owner: bijux-analyze
//! Load step for analyze pipeline.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_core::FactsRowV1;

use crate::load::{load_facts, load_facts_parquet, load_run_summary, AnalyzeError};
use crate::AnalyzeSources;

#[derive(Debug)]
pub(crate) struct LoadedInputs {
    pub(crate) facts: Vec<FactsRowV1>,
    pub(crate) base_dir: PathBuf,
}

pub(crate) fn load_inputs(sources: &AnalyzeSources) -> Result<LoadedInputs> {
    let facts = match sources {
        AnalyzeSources::FactsJsonl(path) => load_facts(path).map_err(map_load_error)?,
        AnalyzeSources::FactsParquet(path) => load_facts_parquet(path).map_err(map_load_error)?,
        AnalyzeSources::RunSummaryJson(path) => {
            let _summary = load_run_summary(path).map_err(map_load_error)?;
            return Err(anyhow!(
                "run summary input does not include facts: {}",
                path.display()
            ));
        }
        AnalyzeSources::RunIndexSqlite(path) => {
            return Err(anyhow!(
                "run index sqlite not yet wired for analyze_run: {}",
                path.display()
            ));
        }
    };

    Ok(LoadedInputs {
        facts,
        base_dir: base_dir_for_sources(sources),
    })
}

fn base_dir_for_sources(sources: &AnalyzeSources) -> PathBuf {
    let path = match sources {
        AnalyzeSources::FactsJsonl(path)
        | AnalyzeSources::FactsParquet(path)
        | AnalyzeSources::RunSummaryJson(path)
        | AnalyzeSources::RunIndexSqlite(path) => path,
    };
    path.parent()
        .map_or_else(|| PathBuf::from("."), Path::to_path_buf)
}

fn map_load_error(err: AnalyzeError) -> anyhow::Error {
    anyhow!(err)
}
