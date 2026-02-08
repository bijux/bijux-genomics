//! Owner: bijux-dna-analyze
//! Validate step for analyze pipeline.

use anyhow::{anyhow, Result};
use bijux_dna_runtime::FactsRowV1;

use crate::model::FactTable;

use super::load_step::LoadedInputs;

#[derive(Debug)]
pub(crate) struct ValidatedFacts {
    pub(crate) facts: FactTable,
    pub(crate) base_dir: std::path::PathBuf,
}

pub(crate) fn validate_inputs(loaded: LoadedInputs) -> Result<ValidatedFacts> {
    let facts = validate_facts(&loaded.facts)?;
    let normalized = normalize_facts(facts);
    let aggregated = aggregate_facts(normalized);

    Ok(ValidatedFacts {
        facts: aggregated,
        base_dir: loaded.base_dir,
    })
}

fn validate_facts(facts: &[FactsRowV1]) -> Result<FactTable> {
    FactTable::from_facts(facts).map_err(|err| anyhow!(err))
}

fn normalize_facts(facts: FactTable) -> FactTable {
    facts
}

fn aggregate_facts(facts: FactTable) -> FactTable {
    facts
}
