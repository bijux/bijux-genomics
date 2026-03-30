use std::collections::BTreeMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::{Map as JsonMap, Value as JsonValue};

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BenchQueryContextMatch {
    Exact,
    LegacyCompatible,
    NoMatch,
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
    pub fn embed_in_parameters(&self, parameters: &JsonValue) -> JsonValue {
        if self.is_empty() {
            return parameters.clone();
        }
        let mut object = match parameters {
            JsonValue::Object(map) => map.clone(),
            _ => JsonMap::new(),
        };
        object.insert(
            Self::PARAMETERS_KEY.to_string(),
            serde_json::to_value(self).unwrap_or(JsonValue::Null),
        );
        JsonValue::Object(object)
    }

    #[must_use]
    pub fn from_parameters(parameters: &JsonValue) -> Option<Self> {
        let object = parameters.as_object()?;
        let encoded = object.get(Self::PARAMETERS_KEY)?.clone();
        serde_json::from_value(encoded).ok()
    }

    #[must_use]
    pub fn match_against_parameters(&self, parameters: &JsonValue) -> BenchQueryContextMatch {
        match Self::from_parameters(parameters) {
            Some(stored) => self.match_against_stored(&stored),
            None => BenchQueryContextMatch::LegacyCompatible,
        }
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

fn scalar_matches(requested: Option<&String>, stored: Option<&String>) -> bool {
    match requested {
        Some(requested_value) => stored == Some(requested_value),
        None => true,
    }
}

/// # Errors
/// Returns an error if the governed stage contract hash cannot be computed.
pub fn governed_stage_bench_query_context(stage_id: &str) -> Result<BenchQueryContext> {
    let mut context = BenchQueryContext::new();
    if let Some(contract_hash) = crate::stage_contract_hash(stage_id) {
        context = context.with_stage_contract_hash(contract_hash?);
    }
    Ok(context)
}

#[cfg(test)]
mod tests {
    use super::{BenchQueryContext, BenchQueryContextMatch};

    #[test]
    fn bench_query_context_round_trips_through_parameters_json() {
        let context = BenchQueryContext::new()
            .with_stage_contract_hash("contract-a")
            .with_reference_hash("reference-a")
            .with_bank_hash("adapter_bank", "bank-a")
            .with_lineage_hash("fastq.validate_reads=fastqvalidator");
        let parameters = context.embed_in_parameters(&serde_json::json!({
            "min_length": 30
        }));

        assert_eq!(
            BenchQueryContext::from_parameters(&parameters),
            Some(context)
        );
    }

    #[test]
    fn bench_query_context_treats_missing_embedded_metadata_as_legacy_compatible() {
        let context = BenchQueryContext::new().with_stage_contract_hash("contract-a");

        assert_eq!(
            context.match_against_parameters(&serde_json::json!({"min_length": 30})),
            BenchQueryContextMatch::LegacyCompatible
        );
    }

    #[test]
    fn bench_query_context_rejects_conflicting_embedded_metadata() {
        let requested = BenchQueryContext::new()
            .with_stage_contract_hash("contract-a")
            .with_bank_hash("adapter_bank", "bank-a");
        let stored = BenchQueryContext::new()
            .with_stage_contract_hash("contract-b")
            .with_bank_hash("adapter_bank", "bank-b");

        assert_eq!(
            requested.match_against_stored(&stored),
            BenchQueryContextMatch::NoMatch
        );
    }
}
