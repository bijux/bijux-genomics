//! Canonical benchmark identity reconciliation over already acquired rows.

use std::collections::BTreeMap;

use bijux_dna_core::ids::{parse_benchmark_result_id, BenchmarkResultScopeKind};
use bijux_dna_core::prelude::Result;

use crate::AllDomainExpectedBenchmarkResultRow;

/// Explicit builder binding; fields remain independent so drift can be diagnosed.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CanonicalBenchmarkResultBinding {
    pub result_id: String,
    pub domain: String,
    pub stage_id: String,
    pub tool_id: String,
    pub corpus_id: String,
    pub scope_kind: BenchmarkResultScopeKind,
    pub scope_id: String,
}

/// A pure identity finding; report serialization belongs to the caller.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalResultIdentityViolation {
    pub result_id: String,
    pub surface: String,
    pub detail: String,
}

/// Last-binding index and original row count, built before expected-row acquisition.
#[derive(Debug)]
pub struct CanonicalBenchmarkResultIndex {
    row_count: usize,
    by_result: BTreeMap<String, CanonicalBenchmarkResultBinding>,
}

/// Index explicit canonical bindings and append duplicate findings in source order.
#[must_use]
pub fn index_canonical_benchmark_result_bindings(
    rows: &[CanonicalBenchmarkResultBinding],
    violations: &mut Vec<CanonicalResultIdentityViolation>,
) -> CanonicalBenchmarkResultIndex {
    let mut canonical_by_result = BTreeMap::new();
    for row in rows {
        if canonical_by_result.insert(row.result_id.clone(), row.clone()).is_some() {
            push_violation(
                violations,
                &row.result_id,
                "canonical_result_builder",
                "canonical result-id builder produced a duplicate binding",
            );
        }
    }
    CanonicalBenchmarkResultIndex { row_count: rows.len(), by_result: canonical_by_result }
}

/// Compare acquired all-domain rows against the indexed canonical bindings.
///
/// Findings preserve input order and remain available when a later row fails parsing.
/// Missing canonical bindings are findings and do not reach the parser.
///
/// # Errors
/// Returns the unchanged Core parsing error for the first matched malformed result ID.
pub fn reconcile_canonical_benchmark_result_identities(
    canonical: &CanonicalBenchmarkResultIndex,
    all_domain_rows: &[AllDomainExpectedBenchmarkResultRow],
    violations: &mut Vec<CanonicalResultIdentityViolation>,
) -> Result<()> {
    if all_domain_rows.len() != canonical.row_count {
        push_violation(
            violations,
            "all-domain-expected-row-count",
            "all_domain_expected_benchmark_results",
            format!(
                "all-domain expected rows drifted from canonical benchmark rows: expected {}, found {}",
                canonical.row_count,
                all_domain_rows.len()
            ),
        );
    }
    for row in all_domain_rows {
        let Some(canonical) = canonical.by_result.get(&row.result_id) else {
            push_violation(
                violations,
                &row.result_id,
                "all_domain_expected_benchmark_results",
                "all-domain expected row is missing from the canonical result-id builder",
            );
            continue;
        };
        let parsed = parse_benchmark_result_id(&row.result_id)?;
        if canonical.domain != row.domain
            || canonical.stage_id != row.stage_id.as_str()
            || canonical.tool_id != row.tool_id.as_str()
            || canonical.corpus_id != row.corpus_id
            || canonical.scope_kind != parsed.scope_kind
            || canonical.scope_id != parsed.scope_id
        {
            push_violation(
                violations,
                &row.result_id,
                "all_domain_expected_benchmark_results",
                "all-domain expected row drifted from the canonical result-id identity",
            );
        }
    }
    Ok(())
}

fn push_violation(
    violations: &mut Vec<CanonicalResultIdentityViolation>,
    result_id: &str,
    surface: &str,
    detail: impl Into<String>,
) {
    violations.push(CanonicalResultIdentityViolation {
        result_id: result_id.to_string(),
        surface: surface.to_string(),
        detail: detail.into(),
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bam() -> CanonicalBenchmarkResultBinding {
        CanonicalBenchmarkResultBinding {
            result_id: "bam:corpus-01-kinship-mini:bam.kinship:sample-set:king".to_string(),
            domain: "bam".to_string(),
            stage_id: "bam.kinship".to_string(),
            tool_id: "king".to_string(),
            corpus_id: "corpus-01-kinship-mini".to_string(),
            scope_kind: BenchmarkResultScopeKind::SampleScope,
            scope_id: "sample-set".to_string(),
        }
    }

    fn expected(binding: &CanonicalBenchmarkResultBinding) -> AllDomainExpectedBenchmarkResultRow {
        AllDomainExpectedBenchmarkResultRow {
            result_id: binding.result_id.clone(),
            domain: binding.domain.clone(),
            stage_id: binding.stage_id.clone(),
            tool_id: binding.tool_id.clone(),
            corpus_id: binding.corpus_id.clone(),
            asset_profile_id: "reference_fasta+reference_panel".to_string(),
            expected_outputs: vec!["kinship_report".to_string()],
            expected_metrics: vec!["kinship_report".to_string()],
            report_section: "sample_identity".to_string(),
        }
    }

    #[test]
    fn accepts_sample_and_asset_scopes_without_equating_sample_scope_with_asset_profile(
    ) -> Result<()> {
        let bam = bam();
        let vcf = CanonicalBenchmarkResultBinding {
            result_id: "vcf:vcf_production_regression:vcf.call:bam_bundle:bcftools".to_string(),
            domain: "vcf".to_string(),
            stage_id: "vcf.call".to_string(),
            tool_id: "bcftools".to_string(),
            corpus_id: "vcf_production_regression".to_string(),
            scope_kind: BenchmarkResultScopeKind::AssetProfile,
            scope_id: "bam_bundle".to_string(),
        };
        let rows = [bam.clone(), vcf.clone()];
        let mut found = Vec::new();
        let index = index_canonical_benchmark_result_bindings(&rows, &mut found);
        let mut vcf_expected = expected(&vcf);
        vcf_expected.asset_profile_id = "bam_bundle".to_string();
        reconcile_canonical_benchmark_result_identities(
            &index,
            &[expected(&bam), vcf_expected],
            &mut found,
        )?;
        assert!(found.is_empty());
        Ok(())
    }

    #[test]
    fn duplicate_count_and_identity_findings_keep_order_and_last_binding() -> Result<()> {
        let first = bam();
        let mut last = first.clone();
        last.scope_id = "different-sample-scope".to_string();
        let mut found = Vec::new();
        let index = index_canonical_benchmark_result_bindings(&[first.clone(), last], &mut found);
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].surface, "canonical_result_builder");
        reconcile_canonical_benchmark_result_identities(&index, &[expected(&first)], &mut found)?;
        assert_eq!(
            found.iter().map(|row| row.result_id.as_str()).collect::<Vec<_>>(),
            [first.result_id.as_str(), "all-domain-expected-row-count", first.result_id.as_str()]
        );
        assert_eq!(found.iter().map(|row| row.detail.as_str()).collect::<Vec<_>>(), [
            "canonical result-id builder produced a duplicate binding",
            "all-domain expected rows drifted from canonical benchmark rows: expected 2, found 1",
            "all-domain expected row drifted from the canonical result-id identity",
        ]);
        Ok(())
    }

    #[test]
    fn every_identity_field_and_both_scope_fields_reject_independent_drift() -> Result<()> {
        let mutations: [fn(&mut CanonicalBenchmarkResultBinding); 6] = [
            |row| row.domain = "fastq".to_string(),
            |row| row.stage_id = "bam.other".to_string(),
            |row| row.tool_id = "other-tool".to_string(),
            |row| row.corpus_id = "other-corpus".to_string(),
            |row| row.scope_kind = BenchmarkResultScopeKind::AssetProfile,
            |row| row.scope_id = "other-scope".to_string(),
        ];
        for mutate in mutations {
            let original = bam();
            let mut changed = original.clone();
            mutate(&mut changed);
            let mut found = Vec::new();
            let index = index_canonical_benchmark_result_bindings(&[changed], &mut found);
            reconcile_canonical_benchmark_result_identities(
                &index,
                &[expected(&original)],
                &mut found,
            )?;
            assert_eq!(found.len(), 1);
            assert_eq!(found[0].result_id, original.result_id);
            assert_eq!(
                found[0].detail,
                "all-domain expected row drifted from the canonical result-id identity"
            );
        }
        Ok(())
    }

    #[test]
    fn missing_bindings_skip_parsing_and_retain_acquired_row_order() -> Result<()> {
        let mut first = expected(&bam());
        first.result_id = "z-missing-malformed".to_string();
        let mut second = first.clone();
        second.result_id = "a-missing-malformed".to_string();
        let mut found = Vec::new();
        let index = index_canonical_benchmark_result_bindings(&[], &mut found);
        reconcile_canonical_benchmark_result_identities(&index, &[first, second], &mut found)?;
        assert_eq!(
            found.iter().map(|row| row.result_id.as_str()).collect::<Vec<_>>(),
            ["all-domain-expected-row-count", "z-missing-malformed", "a-missing-malformed"]
        );
        assert!(found[1..].iter().all(|row| row.detail
            == "all-domain expected row is missing from the canonical result-id builder"));
        Ok(())
    }

    #[test]
    fn core_parse_error_precedes_field_drift_and_retains_prior_findings() -> anyhow::Result<()> {
        let mut malformed = bam();
        malformed.result_id = "malformed".to_string();
        let mut missing = expected(&malformed);
        missing.result_id = "also-malformed-but-missing".to_string();
        let mut matched = expected(&malformed);
        matched.domain = "field-drift-must-not-mask-parse-error".to_string();
        let mut found = Vec::new();
        let index =
            index_canonical_benchmark_result_bindings(&[malformed.clone(), malformed], &mut found);
        let actual = reconcile_canonical_benchmark_result_identities(
            &index,
            &[missing, matched],
            &mut found,
        );
        let Err(actual) = actual else {
            anyhow::bail!("matched malformed identity unexpectedly admitted");
        };
        let Err(expected) = parse_benchmark_result_id("malformed") else {
            anyhow::bail!("Core accepted malformed control");
        };
        assert_eq!(actual.to_string(), expected.to_string());
        assert_eq!(found.len(), 2);
        assert_eq!(found[0].detail, "canonical result-id builder produced a duplicate binding");
        assert_eq!(
            found[1].detail,
            "all-domain expected row is missing from the canonical result-id builder"
        );
        Ok(())
    }
}
