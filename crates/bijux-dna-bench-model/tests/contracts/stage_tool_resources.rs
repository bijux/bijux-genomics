use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    stage_tool_resources_config_schema, stage_tool_resources_report_schema,
    validate_stage_tool_resources_config, validate_stage_tool_resources_report,
    StageToolResourceRow, StageToolResourcesConfig, StageToolResourcesReport,
    LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1, STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1,
    STAGE_TOOL_RESOURCES_SCOPE,
};

#[test]
fn stage_tool_resource_schemas_match_checked_in_contracts() -> anyhow::Result<()> {
    assert_schema(
        "stage-tool-resources.schema.json",
        &serde_json::to_value(stage_tool_resources_config_schema())?,
    )?;
    assert_schema(
        "stage-tool-resources-report.schema.json",
        &serde_json::to_value(stage_tool_resources_report_schema())?,
    )
}

#[test]
fn stage_tool_resource_contracts_admit_exact_closure() -> anyhow::Result<()> {
    validate_stage_tool_resources_config(&valid_config())?;
    validate_stage_tool_resources_report(&valid_report())?;
    Ok(())
}

#[test]
fn stage_tool_resource_contracts_reject_identity_resource_and_aggregate_drift() {
    let mut config = valid_config();
    config.rows[0].threads = 0;
    assert!(validate_stage_tool_resources_config(&config).is_err());

    let mut config = valid_config();
    config.rows.push(config.rows[0].clone());
    assert!(validate_stage_tool_resources_config(&config).is_err());

    let mut report = valid_report();
    report.row_count += 1;
    assert!(validate_stage_tool_resources_report(&report).is_err());

    let mut report = valid_report();
    report.config_path = "../outside.toml".to_string();
    assert!(validate_stage_tool_resources_report(&report).is_err());
}

fn valid_row() -> StageToolResourceRow {
    StageToolResourceRow {
        domain: "fastq".to_string(),
        stage_id: "fastq.validate_reads".to_string(),
        tool_id: "seqkit".to_string(),
        threads: 2,
        memory_gb: 4,
        walltime_minutes: 10,
        scratch_gb: 2,
        resource_origin: "tool_constraints_with_stage_walltime_profile".to_string(),
    }
}

fn valid_config() -> StageToolResourcesConfig {
    StageToolResourcesConfig {
        schema_version: LOCAL_STAGE_TOOL_RESOURCES_SCHEMA_V1.to_string(),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE.to_string(),
        rows: vec![valid_row()],
    }
}

fn valid_report() -> StageToolResourcesReport {
    StageToolResourcesReport {
        schema_version: STAGE_TOOL_RESOURCES_REPORT_SCHEMA_V1.to_string(),
        config_path: "configs/bench/local/stage-tool-resources.toml".to_string(),
        classification_scope: STAGE_TOOL_RESOURCES_SCOPE.to_string(),
        row_count: 1,
        benchmark_ready_row_count: 1,
        nonzero_resource_row_count: 1,
        domain_counts: BTreeMap::from([("fastq".to_string(), 1)]),
        resource_origin_counts: BTreeMap::from([(
            "tool_constraints_with_stage_walltime_profile".to_string(),
            1,
        )]),
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
