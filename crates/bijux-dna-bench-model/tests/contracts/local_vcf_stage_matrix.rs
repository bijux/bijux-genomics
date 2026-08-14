use bijux_dna_bench_model::{
    validate_local_vcf_stage_matrix_config, validate_local_vcf_stage_matrix_report,
    validate_vcf_stage_matrix_validation_report, LocalVcfStageMatrixConfig,
    LocalVcfStageMatrixReport, ValidateVcfStageMatrixReport, VcfStageMatrixRow,
    LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1,
    VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1,
};

fn row() -> VcfStageMatrixRow {
    VcfStageMatrixRow {
        stage_id: "vcf.call_variants".to_string(),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf-production-regression".to_string(),
        asset_profile_id: "tiny-diploid".to_string(),
        adapter_id: "vcf.call_variants.bcftools".to_string(),
        parser_id: "vcf.call_variants.v1".to_string(),
        expected_outputs: vec!["variants_vcf".to_string()],
    }
}

#[test]
fn local_vcf_stage_matrix_contracts_admit_closed_reports() {
    let rows = vec![row()];
    validate_local_vcf_stage_matrix_config(&LocalVcfStageMatrixConfig {
        schema_version: LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1.to_string(),
        rows: rows.clone(),
    })
    .expect("valid config");
    validate_local_vcf_stage_matrix_report(&LocalVcfStageMatrixReport {
        schema_version: LOCAL_VCF_STAGE_MATRIX_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/vcf-stage-matrix.toml".to_string(),
        row_count: 1,
        stage_count: 1,
        supported_stage_count: 1,
        planned_stage_count: 0,
        rows: rows.clone(),
    })
    .expect("valid render report");
    validate_vcf_stage_matrix_validation_report(&ValidateVcfStageMatrixReport {
        schema_version: VCF_STAGE_MATRIX_VALIDATION_SCHEMA_V1.to_string(),
        domain: "vcf".to_string(),
        matrix_path: "configs/bench/local/vcf-stage-matrix.toml".to_string(),
        strict: true,
        row_count: 1,
        stage_count: 1,
        required_tool_count: 2,
        registry_tool_count: 2,
        rows,
    })
    .expect("valid validation report");
}

#[test]
fn local_vcf_stage_matrix_contract_rejects_duplicate_stage_identity() {
    let duplicate = row();
    let error = validate_local_vcf_stage_matrix_config(&LocalVcfStageMatrixConfig {
        schema_version: LOCAL_VCF_STAGE_MATRIX_SCHEMA_V1.to_string(),
        rows: vec![duplicate.clone(), duplicate],
    })
    .expect_err("duplicate stage must fail");
    assert!(error.to_string().contains("repeats a stage identity"));
}
