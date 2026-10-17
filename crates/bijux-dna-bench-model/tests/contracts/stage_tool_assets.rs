use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    stage_tool_assets_config_schema, stage_tool_assets_report_schema,
    validate_governed_stage_tool_asset_coverage, validate_stage_tool_assets_config,
    validate_stage_tool_assets_report, StageToolAssetRow, StageToolAssetsConfig,
    StageToolAssetsReport, LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1, STAGE_TOOL_ASSETS_DECLARATION_ORIGIN,
    STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1, STAGE_TOOL_ASSETS_SCOPE,
};

#[test]
fn stage_tool_asset_schemas_match_checked_in_contracts() -> anyhow::Result<()> {
    assert_schema(
        "stage-tool-assets.schema.json",
        &serde_json::to_value(stage_tool_assets_config_schema())?,
    )?;
    assert_schema(
        "stage-tool-assets-report.schema.json",
        &serde_json::to_value(stage_tool_assets_report_schema())?,
    )
}

#[test]
fn stage_tool_asset_contracts_admit_exact_closure() -> anyhow::Result<()> {
    validate_stage_tool_assets_config(&valid_config())?;
    validate_stage_tool_assets_report(&valid_report())?;
    Ok(())
}

#[test]
fn stage_tool_asset_contracts_reject_path_identity_and_aggregate_drift() {
    let mut config = valid_config();
    config.rows[0].asset_path = "../outside.fasta".to_string();
    assert!(validate_stage_tool_assets_config(&config).is_err());

    let mut config = valid_config();
    config.rows.push(config.rows[0].clone());
    assert!(validate_stage_tool_assets_config(&config).is_err());

    let mut report = valid_report();
    report.unique_asset_id_count += 1;
    assert!(validate_stage_tool_assets_report(&report).is_err());
}

fn valid_row() -> StageToolAssetRow {
    StageToolAssetRow {
        domain: "fastq".to_string(),
        stage_id: "fastq.index_reference".to_string(),
        tool_id: "bwa".to_string(),
        asset_role: "reference_fasta".to_string(),
        asset_id: "hg38".to_string(),
        asset_path: "benchmarks/data/reference/hg38.fasta".to_string(),
        declaration_origin: STAGE_TOOL_ASSETS_DECLARATION_ORIGIN.to_string(),
    }
}

fn valid_config() -> StageToolAssetsConfig {
    StageToolAssetsConfig {
        schema_version: LOCAL_STAGE_TOOL_ASSETS_SCHEMA_V1.to_string(),
        classification_scope: STAGE_TOOL_ASSETS_SCOPE.to_string(),
        rows: vec![valid_row()],
    }
}

fn valid_report() -> StageToolAssetsReport {
    StageToolAssetsReport {
        schema_version: STAGE_TOOL_ASSETS_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/stage-tool-assets.toml".to_string(),
        classification_scope: STAGE_TOOL_ASSETS_SCOPE.to_string(),
        row_count: 1,
        declared_stage_tool_row_count: 1,
        asset_id_row_count: 1,
        unique_asset_id_count: 1,
        domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        asset_role_counts: BTreeMap::from([("reference_fasta".to_string(), 1)]),
        rows: vec![valid_row()],
    }
}

fn assert_schema(file_name: &str, generated: &serde_json::Value) -> anyhow::Result<()> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let path = root.join("benchmarks/contracts/schemas").join(file_name);
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    assert_eq!(&serde_json::from_slice::<serde_json::Value>(&std::fs::read(path)?)?, generated);
    Ok(())
}

fn governed_config() -> anyhow::Result<StageToolAssetsConfig> {
    // This is compiled fixture data, not authority to discover or modify a live repository.
    Ok(toml::from_str(include_str!("../../../../configs/bench/local/stage-tool-assets.toml"))?)
}

#[test]
fn governed_stage_tool_asset_coverage_accepts_committed_bindings() -> anyhow::Result<()> {
    let config = governed_config()?;
    validate_stage_tool_assets_config(&config)?;
    validate_governed_stage_tool_asset_coverage(&config.rows)?;
    Ok(())
}

#[test]
fn governed_stage_tool_asset_coverage_rejects_missing_bindings_in_order() -> anyhow::Result<()> {
    use bijux_dna_core::id_catalog::{BAM_GENOTYPING, FASTQ_SCREEN};

    let missing_taxonomy = "FASTQ taxonomy asset coverage expected 4 rows but found 3";
    let missing_sites = format!(
        "BAM genotyping asset coverage is missing `{BAM_GENOTYPING}` / `bcftools` / `sites_vcf`"
    );
    for (missing_fastq, missing_bam, expected) in [
        (true, false, missing_taxonomy),
        (false, true, missing_sites.as_str()),
        (true, true, missing_taxonomy),
    ] {
        let mut config = governed_config()?;
        let before = config.rows.len();
        config.rows.retain(|row| {
            !((missing_fastq
                && row.stage_id == FASTQ_SCREEN
                && row.tool_id == "kraken2"
                && row.asset_role == "taxonomy_database_root")
                || (missing_bam
                    && row.stage_id == BAM_GENOTYPING
                    && row.tool_id == "bcftools"
                    && row.asset_role == "sites_vcf"))
        });
        assert_eq!(
            before - config.rows.len(),
            usize::from(missing_fastq) + usize::from(missing_bam)
        );
        validate_stage_tool_assets_config(&config)?;
        let error = validate_governed_stage_tool_asset_coverage(&config.rows)
            .err()
            .ok_or_else(|| anyhow::anyhow!("missing governed asset binding was admitted"))?;
        assert_eq!(error.to_string(), expected);
    }
    Ok(())
}

#[test]
fn governed_stage_tool_asset_coverage_rejects_rebound_paths() -> anyhow::Result<()> {
    use bijux_dna_core::id_catalog::{BAM_RECALIBRATION, FASTQ_SCREEN};

    for (stage_id, tool_id, role, expected) in [
        (
            FASTQ_SCREEN,
            "kraken2",
            "taxonomy_database_root",
            "FASTQ taxonomy asset row `kraken2` must stay bound to the governed taxonomy reference root"
                .to_string(),
        ),
        (
            BAM_RECALIBRATION,
            "gatk",
            "known_sites",
            format!(
                "BAM recalibration asset row `{BAM_RECALIBRATION}` / `gatk` / `known_sites` must stay bound to `human_like_recalibration_known_sites` at `benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/variants/human_like_recalibration_known_sites.vcf`"
            ),
        ),
    ] {
        let mut config = governed_config()?;
        let row = config.rows.iter_mut()
            .find(|row| row.stage_id == stage_id && row.tool_id == tool_id && row.asset_role == role)
            .ok_or_else(|| anyhow::anyhow!("governed reference fixture omitted the selected binding"))?;
        row.asset_path = "assets/reference/another-governed-reference.fasta".to_string();
        validate_stage_tool_assets_config(&config)?;
        let error = validate_governed_stage_tool_asset_coverage(&config.rows)
            .err()
            .ok_or_else(|| anyhow::anyhow!("rebound governed asset path was admitted"))?;
        assert_eq!(error.to_string(), expected);
    }
    Ok(())
}
