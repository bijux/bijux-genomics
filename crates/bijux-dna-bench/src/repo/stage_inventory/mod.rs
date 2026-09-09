//! Read and admit domain-local stage matrix declarations from an explicit repository root.

mod domain;
#[cfg(test)]
mod tests;

pub use domain::BenchmarkLocalStageInventoryDomain;

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    validate_benchmark_local_stage_inventory, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry, BenchmarkLocalStageReadinessKind as LocalStageReadinessKind,
    LocalVcfStageMatrixConfig, BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};
use serde::Deserialize;

const VCF_LOCAL_STAGE_READINESS_KIND: LocalStageReadinessKind = LocalStageReadinessKind::Smoke;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalStageMatrix {
    schema_version: String,
    stages: Vec<LocalStageMatrixEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocalStageMatrixEntry {
    stage_id: String,
    readiness_kind: LocalStageReadinessKind,
}

/// Load one domain's stage inventory without creating outputs.
///
/// # Errors
/// Returns an error for missing/malformed matrices, incompatible schema versions, or invalid stages.
pub fn load_benchmark_local_stage_inventory(
    repo_root: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
) -> Result<BenchmarkLocalStageInventory> {
    match domain {
        BenchmarkLocalStageInventoryDomain::Fastq | BenchmarkLocalStageInventoryDomain::Bam => {
            load_matrix_backed_stage_inventory(repo_root, domain)
        }
        BenchmarkLocalStageInventoryDomain::Vcf => load_vcf_stage_inventory(repo_root),
    }
}

fn load_matrix_backed_stage_inventory(
    repo_root: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
) -> Result<BenchmarkLocalStageInventory> {
    let matrix_path = repo_root.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalStageMatrix =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.stages.into_iter().map(|entry| (entry.stage_id, entry.readiness_kind)),
    )?;

    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.as_str().to_string(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    };
    validate_benchmark_local_stage_inventory(&inventory)?;
    Ok(inventory)
}

fn load_vcf_stage_inventory(repo_root: &Path) -> Result<BenchmarkLocalStageInventory> {
    let domain = BenchmarkLocalStageInventoryDomain::Vcf;
    let matrix_path = repo_root.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: LocalVcfStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    if matrix.schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            matrix.schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }

    let stages = validate_matrix_backed_stage_entries(
        &matrix_path,
        domain,
        matrix.rows.into_iter().map(|row| (row.stage_id, VCF_LOCAL_STAGE_READINESS_KIND)),
    )?;

    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.as_str().to_string(),
        stage_matrix_schema_version: matrix.schema_version,
        stage_matrix_path: domain.matrix_relative_path().to_string(),
        stage_count: stages.len(),
        stages,
    };
    validate_benchmark_local_stage_inventory(&inventory)?;
    Ok(inventory)
}

fn validate_matrix_backed_stage_entries(
    matrix_path: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
    entries: impl Iterator<Item = (String, LocalStageReadinessKind)>,
) -> Result<Vec<BenchmarkLocalStageInventoryEntry>> {
    let mut seen_stage_ids = BTreeSet::new();
    let mut stages = Vec::new();
    for (stage_id, readiness_kind) in entries {
        if !stage_id.starts_with(domain.stage_prefix()) {
            return Err(anyhow!(
                "{} contains out-of-domain stage `{}` for `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        if !seen_stage_ids.insert(stage_id.clone()) {
            return Err(anyhow!(
                "{} repeats stage `{}` in `{}` inventory",
                matrix_path.display(),
                stage_id,
                domain.as_str()
            ));
        }
        stages.push(BenchmarkLocalStageInventoryEntry { stage_id, readiness_kind });
    }
    Ok(stages)
}
