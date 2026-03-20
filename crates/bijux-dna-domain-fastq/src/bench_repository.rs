use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

use bijux_dna_core::contract::BenchResultRecord;
use bijux_dna_core::ids::StageId;

use crate::BenchCorpus;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BenchQueryContext {
    pub params_hash: Option<String>,
    pub image_digest: Option<String>,
    pub stage_contract_hash: Option<String>,
    pub reference_hash: Option<String>,
    pub database_hash: Option<String>,
    #[serde(default)]
    pub bank_hashes: BTreeMap<String, String>,
    pub lineage_hash: Option<String>,
}

impl BenchQueryContext {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub fn with_params_hash(mut self, params_hash: impl Into<String>) -> Self {
        self.params_hash = Some(params_hash.into());
        self
    }

    #[must_use]
    pub fn with_image_digest(mut self, image_digest: impl Into<String>) -> Self {
        self.image_digest = Some(image_digest.into());
        self
    }

    #[must_use]
    pub fn with_stage_contract_hash(mut self, stage_contract_hash: impl Into<String>) -> Self {
        self.stage_contract_hash = Some(stage_contract_hash.into());
        self
    }

    #[must_use]
    pub fn with_reference_hash(mut self, reference_hash: impl Into<String>) -> Self {
        self.reference_hash = Some(reference_hash.into());
        self
    }

    #[must_use]
    pub fn with_database_hash(mut self, database_hash: impl Into<String>) -> Self {
        self.database_hash = Some(database_hash.into());
        self
    }

    #[must_use]
    pub fn with_bank_hash(
        mut self,
        bank_id: impl Into<String>,
        bank_hash: impl Into<String>,
    ) -> Self {
        self.bank_hashes.insert(bank_id.into(), bank_hash.into());
        self
    }

    #[must_use]
    pub fn with_lineage_hash(mut self, lineage_hash: impl Into<String>) -> Self {
        self.lineage_hash = Some(lineage_hash.into());
        self
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.params_hash.is_none()
            && self.image_digest.is_none()
            && self.stage_contract_hash.is_none()
            && self.reference_hash.is_none()
            && self.database_hash.is_none()
            && self.bank_hashes.is_empty()
            && self.lineage_hash.is_none()
    }
}

pub trait BenchResultsRepository {
    /// # Errors
    /// Returns an error if benchmark records cannot be loaded for the request.
    fn bench_results(
        &self,
        stage: &StageId,
        tool: &str,
        corpus: &BenchCorpus,
        context: &BenchQueryContext,
    ) -> Result<Vec<BenchResultRecord>>;
}
