mod domain;
mod publication;
use crate::{
    validate_benchmark_local_stage_inventory, BenchmarkLocalStageInventory,
    BenchmarkLocalStageInventoryEntry, BenchmarkLocalStageMatrixConfig,
    BenchmarkLocalStageReadinessKind as LocalStageReadinessKind,
    BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1,
};
use anyhow::{anyhow, Context, Result};
pub use domain::BenchmarkLocalStageInventoryDomain;
pub use publication::publish_inventories;
use std::{collections::BTreeSet, fs, path::Path};

/// Load the declared FASTQ or BAM matrix from an explicit repository root.
///
/// # Errors
/// Refuses malformed, unsupported or out-of-domain declarations.
pub fn load_matrix_backed_stage_inventory(
    repo_root: &Path,
    domain: BenchmarkLocalStageInventoryDomain,
) -> Result<BenchmarkLocalStageInventory> {
    anyhow::ensure!(
        domain != BenchmarkLocalStageInventoryDomain::Vcf,
        "VCF inventory requires the canonical row adapter"
    );
    let matrix_path = repo_root.join(domain.matrix_relative_path());
    let raw = fs::read_to_string(&matrix_path)
        .with_context(|| format!("read {}", matrix_path.display()))?;
    let matrix: BenchmarkLocalStageMatrixConfig =
        toml::from_str(&raw).with_context(|| format!("parse {}", matrix_path.display()))?;

    inventory_from_entries(
        domain,
        &matrix_path,
        matrix.schema_version,
        matrix.stages.into_iter().map(|entry| (entry.stage_id, entry.readiness_kind)),
    )
}

/// Admit stage rows supplied by a canonical domain matrix adapter.
///
/// # Errors
/// Refuses incompatible schema versions, duplicate identities and invalid inventories.
pub fn inventory_from_entries(
    domain: BenchmarkLocalStageInventoryDomain,
    matrix_path: &Path,
    schema_version: String,
    entries: impl Iterator<Item = (String, LocalStageReadinessKind)>,
) -> Result<BenchmarkLocalStageInventory> {
    if schema_version != domain.expected_matrix_schema_version() {
        return Err(anyhow!(
            "{} declares `{}` but `{}` is required for `{}` inventory",
            matrix_path.display(),
            schema_version,
            domain.expected_matrix_schema_version(),
            domain.as_str()
        ));
    }
    let stages = validate_matrix_backed_stage_entries(matrix_path, domain, entries)?;
    let inventory = BenchmarkLocalStageInventory {
        schema_version: BENCHMARK_LOCAL_STAGE_INVENTORY_SCHEMA_V1.to_string(),
        domain: domain.as_str().to_string(),
        stage_matrix_schema_version: schema_version,
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
