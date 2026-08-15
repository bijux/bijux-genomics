//! Owner: bijux-dna-bench-model
//! Validation for domain-neutral production data source authorities.

use std::collections::BTreeSet;

use crate::contract::materialization::{
    validate_corpus_evaluation_design, validate_https, validate_identifier, validate_nonempty,
    validate_nonempty_list, validate_source_policy,
};
use crate::diagnostics::BenchError;
use crate::model::BenchmarkDataSourceSpec;

use super::BENCHMARK_DATA_SOURCE_SCHEMA_V1;

/// Validate source identity, access policy, projections, selection scope, and truth authority.
///
/// # Errors
/// Returns an error when a source-defined corpus is incomplete or internally inconsistent.
pub fn validate_benchmark_data_source_spec(
    spec: &BenchmarkDataSourceSpec,
) -> Result<(), BenchError> {
    if spec.schema_version != BENCHMARK_DATA_SOURCE_SCHEMA_V1 {
        return Err(invalid("benchmark data source schema mismatch"));
    }
    validate_identifier(&spec.portfolio_id, "portfolio")?;
    validate_source_policy(&spec.policy)?;
    validate_corpus_evaluation_design(&spec.evaluation)?;
    validate_nonempty(&spec.description, "data source description")?;
    validate_ordered_values(&spec.data_types, "data source type")?;
    if spec.target_sample_count == 0 || spec.collections.is_empty() {
        return Err(invalid("benchmark data source selection is empty"));
    }

    let declared_types = spec.data_types.iter().map(String::as_str).collect::<BTreeSet<_>>();
    let mut collection_ids = BTreeSet::new();
    let mut explicit_selected_accessions = BTreeSet::new();
    let mut previous_id = None;
    let mut planned_samples = 0_u64;
    for collection in &spec.collections {
        validate_identifier(&collection.id, "source collection")?;
        if !collection_ids.insert(collection.id.as_str())
            || previous_id.is_some_and(|previous| previous >= collection.id.as_str())
        {
            return Err(invalid("benchmark data source collections are not strictly ordered"));
        }
        previous_id = Some(collection.id.as_str());
        for (value, label) in [
            (&collection.authority, "source collection authority"),
            (&collection.accession, "source collection accession"),
            (&collection.selection_scope, "source collection selection scope"),
            (&collection.truth_basis, "source collection truth basis"),
        ] {
            validate_nonempty(value, label)?;
        }
        validate_https(&collection.source_url, "source collection record")?;
        validate_https(&collection.access_terms_url, "source collection access terms")?;
        validate_ordered_values(&collection.data_types, "source collection data type")?;
        if collection
            .data_types
            .iter()
            .any(|data_type| !declared_types.contains(data_type.as_str()))
        {
            return Err(invalid(
                "source collection data type is absent from the corpus declaration",
            ));
        }
        if collection.planned_sample_count == Some(0) {
            return Err(invalid("source collection planned sample count is zero"));
        }
        if !collection.selected_accessions.is_empty() {
            validate_ordered_values(&collection.selected_accessions, "selected source accession")?;
            let explicit_count = u64::try_from(collection.selected_accessions.len())
                .map_err(|_| invalid("selected source accession count overflows"))?;
            if collection.planned_sample_count != Some(explicit_count) {
                return Err(invalid(
                    "selected source accessions differ from the planned sample count",
                ));
            }
            if collection
                .selected_accessions
                .iter()
                .any(|accession| !explicit_selected_accessions.insert(accession.as_str()))
            {
                return Err(invalid("selected source accession is declared more than once"));
            }
        }
        validate_nonempty_list(&collection.known_limitations, "source collection limitation")?;
        if let Some(collection_samples) = collection.planned_sample_count {
            planned_samples = planned_samples
                .checked_add(collection_samples)
                .ok_or_else(|| invalid("source collection sample count overflows"))?;
        }
    }
    if planned_samples < spec.target_sample_count {
        return Err(invalid("source collections do not satisfy the target sample count"));
    }
    Ok(())
}

fn validate_ordered_values(values: &[String], label: &str) -> Result<(), BenchError> {
    validate_nonempty_list(values, label)?;
    if values.windows(2).any(|pair| pair[0] >= pair[1]) {
        return Err(invalid("benchmark data source values are not strictly ordered"));
    }
    Ok(())
}

fn invalid(message: &str) -> BenchError {
    BenchError::InvalidPolicy(message.to_string())
}
