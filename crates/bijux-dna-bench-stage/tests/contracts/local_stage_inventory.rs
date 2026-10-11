use std::collections::BTreeMap;

use bijux_dna_bench_stage::{
    benchmark_local_all_domain_stage_inventory_schema, benchmark_local_stage_inventory_schema,
    validate_benchmark_local_all_domain_stage_inventory, validate_benchmark_local_stage_inventory,
    BenchmarkLocalAllDomainStageInventory, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry, BenchmarkLocalStageReadinessKind,
    BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};

fn inventory(domain: &str, stage_id: &str) -> BenchmarkLocalStageInventory {
    BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.to_string(),
        stage_matrix_schema_version: format!("bijux.bench.{domain}.local_stage_matrix.v1"),
        stage_matrix_path: format!("configs/bench/local/{domain}-stage-matrix.toml"),
        stage_count: 1,
        stages: vec![BenchmarkLocalStageInventoryEntry {
            stage_id: stage_id.to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
        }],
    }
}

fn all_domain() -> BenchmarkLocalAllDomainStageInventory {
    BenchmarkLocalAllDomainStageInventory {
        schema_version: BENCHMARK_LOCAL_ALL_DOMAIN_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/all-domain-stage-list.json".to_string(),
        selected_domains: vec!["fastq".to_string(), "bam".to_string()],
        domain_counts: BTreeMap::from([("bam".to_string(), 1), ("fastq".to_string(), 1)]),
        total_stage_count: 2,
        inventories: vec![inventory("fastq", "fastq.report_qc"), inventory("bam", "bam.align")],
    }
}

#[test]
fn local_stage_inventories_round_trip_and_match_model_schemas() -> anyhow::Result<()> {
    let report = all_domain();
    validate_benchmark_local_all_domain_stage_inventory(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalAllDomainStageInventory>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    assert_schema("local-stage-inventory.schema.json", benchmark_local_stage_inventory_schema())?;
    assert_schema(
        "local-all-domain-stage-inventory.schema.json",
        benchmark_local_all_domain_stage_inventory_schema(),
    )
}

#[test]
fn local_stage_inventories_reject_domain_and_count_drift() {
    let mut wrong_domain = inventory("fastq", "bam.align");
    assert!(validate_benchmark_local_stage_inventory(&wrong_domain).is_err());
    wrong_domain.domain = "unknown".to_string();
    assert!(validate_benchmark_local_stage_inventory(&wrong_domain).is_err());

    let mut wrong_count = all_domain();
    wrong_count.domain_counts.insert("fastq".to_string(), 2);
    assert!(validate_benchmark_local_all_domain_stage_inventory(&wrong_count).is_err());
}

fn assert_schema(file_name: &str, schema: schemars::Schema) -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(schema)?;
    let path = repository_root.join("benchmarks/contracts/schemas").join(file_name);
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    anyhow::ensure!(checked == generated, "{file_name} drifted from model authority");
    Ok(())
}

#[test]
fn local_matrix_inputs_retain_governed_order_and_round_trip() -> anyhow::Result<()> {
    use bijux_dna_bench_stage::BenchmarkLocalStageMatrixConfig;
    let root = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (domain, count) in [("fastq", 27), ("bam", 24)] {
        let path = root.join(format!("configs/bench/local/{domain}-stage-matrix.toml"));
        let text = std::fs::read_to_string(path)?;
        let original: toml::Value = toml::from_str(&text)?;
        let parsed: BenchmarkLocalStageMatrixConfig = toml::from_str(&text)?;
        assert_eq!(parsed.stages.len(), count);
        assert_eq!(parsed.schema_version, format!("bijux.bench.{domain}.local_stage_matrix.v1"));
        let encoded = toml::to_string(&parsed)?;
        assert_eq!(toml::from_str::<toml::Value>(&encoded)?, original);
        assert_eq!(
            serde_json::from_slice::<BenchmarkLocalStageMatrixConfig>(&serde_json::to_vec(
                &parsed
            )?)?,
            parsed
        );
    }
    Ok(())
}

#[test]
fn local_matrix_inputs_reject_unknown_fields_missing_values_and_invalid_readiness() {
    use bijux_dna_bench_stage::BenchmarkLocalStageMatrixConfig;
    use serde_json::json;
    let valid = json!({"schema_version": "bijux.bench.fastq.local_stage_matrix.v1", "stages": [{"stage_id": "fastq.validate_reads", "readiness_kind": "smoke"}]});
    let mut unknown_root = valid.clone();
    unknown_root["unexpected"] = json!(true);
    let mut unknown_stage = valid.clone();
    unknown_stage["stages"][0]["unexpected"] = json!(true);
    let mut invalid_readiness = valid.clone();
    invalid_readiness["stages"][0]["readiness_kind"] = json!("unsupported");
    let mut missing_stage = valid.clone();
    missing_stage["stages"][0] = json!({"readiness_kind": "smoke"});
    let missing_version = json!({"stages": valid["stages"]});
    for input in [unknown_root, unknown_stage, invalid_readiness, missing_stage, missing_version] {
        assert!(serde_json::from_value::<BenchmarkLocalStageMatrixConfig>(input).is_err());
    }
}
