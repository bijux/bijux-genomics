//! Validate backend identity and masked truth imputation evidence.
use crate::imputation::model::EXPECTED_IMPUTED_GENOTYPES;
use crate::imputation::model::EXPECTED_LOW_CONFIDENCE_COUNT;
use crate::imputation::model::EXPECTED_MASKED_TRUTH_MATCH_COUNT;
use crate::imputation::model::EXPECTED_MASKED_TRUTH_SITE_COUNT;
use crate::imputation::model::EXPECTED_MISSING_AFTER;
use crate::imputation::model::EXPECTED_MISSING_BEFORE;
use crate::imputation::model::EXPECTED_UNRESOLVED_COUNT;
use crate::imputation::observations::reports::read_json;
use crate::imputation::observations::reports::read_u64;
use anyhow::bail;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::ImputeBackend;
use std::collections::BTreeMap;
use std::path::Path;

pub(crate) struct ImputationQuality {
    pub(crate) missing_before: u64,
    pub(crate) missing_after: u64,
    pub(crate) imputed_genotypes: u64,
    pub(crate) low_confidence_count: u64,
    pub(crate) masked_truth_site_count: u64,
    pub(crate) masked_truth_match_count: u64,
    pub(crate) unresolved_count: u64,
    pub(crate) not_imputable_reasons: BTreeMap<String, u64>,
}

pub(crate) fn observe_quality(
    imputation_qc_path: &Path,
    backend: ImputeBackend,
) -> Result<ImputationQuality> {
    let imputation_qc = read_json(imputation_qc_path)?;
    if imputation_qc.get("backend").and_then(serde_json::Value::as_str) != Some(backend.as_str()) {
        bail!("imputation QC report drifted away from selected `{}` backend", backend.as_str());
    }
    if imputation_qc.pointer("/concordance/truth_provided").and_then(serde_json::Value::as_bool)
        != Some(true)
    {
        bail!("imputation QC report drifted away from masked truth validation");
    }
    let missing_before = read_u64(&imputation_qc, "/missing_genotypes_before")?;
    let missing_after = read_u64(&imputation_qc, "/missing_genotypes_after")?;
    let imputed_genotypes = read_u64(&imputation_qc, "/imputed_genotypes")?;
    let low_confidence_count = read_u64(&imputation_qc, "/low_confidence_count")?;
    let masked_truth_site_count = read_u64(&imputation_qc, "/concordance/masked_truth_site_count")?;
    let masked_truth_match_count = read_u64(&imputation_qc, "/concordance/imputed_match_count")?;
    let unresolved_count = read_u64(&imputation_qc, "/concordance/unresolved_count")?;
    let not_imputable_reasons = imputation_qc
        .get("not_imputable_reasons")
        .and_then(serde_json::Value::as_object)
        .map(|rows| {
            rows.iter()
                .filter_map(|(key, value)| value.as_u64().map(|count| (key.clone(), count)))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    if missing_before != EXPECTED_MISSING_BEFORE
        || missing_after != EXPECTED_MISSING_AFTER
        || imputed_genotypes != EXPECTED_IMPUTED_GENOTYPES
        || low_confidence_count != EXPECTED_LOW_CONFIDENCE_COUNT
        || masked_truth_site_count != EXPECTED_MASKED_TRUTH_SITE_COUNT
        || masked_truth_match_count != EXPECTED_MASKED_TRUTH_MATCH_COUNT
        || unresolved_count != EXPECTED_UNRESOLVED_COUNT
    {
        bail!(
            "governed VCF impute smoke drifted from masked-truth contract: before={missing_before} after={missing_after} imputed={imputed_genotypes} low_confidence={low_confidence_count} masked_sites={masked_truth_site_count} matches={masked_truth_match_count} unresolved={unresolved_count}"
        );
    }
    if !not_imputable_reasons.is_empty() {
        bail!(
            "governed VCF impute smoke expected zero unresolved reasons, found {not_imputable_reasons:?}"
        );
    }

    Ok(ImputationQuality {
        missing_before,
        missing_after,
        imputed_genotypes,
        low_confidence_count,
        masked_truth_site_count,
        masked_truth_match_count,
        unresolved_count,
        not_imputable_reasons,
    })
}
