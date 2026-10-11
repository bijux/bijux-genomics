//! Owner: bijux-dna-bench-model
//! Validation for governed benchmark data-portfolio audits.

use std::collections::{BTreeMap, BTreeSet};

use crate::model::{
    BenchmarkCorpusClass, BenchmarkCorpusDataAudit, BenchmarkCorpusStatus,
    BenchmarkDataPortfolioAudit, BenchmarkEvidenceScope, BenchmarkGovernedDataBinding,
};
use crate::BenchError;

use super::BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2;

/// Validate identities, totals, bindings, projections, and publishable contamination closure.
///
/// # Errors
/// Returns [`BenchError`] when the audit is internally inconsistent or promotes incomplete data.
pub fn validate_benchmark_data_portfolio_audit(
    audit: &BenchmarkDataPortfolioAudit,
) -> Result<(), BenchError> {
    validate_authorities(audit)?;
    let (totals, bound_ids) = derive_corpus_totals(audit)?;
    validate_unbound_assets(audit, &bound_ids)?;
    validate_contamination(audit)?;
    validate_summary(audit, &totals)
}

#[derive(Default)]
struct AuditTotals {
    admitted: u64,
    source_defined: u64,
    missing: u64,
    ready: u64,
    present: u64,
    absent: u64,
    files: u64,
    bytes: u64,
    unknown_storage: u64,
    unknown_compute: u64,
}

fn validate_authorities(audit: &BenchmarkDataPortfolioAudit) -> Result<(), BenchError> {
    if audit.schema_version != BENCHMARK_DATA_PORTFOLIO_AUDIT_SCHEMA_V2 {
        return Err(BenchError::InvalidPolicy("unsupported data-portfolio audit schema".into()));
    }
    validate_hex(&audit.source_commit, 40, "source commit")?;
    validate_hex(&audit.portfolio_sha256, 64, "portfolio digest")?;
    validate_hex(&audit.data_index_sha256, 64, "data-index digest")?;
    if audit.portfolio_path.is_empty() || audit.data_index_path.is_empty() {
        return Err(BenchError::InvalidPolicy("data-portfolio authority path is empty".into()));
    }
    Ok(())
}

fn derive_corpus_totals(
    audit: &BenchmarkDataPortfolioAudit,
) -> Result<(AuditTotals, BTreeSet<String>), BenchError> {
    let mut corpus_ids = BTreeSet::new();
    let mut global_bindings = BTreeMap::new();
    let mut totals = AuditTotals::default();

    for corpus in &audit.corpora {
        if corpus.corpus_id.is_empty() || !corpus_ids.insert(corpus.corpus_id.as_str()) {
            return Err(BenchError::InvalidPolicy("duplicate or empty audited corpus".into()));
        }
        match corpus.status {
            BenchmarkCorpusStatus::Admitted => totals.admitted += 1,
            BenchmarkCorpusStatus::SourceDefined => totals.source_defined += 1,
            BenchmarkCorpusStatus::Missing => totals.missing += 1,
        }
        validate_corpus(corpus, &mut totals, &mut global_bindings)?;
    }
    Ok((totals, global_bindings.into_keys().collect()))
}

fn validate_corpus(
    corpus: &BenchmarkCorpusDataAudit,
    totals: &mut AuditTotals,
    global_bindings: &mut BTreeMap<String, BenchmarkGovernedDataBinding>,
) -> Result<(), BenchError> {
    if corpus.minimum_sample_count == 0
        || corpus.data_types.is_empty()
        || corpus.truth_roles.is_empty()
        || corpus.truth_strategy.is_empty()
    {
        return Err(BenchError::InvalidPolicy("audited corpus scientific closure is empty".into()));
    }
    let valid_class = matches!(
        (corpus.class, corpus.evidence_scope),
        (BenchmarkCorpusClass::ProductionReal, BenchmarkEvidenceScope::Comparative)
            | (BenchmarkCorpusClass::QualificationReal, BenchmarkEvidenceScope::Qualification)
    );
    if !valid_class {
        return Err(BenchError::InvalidPolicy(
            "audited corpus class is not publishable real data".into(),
        ));
    }
    validate_source_specs(corpus)?;
    if corpus.publishable_data_ready {
        totals.ready += 1;
        if corpus.status != BenchmarkCorpusStatus::Admitted || !corpus.blockers.is_empty() {
            return Err(BenchError::InvalidPolicy(
                "incomplete corpus is marked publishable-data ready".into(),
            ));
        }
    } else if corpus.blockers.is_empty() {
        return Err(BenchError::InvalidPolicy("blocked corpus lacks a blocker".into()));
    }
    let mut corpus_bindings = BTreeSet::new();
    let mut corpus_bytes = 0_u64;
    for binding in &corpus.catalog_bindings {
        validate_binding(binding)?;
        if !corpus_bindings.insert(binding.catalog_id.as_str()) {
            return Err(BenchError::InvalidPolicy("duplicate corpus catalog binding".into()));
        }
        let newly_bound = match global_bindings.get(&binding.catalog_id) {
            Some(existing) if existing != binding => {
                return Err(BenchError::InvalidPolicy(
                    "reused catalog binding carries inconsistent evidence".into(),
                ));
            }
            Some(_) => false,
            None => {
                global_bindings.insert(binding.catalog_id.clone(), binding.clone());
                true
            }
        };
        if binding.present && newly_bound {
            totals.present += 1;
            totals.files =
                totals.files.checked_add(binding.verified_file_count).ok_or_else(overflow)?;
            totals.bytes =
                totals.bytes.checked_add(binding.verified_size_bytes).ok_or_else(overflow)?;
        }
        if binding.present {
            corpus_bytes =
                corpus_bytes.checked_add(binding.verified_size_bytes).ok_or_else(overflow)?;
        } else if newly_bound {
            totals.absent += 1;
        }
    }
    if corpus.verified_storage_bytes != corpus_bytes {
        return Err(BenchError::InvalidPolicy("corpus verified byte total drifted".into()));
    }
    if corpus.projected_storage_bytes.is_none() {
        totals.unknown_storage += 1;
    }
    if corpus.projected_cpu_hours.is_none() {
        totals.unknown_compute += 1;
    }
    if corpus.publishable_data_ready {
        validate_ready_corpus(corpus)?;
    }
    Ok(())
}

fn validate_source_specs(corpus: &BenchmarkCorpusDataAudit) -> Result<(), BenchError> {
    let mut paths = BTreeSet::new();
    for source in &corpus.source_specs {
        if source.path.is_empty() || !paths.insert(source.path.as_str()) {
            return Err(BenchError::InvalidPolicy("duplicate or empty source-spec path".into()));
        }
        validate_hex(&source.sha256, 64, "source-spec digest")?;
        let selection = [
            source.selection_group_count.is_some(),
            source.planned_sample_count.is_some(),
            source.planned_accession_count.is_some(),
            source.selected_accession_count.is_some(),
            source.selection_complete.is_some(),
        ];
        if selection.iter().any(|present| *present) && !selection.iter().all(|present| *present) {
            return Err(BenchError::InvalidPolicy(
                "source-spec selection authority is partial".into(),
            ));
        }
        if let (
            Some(groups),
            Some(planned_samples),
            Some(planned_accessions),
            Some(selected),
            Some(complete),
        ) = (
            source.selection_group_count,
            source.planned_sample_count,
            source.planned_accession_count,
            source.selected_accession_count,
            source.selection_complete,
        ) {
            if groups == 0
                || planned_samples == 0
                || planned_accessions < planned_samples
                || selected > planned_accessions
                || (complete != (selected == planned_accessions))
            {
                return Err(BenchError::InvalidPolicy(
                    "source-spec selection authority is inconsistent".into(),
                ));
            }
        }
    }
    Ok(())
}

fn validate_ready_corpus(corpus: &BenchmarkCorpusDataAudit) -> Result<(), BenchError> {
    let observed =
        corpus.catalog_bindings.iter().filter_map(|binding| binding.observed_sample_count).max();
    if corpus.declared_sample_count < corpus.minimum_sample_count
        || corpus.catalog_bindings.is_empty()
        || corpus.catalog_bindings.iter().any(|binding| {
            !binding.present || !binding.integrity_verified || !binding.benchmark_eligible
        })
        || observed.is_none_or(|count| count < u64::from(corpus.declared_sample_count))
    {
        return Err(BenchError::InvalidPolicy(
            "publishable-data ready corpus lacks complete governed evidence".into(),
        ));
    }
    Ok(())
}

fn validate_unbound_assets(
    audit: &BenchmarkDataPortfolioAudit,
    bound_ids: &BTreeSet<String>,
) -> Result<(), BenchError> {
    let mut unbound_ids = BTreeSet::new();
    for asset in &audit.unbound_assets {
        if asset.catalog_id.is_empty()
            || bound_ids.contains(asset.catalog_id.as_str())
            || !unbound_ids.insert(asset.catalog_id.as_str())
            || asset.access_classification.is_empty()
            || asset.redistribution_classification.is_empty()
            || asset.suspected_nonproduction == asset.classification_reasons.is_empty()
        {
            return Err(BenchError::InvalidPolicy("invalid unbound data asset".into()));
        }
    }
    Ok(())
}

fn validate_contamination(audit: &BenchmarkDataPortfolioAudit) -> Result<(), BenchError> {
    let mut contamination_ids = BTreeSet::new();
    for finding in &audit.publishable_contamination {
        let (corpus_id, catalog_id) = finding
            .split_once(':')
            .ok_or_else(|| BenchError::InvalidPolicy("invalid contamination identity".into()))?;
        let corpus = audit
            .corpora
            .iter()
            .find(|corpus| corpus.corpus_id == corpus_id)
            .ok_or_else(|| BenchError::InvalidPolicy("unknown contamination corpus".into()))?;
        if catalog_id.is_empty()
            || corpus.publishable_data_ready
            || !corpus.catalog_bindings.iter().any(|binding| binding.catalog_id == catalog_id)
            || !contamination_ids.insert(finding.as_str())
        {
            return Err(BenchError::InvalidPolicy(
                "invalid publishable contamination finding".into(),
            ));
        }
    }
    Ok(())
}

fn validate_summary(
    audit: &BenchmarkDataPortfolioAudit,
    totals: &AuditTotals,
) -> Result<(), BenchError> {
    let corpus_count = u64::try_from(audit.corpora.len()).map_err(|_| overflow())?;
    let unbound_count = u64::try_from(audit.unbound_assets.len()).map_err(|_| overflow())?;
    let suspected = count_suspected_assets(audit)?;
    let contamination =
        u64::try_from(audit.publishable_contamination.len()).map_err(|_| overflow())?;
    let summary = &audit.summary;
    let expected = [
        (summary.corpus_count, corpus_count),
        (summary.admitted_corpus_count, totals.admitted),
        (summary.source_defined_corpus_count, totals.source_defined),
        (summary.missing_corpus_count, totals.missing),
        (summary.publishable_data_ready_count, totals.ready),
        (summary.bound_catalog_count, totals.present),
        (summary.missing_catalog_binding_count, totals.absent),
        (summary.unbound_catalog_count, unbound_count),
        (summary.verified_bound_file_count, totals.files),
        (summary.verified_bound_size_bytes, totals.bytes),
        (summary.unknown_storage_projection_count, totals.unknown_storage),
        (summary.unknown_compute_projection_count, totals.unknown_compute),
        (summary.suspected_nonproduction_asset_count, suspected),
        (summary.publishable_contamination_count, contamination),
    ];
    if expected.iter().any(|(actual, derived)| actual != derived) {
        return Err(BenchError::InvalidPolicy("data-portfolio audit summary drifted".into()));
    }
    Ok(())
}

fn count_suspected_assets(audit: &BenchmarkDataPortfolioAudit) -> Result<u64, BenchError> {
    u64::try_from(audit.unbound_assets.iter().filter(|asset| asset.suspected_nonproduction).count())
        .map_err(|_| overflow())
}

fn validate_binding(binding: &BenchmarkGovernedDataBinding) -> Result<(), BenchError> {
    if binding.catalog_id.is_empty() {
        return Err(BenchError::InvalidPolicy("empty catalog binding identity".into()));
    }
    if binding.present {
        if binding.catalog_path.is_none()
            || binding.catalog_sha256.is_none()
            || binding.access_classification.as_deref().is_none_or(str::is_empty)
            || binding.redistribution_classification.as_deref().is_none_or(str::is_empty)
            || binding.access_terms.as_deref().is_none_or(str::is_empty)
            || binding.verified_file_count == 0
        {
            return Err(BenchError::InvalidPolicy("present catalog binding is incomplete".into()));
        }
        validate_hex(binding.catalog_sha256.as_deref().unwrap_or_default(), 64, "catalog digest")?;
    } else if binding.catalog_path.is_some()
        || binding.catalog_sha256.is_some()
        || binding.access_classification.is_some()
        || binding.redistribution_classification.is_some()
        || binding.access_terms.is_some()
        || binding.integrity_verified
        || binding.benchmark_eligible
        || binding.verified_file_count != 0
        || binding.verified_size_bytes != 0
        || binding.observed_sample_count.is_some()
        || binding.sample_count_authority.is_some()
        || !binding.file_roles.is_empty()
    {
        return Err(BenchError::InvalidPolicy("absent catalog binding carries evidence".into()));
    }
    if binding.observed_sample_count.is_some() != binding.sample_count_authority.is_some() {
        return Err(BenchError::InvalidPolicy("sample count lacks an authority".into()));
    }
    Ok(())
}

fn validate_hex(value: &str, length: usize, label: &str) -> Result<(), BenchError> {
    if value.len() != length
        || !value.bytes().all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(BenchError::InvalidPolicy(format!("invalid {label}")));
    }
    Ok(())
}

fn overflow() -> BenchError {
    BenchError::InvalidPolicy("data-portfolio audit count overflow".into())
}
