use serde_json::{Map as JsonMap, Value as JsonValue};

use super::{BenchQueryContext, BenchQueryContextMatch};

impl BenchQueryContext {
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
}

#[cfg(test)]
mod tests {
    use super::super::{BenchQueryContext, BenchQueryContextMatch};

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

        assert_eq!(BenchQueryContext::from_parameters(&parameters), Some(context));
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

        assert_eq!(requested.match_against_stored(&stored), BenchQueryContextMatch::NoMatch);
    }
}
