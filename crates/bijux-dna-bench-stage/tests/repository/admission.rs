use anyhow::Result;
use bijux_dna_bench_stage::{
    repository::{
        load_matrix_backed_stage_inventory, publish_inventories,
        BenchmarkLocalStageInventoryDomain as Domain,
    },
    BenchmarkLocalStageInventory, BenchmarkLocalStageInventoryEntry,
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};
use std::fs;
#[test]
fn duplicate_domain_selection_refuses_before_directory_creation() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: "fastq".to_string(),
        stage_matrix_schema_version: "bijux.bench.fastq.local_stage_matrix.v1".to_string(),
        stage_matrix_path: "matrix.toml".to_string(),
        stage_count: 1,
        stages: vec![BenchmarkLocalStageInventoryEntry {
            stage_id: "fastq.trim_reads".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
        }],
    };
    let output = dir.path().join("absent/report.json");
    assert!(publish_inventories(dir.path(), vec![inventory.clone(), inventory], &output).is_err());
    assert!(!output.parent().ok_or_else(|| anyhow::anyhow!("output has no parent"))?.exists());
    Ok(())
}
#[test]
fn canonical_matrix_reader_refuses_wrong_domain_and_vcf_without_adapter() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("configs/bench/local");
    fs::create_dir_all(&path)?;
    fs::write(path.join("fastq-stage-matrix.toml"),"schema_version='bijux.bench.fastq.local_stage_matrix.v1'\n[[stages]]\nstage_id='bam.align'\nreadiness_kind='smoke'\n")?;
    assert!(load_matrix_backed_stage_inventory(dir.path(), Domain::Fastq).is_err());
    assert!(load_matrix_backed_stage_inventory(dir.path(), Domain::Vcf).is_err());
    Ok(())
}
