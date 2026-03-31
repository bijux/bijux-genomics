use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use super::matching::{scalar_matches, BenchQueryContextMatch};

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
    pub const PARAMETERS_KEY: &'static str = "bench_query_context";

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

    #[must_use]
    pub fn match_against_stored(&self, stored: &Self) -> BenchQueryContextMatch {
        if !scalar_matches(self.params_hash.as_ref(), stored.params_hash.as_ref())
            || !scalar_matches(self.image_digest.as_ref(), stored.image_digest.as_ref())
            || !scalar_matches(
                self.stage_contract_hash.as_ref(),
                stored.stage_contract_hash.as_ref(),
            )
            || !scalar_matches(self.reference_hash.as_ref(), stored.reference_hash.as_ref())
            || !scalar_matches(self.database_hash.as_ref(), stored.database_hash.as_ref())
            || !scalar_matches(self.lineage_hash.as_ref(), stored.lineage_hash.as_ref())
        {
            return BenchQueryContextMatch::NoMatch;
        }
        if !self.bank_hashes.is_empty() && self.bank_hashes != stored.bank_hashes {
            return BenchQueryContextMatch::NoMatch;
        }
        BenchQueryContextMatch::Exact
    }
}
