//! Owner: bijux-dna-bench
//! Admission and persistence of the governed local VCF stage catalog declaration.

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use bijux_dna_bench_model::{
    validate_local_vcf_stage_catalog_config, validate_local_vcf_stage_catalog_report,
    LocalVcfStageCatalogConfig, LocalVcfStageCatalogReport, VcfStageCatalogRow,
    LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1,
};

pub const DEFAULT_VCF_STAGE_CATALOG_PATH: &str = "configs/bench/local/vcf-stage-catalog.toml";
const DECLARED_VCF_STAGE_CATALOG: &str =
    include_str!("../../../../configs/bench/local/vcf-stage-catalog.toml");
const LOCAL_VCF_STAGE_CATALOG_HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-bench\n\
# purpose = Governed VCF local benchmark stage catalog.\n\
# authority = bijux-dna-bench\n\
# stability = evolving\n\
# last_updated = 2026-06-25\n\n";

/// Render the admitted local VCF stage catalog at a repository-relative or absolute path.
///
/// # Errors
/// Returns an error when the declaration is invalid or the catalog cannot be persisted.
pub fn render_vcf_stage_catalog(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageCatalogReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = build_vcf_stage_catalog_rows()?;
    let config = LocalVcfStageCatalogConfig {
        schema_version: bijux_dna_bench_model::LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1.to_string(),
        rows: rows.clone(),
    };
    let body = toml::to_string_pretty(&config).context("serialize VCF stage catalog TOML")?;
    let rendered = format!("{LOCAL_VCF_STAGE_CATALOG_HEADER}{body}");

    if let Some(parent) = output_path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let supported_stage_count = rows.iter().filter(|row| row.support_status == "supported").count();
    let planned_stage_count = rows.iter().filter(|row| row.support_status == "planned").count();
    let report = LocalVcfStageCatalogReport {
        schema_version: LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1.to_string(),
        config_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: rows.len(),
        supported_stage_count,
        planned_stage_count,
        rows,
    };
    validate_local_vcf_stage_catalog_report(&report)?;
    Ok(report)
}

/// Admit the ordered VCF stage catalog from its checked declaration.
///
/// # Errors
/// Returns an error when the declaration violates its model-owned schema or semantics.
pub fn build_vcf_stage_catalog_rows() -> Result<Vec<VcfStageCatalogRow>> {
    let config: LocalVcfStageCatalogConfig =
        toml::from_str(DECLARED_VCF_STAGE_CATALOG).context("parse declared VCF stage catalog")?;
    validate_local_vcf_stage_catalog_config(&config)?;
    Ok(config.rows)
}

fn repo_relative_path(repo_root: &Path, candidate: &Path) -> PathBuf {
    if candidate.is_absolute() {
        candidate.to_path_buf()
    } else {
        repo_root.join(candidate)
    }
}

fn path_relative_to_repo(repo_root: &Path, path: &Path) -> String {
    path.strip_prefix(repo_root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
