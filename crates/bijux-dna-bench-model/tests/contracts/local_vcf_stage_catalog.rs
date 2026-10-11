use bijux_dna_bench_model::{
    validate_local_vcf_stage_catalog_config, validate_local_vcf_stage_catalog_report,
    LocalVcfStageCatalogConfig, LocalVcfStageCatalogReport, VcfStageCatalogRow,
    LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1,
};

fn row() -> VcfStageCatalogRow {
    VcfStageCatalogRow {
        stage_id: "vcf.call_variants".to_string(),
        stage_name: "Variant Calling".to_string(),
        support_status: "supported".to_string(),
        default_tool_id: "bcftools".to_string(),
        metrics_schema_id: "bijux.metrics.vcf.call.v1".to_string(),
        input_types: vec!["bam".to_string()],
        output_types: vec!["vcf".to_string()],
        required_assets: vec!["reference_fasta".to_string()],
        benchmark_category: "variant_calling".to_string(),
        local_smoke_mode: "bam_bundle".to_string(),
    }
}

#[test]
fn local_vcf_stage_catalog_contracts_admit_closed_reports() -> anyhow::Result<()> {
    let rows = vec![row()];
    validate_local_vcf_stage_catalog_config(&LocalVcfStageCatalogConfig {
        schema_version: LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1.to_string(),
        rows: rows.clone(),
    })?;
    validate_local_vcf_stage_catalog_report(&LocalVcfStageCatalogReport {
        schema_version: LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/vcf-stage-catalog.toml".to_string(),
        stage_count: 1,
        supported_stage_count: 1,
        planned_stage_count: 0,
        rows,
    })?;
    Ok(())
}

#[test]
fn local_vcf_stage_catalog_contract_rejects_unknown_support_status() -> anyhow::Result<()> {
    let mut invalid = row();
    invalid.support_status = "redistributable".to_string();
    let result = validate_local_vcf_stage_catalog_config(&LocalVcfStageCatalogConfig {
        schema_version: LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1.to_string(),
        rows: vec![invalid],
    });
    let error =
        result.err().ok_or_else(|| anyhow::anyhow!("unknown support status was admitted"))?;
    assert!(error.to_string().contains("unsupported support_status"));
    Ok(())
}
