use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    stage_tool_containers_config_schema, stage_tool_containers_report_schema,
    validate_stage_tool_containers_config, validate_stage_tool_containers_report,
    StageToolContainerRow, StageToolContainersConfig, StageToolContainersReport,
    LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1, STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN,
    STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1, STAGE_TOOL_CONTAINERS_SCOPE,
};

#[test]
fn stage_tool_container_schemas_match_checked_in_contracts() -> anyhow::Result<()> {
    assert_schema(
        "stage-tool-containers.schema.json",
        &serde_json::to_value(stage_tool_containers_config_schema())?,
    )?;
    assert_schema(
        "stage-tool-containers-report.schema.json",
        &serde_json::to_value(stage_tool_containers_report_schema())?,
    )
}

#[test]
fn stage_tool_container_contracts_admit_runtime_closure() -> anyhow::Result<()> {
    validate_stage_tool_containers_config(&valid_config())?;
    validate_stage_tool_containers_report(&valid_report())?;

    let mut vcf_config = valid_config();
    vcf_config.rows[0].domain = "vcf".to_string();
    vcf_config.rows[0].stage_id = "vcf.impute".to_string();
    vcf_config.rows[0].tool_id = "impute5".to_string();
    vcf_config.rows[0].container_id = Some("external-artifact://impute5".to_string());
    validate_stage_tool_containers_config(&vcf_config)?;
    Ok(())
}

#[test]
fn stage_tool_container_contracts_reject_shape_identity_and_aggregate_drift() {
    let mut config = valid_config();
    config.rows[0].container_id = Some("external-artifact://angsd".to_string());
    assert!(validate_stage_tool_containers_config(&config).is_err());

    let mut config = valid_config();
    config.rows.push(config.rows[0].clone());
    assert!(validate_stage_tool_containers_config(&config).is_err());

    for placeholder in ["external@external", "containers/apptainer/tool.def@planned"] {
        let mut config = valid_config();
        config.rows[0].execution_mode = "containerized".to_string();
        config.rows[0].install_kind = "container".to_string();
        config.rows[0].container_id = Some(placeholder.to_string());
        assert!(validate_stage_tool_containers_config(&config).is_err());
    }

    let mut report = valid_report();
    report.config_path = "../outside.toml".to_string();
    assert!(validate_stage_tool_containers_report(&report).is_err());

    let mut report = valid_report();
    report.command_entrypoint_row_count = 0;
    assert!(validate_stage_tool_containers_report(&report).is_err());
}

fn valid_row() -> StageToolContainerRow {
    StageToolContainerRow {
        domain: "fastq".to_string(),
        stage_id: "fastq.remove_duplicates".to_string(),
        tool_id: "fastuniq".to_string(),
        execution_mode: "external".to_string(),
        install_kind: "external".to_string(),
        container_id: Some("external-artifact://fastuniq".to_string()),
        command_entrypoint: Some("FastUniq".to_string()),
        host_binary_mode: None,
        declaration_origin: STAGE_TOOL_CONTAINERS_DECLARATION_ORIGIN.to_string(),
    }
}

fn valid_config() -> StageToolContainersConfig {
    StageToolContainersConfig {
        schema_version: LOCAL_STAGE_TOOL_CONTAINERS_SCHEMA_V1.to_string(),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE.to_string(),
        rows: vec![valid_row()],
    }
}

fn valid_report() -> StageToolContainersReport {
    StageToolContainersReport {
        schema_version: STAGE_TOOL_CONTAINERS_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/stage-tool-containers.toml".to_string(),
        classification_scope: STAGE_TOOL_CONTAINERS_SCOPE.to_string(),
        row_count: 1,
        benchmark_ready_row_count: 1,
        external_row_count: 1,
        container_declared_row_count: 1,
        command_entrypoint_row_count: 1,
        host_binary_row_count: 0,
        domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        execution_mode_counts: BTreeMap::from([("external".to_string(), 1)]),
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
