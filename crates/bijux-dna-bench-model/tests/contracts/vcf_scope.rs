use super::*;
use crate::REMOVED_FROM_SCOPE_SCHEMA_VERSION;
use std::collections::BTreeMap;

fn removed() -> RemovedFromScopeReport {
    RemovedFromScopeReport {
        schema_version: REMOVED_FROM_SCOPE_SCHEMA_VERSION.to_string(),
        output_path: "removed.tsv".to_string(),
        full_benchmark_report_check_source: "observed".to_string(),
        candidate_row_count: 0,
        candidate_stage_count: 0,
        candidate_tool_count: 0,
        active_row_count: 0,
        active_stage_count: 0,
        active_tool_count: 0,
        removed_row_count: 0,
        removed_stage_count: 0,
        removed_tool_count: 0,
        fully_removed_stage_count: 0,
        fully_removed_tool_count: 0,
        scope_exit_kind_counts: BTreeMap::new(),
        violation_count: 0,
        ok: true,
        rows: vec![],
        violations: vec![],
    }
}
fn active() -> AllDomainActiveStageToolMatrixReport {
    let rows = [
        (VCF_IMPUTE_STAGE_ID, VCF_OUTPUT_PARSER_ID, VCF_IMPUTE_SCHEMA_ID),
        (
            VCF_IMPUTATION_METRICS_STAGE_ID,
            VCF_REPORT_JSON_PARSER_ID,
            VCF_IMPUTATION_METRICS_SCHEMA_ID,
        ),
    ]
    .into_iter()
    .map(|(stage, parser, schema)| crate::AllDomainActiveStageToolMatrixRow {
        domain: "vcf".to_string(),
        stage_id: stage.to_string(),
        tool_id: "beagle_v5".to_string(),
        corpus_id: "panel".to_string(),
        asset_profile_id: "diploid".to_string(),
        adapter_id: "adapter".to_string(),
        parser_id: parser.to_string(),
        schema_id: schema.to_string(),
        status: "benchmark_ready".to_string(),
    })
    .collect();
    AllDomainActiveStageToolMatrixReport {
        schema_version: crate::ALL_DOMAIN_ACTIVE_STAGE_TOOL_MATRIX_SCHEMA_V1.to_string(),
        output_path: "active.tsv".to_string(),
        row_count: 2,
        stage_count: 2,
        tool_count: 1,
        domain_counts: BTreeMap::from([("vcf".to_string(), 2)]),
        status_counts: BTreeMap::from([("benchmark_ready".to_string(), 2)]),
        rows,
    }
}

#[test]
fn vcf_scope_imputation_rejects_legacy_missing_and_cross_wired_contracts() -> Result<()> {
    let observed = active();
    let removed = removed();
    assert!(validate_vcf_scope_imputation_identity(Some(&observed), Some(&removed))?
        .contains("imputation_metrics_row_count=1"));
    assert!(validate_vcf_scope_imputation_identity(None, Some(&removed)).is_err());
    assert!(validate_vcf_scope_imputation_identity(Some(&observed), None).is_err());
    for index in 0..2 {
        let mut changed = observed.clone();
        changed.rows.remove(index);
        assert!(validate_vcf_scope_imputation_identity(Some(&changed), Some(&removed)).is_err());
        let mut changed = observed.clone();
        changed.rows[index].stage_id = VCF_LEGACY_IMPUTATION_STAGE_ID.to_string();
        assert!(validate_vcf_scope_imputation_identity(Some(&changed), Some(&removed)).is_err());
        let mut changed = observed.clone();
        changed.rows[index].parser_id = observed.rows[1 - index].parser_id.clone();
        assert!(validate_vcf_scope_imputation_identity(Some(&changed), Some(&removed)).is_err());
        let mut changed = observed.clone();
        changed.rows[index].schema_id = observed.rows[1 - index].schema_id.clone();
        assert!(validate_vcf_scope_imputation_identity(Some(&changed), Some(&removed)).is_err());
    }
    Ok(())
}

#[test]
fn vcf_scope_coverage_requires_the_same_corpus_and_asset_identity() -> Result<()> {
    let key = BindingKey {
        domain: "vcf".to_string(),
        stage_id: VCF_POSTPROCESS_STAGE_ID.to_string(),
        tool_id: "bcftools_v1".to_string(),
        corpus_id: "truth".to_string(),
        asset_profile_id: "diploid".to_string(),
    };
    let expected = BTreeSet::from([key.clone()]);
    ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "parser fixture coverage",
        &expected,
        &expected,
    )?;
    let mut wrong = key.clone();
    wrong.corpus_id = "other".to_string();
    assert!(ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "parser fixture coverage",
        &expected,
        &BTreeSet::from([wrong])
    )
    .is_err());
    let mut wrong = key;
    wrong.asset_profile_id = "haploid".to_string();
    assert!(ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "parser fixture coverage",
        &expected,
        &BTreeSet::from([wrong])
    )
    .is_err());
    assert!(ensure_covering_keys(
        VCF_POSTPROCESS_STAGE_ID,
        "parser fixture coverage",
        &expected,
        &BTreeSet::new()
    )
    .is_err());
    Ok(())
}

#[test]
fn removed_scope_wire_rejects_foreign_versions_and_unknown_fields() -> Result<()> {
    let report = removed();
    let value = serde_json::to_value(&report)?;
    assert_eq!(
        serde_json::from_value::<RemovedFromScopeReport>(value.clone())?.schema_version,
        report.schema_version
    );
    for (field, replacement) in
        [("schema_version", serde_json::json!("unsupported")), ("unknown", serde_json::json!(true))]
    {
        let mut changed = value.clone();
        changed[field] = replacement;
        assert!(serde_json::from_value::<RemovedFromScopeReport>(changed).is_err());
    }
    Ok(())
}
