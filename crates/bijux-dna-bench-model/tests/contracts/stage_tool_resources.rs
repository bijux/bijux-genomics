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

#[test]
fn hpc_resource_projection_checks_units_and_preserves_domain_defaults() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{project_hpc_job_resources, resolve_hpc_job_resources};
    let mut config = valid_config();
    config.rows[0].walltime_minutes = 125;
    let hints = project_hpc_job_resources(&config)?;
    let selected = resolve_hpc_job_resources(&hints, "fastq", "fastq.validate_reads", "seqkit");
    assert_eq!(selected.memory_mb, 4096);
    assert_eq!(selected.cpus_per_task, 2);
    assert_eq!(selected.time_limit, "02:05:00");
    for (domain, cpus, memory, scratch) in
        [("fastq", 4, 2048, 2), ("bam", 3, 2048, 2), ("vcf", 1, 2048, 2), ("unknown", 1, 1024, 1)]
    {
        let fallback = resolve_hpc_job_resources(&hints, domain, "missing", "missing");
        assert_eq!(
            (fallback.cpus_per_task, fallback.memory_mb, fallback.scratch_gb),
            (cpus, memory, scratch)
        );
        assert_eq!(fallback.time_limit, "00:20:00");
    }
    config.rows[0].memory_gb = u32::MAX / 1024;
    assert!(project_hpc_job_resources(&config).is_ok());
    config.rows[0].memory_gb += 1;
    assert!(project_hpc_job_resources(&config).is_err());
    config.rows[0].memory_gb = 4;
    config.rows[0].threads = 0;
    assert!(project_hpc_job_resources(&config).is_err());
    Ok(())
}

#[test]
fn fastq_walltime_profiles_preserve_tool_precedence_and_fallback() {
    use bijux_dna_bench_model::fastq_benchmark_walltime_minutes;
    use bijux_dna_core::id_catalog::{
        FASTQ_CORRECT, FASTQ_DEPLETE_RRNA, FASTQ_INFER_ASVS, FASTQ_TRIM, FASTQ_VALIDATE_READS,
    };

    for (stage, tool, expected_minutes) in [
        (FASTQ_INFER_ASVS, "dada2", 45),
        (FASTQ_INFER_ASVS, "unprofiled_tool", 15),
        (FASTQ_CORRECT, "dada2", 30),
        (FASTQ_VALIDATE_READS, "dada2", 10),
        (FASTQ_DEPLETE_RRNA, "sortmerna", 20),
        (FASTQ_TRIM, "cutadapt", 15),
        ("", "dada2", 15),
    ] {
        assert_eq!(
            fastq_benchmark_walltime_minutes(stage, tool),
            expected_minutes,
            "walltime profile for {stage} / {tool}"
        );
    }
}

#[test]
fn vcf_walltime_profiles_distinguish_panel_workflows_from_standard_stages() {
    use bijux_dna_bench_model::vcf_benchmark_walltime_minutes;
    use bijux_dna_domain_vcf::VcfDomainStage;

    for (stage, expected_minutes) in [
        (VcfDomainStage::Stats, 20),
        (VcfDomainStage::PrepareReferencePanel, 30),
        (VcfDomainStage::Phasing, 60),
        (VcfDomainStage::Impute, 60),
        (VcfDomainStage::ImputationMetrics, 60),
    ] {
        assert_eq!(vcf_benchmark_walltime_minutes(stage), expected_minutes);
    }
}
