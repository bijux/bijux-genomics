//! Owner: bijux-dna-bench
//! Repository acquisition for pipeline stage and profile inventory evidence.

use super::{
    load_benchmark_local_stage_inventory as load_local_stage_inventory,
    BenchmarkLocalStageInventoryDomain as BenchLocalDomain,
};
use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    BenchmarkLocalPipelineDagDomain as LocalPipelineDagDomain,
    BenchmarkLocalStageReadinessKind as LocalStageReadinessKind, LocalVcfStageMatrixConfig,
    VcfStageMatrixRow,
};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// Load readiness declarations for the selected pipeline domains.
///
/// # Errors
/// Rejects unreadable or malformed governed inventories and unsupported VCF schemas.
pub fn load_benchmark_pipeline_inventory_index(
    repo_root: &Path,
    domain: LocalPipelineDagDomain,
) -> Result<BTreeMap<String, LocalStageReadinessKind>> {
    let mut inventory_index = BTreeMap::new();
    let inventory_domains = match domain {
        LocalPipelineDagDomain::Fastq => vec![BenchLocalDomain::Fastq],
        LocalPipelineDagDomain::Bam => vec![BenchLocalDomain::Bam],
        LocalPipelineDagDomain::Vcf => Vec::new(),
        LocalPipelineDagDomain::Cross => vec![BenchLocalDomain::Fastq, BenchLocalDomain::Bam],
    };

    for inventory_domain in inventory_domains {
        let inventory = load_local_stage_inventory(repo_root, inventory_domain)?;
        for stage in inventory.stages {
            inventory_index.insert(stage.stage_id, stage.readiness_kind);
        }
    }

    if matches!(domain, LocalPipelineDagDomain::Vcf | LocalPipelineDagDomain::Cross) {
        inventory_index.extend(load_local_vcf_pipeline_inventory_index(repo_root)?);
    }

    Ok(inventory_index)
}

fn load_local_vcf_pipeline_inventory_index(
    repo_root: &Path,
) -> Result<BTreeMap<String, LocalStageReadinessKind>> {
    let matrix_path = repo_root.join("configs/bench/local/vcf-stage-matrix.toml");
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != "bijux.bench.vcf.local_stage_matrix.v1" {
        return Err(anyhow!(
            "{} declares `{}` but `bijux.bench.vcf.local_stage_matrix.v1` is required for `vcf` local pipeline inventory",
            matrix_path.display(),
            matrix.schema_version
        ));
    }

    let mut inventory_index = BTreeMap::new();
    for row in matrix.rows {
        inventory_index.entry(row.stage_id).or_insert(LocalStageReadinessKind::Smoke);
    }
    Ok(inventory_index)
}

/// Load the governed VCF matrix used by pipeline profile validation.
///
/// # Errors
/// Rejects unreadable or malformed input and unsupported matrix schemas.
pub fn load_benchmark_pipeline_vcf_stage_matrix_index(
    repo_root: &Path,
) -> Result<BTreeMap<String, VcfStageMatrixRow>> {
    let matrix_path = repo_root.join("configs/bench/local/vcf-stage-matrix.toml");
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != "bijux.bench.vcf.local_stage_matrix.v1" {
        return Err(anyhow!(
            "{} declares `{}` but `bijux.bench.vcf.local_stage_matrix.v1` is required for VCF stage-matrix validation",
            matrix_path.display(),
            matrix.schema_version
        ));
    }

    Ok(matrix.rows.into_iter().map(|row| (row.stage_id.clone(), row)).collect::<BTreeMap<_, _>>())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vcf_inventory_and_profile_evidence_preserve_explicit_input_schema_errors() -> Result<()> {
        let root = tempfile::tempdir()?;
        let matrix = root.path().join("configs/bench/local/vcf-stage-matrix.toml");
        assert!(load_benchmark_pipeline_inventory_index(root.path(), LocalPipelineDagDomain::Vcf)
            .is_err());
        let repository = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let raw = fs::read_to_string(repository.join("configs/bench/local/vcf-stage-matrix.toml"))?;
        fs::create_dir_all(matrix.parent().ok_or_else(|| anyhow!("matrix parent"))?)?;
        fs::write(&matrix, &raw)?;
        let inventory =
            load_benchmark_pipeline_inventory_index(root.path(), LocalPipelineDagDomain::Vcf)?;
        let profile = load_benchmark_pipeline_vcf_stage_matrix_index(root.path())?;
        assert!(!inventory.is_empty());
        assert_eq!(inventory.keys().collect::<Vec<_>>(), profile.keys().collect::<Vec<_>>());
        for invalid in [
            raw.replace("bijux.bench.vcf.local_stage_matrix.v1", "unknown.schema"),
            "not toml".into(),
        ] {
            fs::write(&matrix, invalid)?;
            assert!(load_benchmark_pipeline_inventory_index(
                root.path(),
                LocalPipelineDagDomain::Vcf
            )
            .is_err());
            assert!(load_benchmark_pipeline_vcf_stage_matrix_index(root.path()).is_err());
        }
        Ok(())
    }
}
