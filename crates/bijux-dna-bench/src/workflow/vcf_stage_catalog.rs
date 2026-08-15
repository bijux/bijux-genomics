//! Owner: bijux-dna-bench
//! Derivation and persistence of the governed local VCF stage catalog.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_bench_model::{
    validate_local_vcf_stage_catalog_config, validate_local_vcf_stage_catalog_report,
    LocalVcfStageCatalogConfig, LocalVcfStageCatalogReport, VcfStageCatalogRow,
    LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_V1, LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1,
};
use bijux_dna_domain_vcf::contracts::{
    stage_io_contract, BAM_INDEX_DATA_TYPE, VCF_INDEX_DATA_TYPE,
};
use bijux_dna_domain_vcf::{DomainSupportStatus, VcfDomainStage, VCF_STAGE_ORDER_DOWNSTREAM};
use bijux_dna_stages_vcf::stage_specs::{vcf_stage_catalog, VcfStageSpec};

pub const DEFAULT_VCF_STAGE_CATALOG_PATH: &str = "configs/bench/local/vcf-stage-catalog.toml";
const LOCAL_VCF_STAGE_CATALOG_HEADER: &str = "# schema_version = 1\n\
# owner = bijux-dna-bench\n\
# purpose = Governed VCF local benchmark stage catalog.\n\
# authority = bijux-dna-bench\n\
# stability = evolving\n\
# last_updated = 2026-06-25\n\n";

/// Render the governed local VCF stage catalog at a repository-relative or absolute path.
///
/// # Errors
/// Returns an error when authoritative stage contracts drift or the catalog cannot be persisted.
pub fn render_vcf_stage_catalog(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageCatalogReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = build_vcf_stage_catalog_rows()?;
    let config = LocalVcfStageCatalogConfig {
        schema_version: LOCAL_VCF_STAGE_CATALOG_SCHEMA_V1.to_string(),
        rows: rows.clone(),
    };
    validate_local_vcf_stage_catalog_config(&config)?;
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

/// Derive the ordered local VCF stage catalog from domain and stage specifications.
///
/// # Errors
/// Returns an error when stage order, taxonomy, stage specs, or I/O contracts disagree.
pub fn build_vcf_stage_catalog_rows() -> Result<Vec<VcfStageCatalogRow>> {
    let spec_by_stage_id =
        vcf_stage_catalog().iter().map(|spec| (spec.stage_id, spec)).collect::<BTreeMap<_, _>>();
    let mut rows = Vec::with_capacity(VCF_STAGE_ORDER_DOWNSTREAM.len());
    for stage in VCF_STAGE_ORDER_DOWNSTREAM {
        let spec = spec_by_stage_id.get(stage.as_str()).copied().ok_or_else(|| {
            anyhow!(
                "VCF stage catalog is missing authoritative stage `{}` from downstream order",
                stage.as_str()
            )
        })?;
        validate_stage_support_status(*stage, spec)?;
        rows.push(build_row(*stage, spec)?);
    }
    ensure_stage_set_parity(&rows, &spec_by_stage_id)?;
    Ok(rows)
}

fn build_row(stage: VcfDomainStage, spec: &VcfStageSpec) -> Result<VcfStageCatalogRow> {
    let io = stage_io_contract(stage)
        .ok_or_else(|| anyhow!("VCF stage `{}` is missing an IO contract", stage.as_str()))?;
    Ok(VcfStageCatalogRow {
        stage_id: stage.as_str().to_string(),
        stage_name: stage_name(stage).to_string(),
        support_status: spec.status.to_string(),
        default_tool_id: spec.default_tool_id.to_string(),
        metrics_schema_id: spec.metrics_schema.to_string(),
        input_types: unique_data_types(io.inputs.iter().map(|port| port.data_type)),
        output_types: unique_data_types(io.outputs.iter().map(|port| port.data_type)),
        required_assets: required_assets(stage, &io.required_indices),
        benchmark_category: benchmark_category(stage).to_string(),
        local_smoke_mode: local_smoke_mode(stage).to_string(),
    })
}

fn stage_name(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::PrepareReferencePanel => "Reference Panel Preparation",
        VcfDomainStage::Call => "Variant Calling",
        VcfDomainStage::CallGl => "Genotype Likelihood Calling",
        VcfDomainStage::CallDiploid => "Diploid Variant Calling",
        VcfDomainStage::CallPseudohaploid => "Pseudohaploid Calling",
        VcfDomainStage::DamageFilter => "Damage-Aware Variant Filtering",
        VcfDomainStage::Filter => "Variant Filtering",
        VcfDomainStage::GlPropagation => "Genotype Likelihood Propagation",
        VcfDomainStage::Qc => "Variant Quality Control",
        VcfDomainStage::Phasing => "Haplotype Phasing",
        VcfDomainStage::ImputationMetrics => "Imputation Quality Metrics",
        VcfDomainStage::Impute => "Imputed Genotype Refinement",
        VcfDomainStage::Postprocess => "VCF Postprocess Normalization",
        VcfDomainStage::PopulationStructure => "Population Structure Analysis",
        VcfDomainStage::Pca => "Principal Component Analysis",
        VcfDomainStage::Admixture => "Admixture Inference",
        VcfDomainStage::Roh => "Runs of Homozygosity",
        VcfDomainStage::Ibd => "Identity-by-Descent Inference",
        VcfDomainStage::Demography => "Demographic Inference",
        VcfDomainStage::Stats => "VCF Summary Statistics",
    }
}

fn benchmark_category(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::PrepareReferencePanel => "reference_panel_preparation",
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => "variant_calling",
        VcfDomainStage::DamageFilter => "damage_aware_filtering",
        VcfDomainStage::Filter | VcfDomainStage::Qc | VcfDomainStage::Stats => "quality_control",
        VcfDomainStage::GlPropagation => "likelihood_postprocess",
        VcfDomainStage::Phasing => "phasing",
        VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => "imputation",
        VcfDomainStage::Postprocess => "normalization",
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => {
            "population_structure"
        }
        VcfDomainStage::Roh => "runs_of_homozygosity",
        VcfDomainStage::Ibd => "identity_by_descent",
        VcfDomainStage::Demography => "demography",
    }
}

fn local_smoke_mode(stage: VcfDomainStage) -> &'static str {
    match stage {
        VcfDomainStage::PrepareReferencePanel => "vcf_reference_panel",
        VcfDomainStage::Call
        | VcfDomainStage::CallDiploid
        | VcfDomainStage::CallGl
        | VcfDomainStage::CallPseudohaploid => "bam_bundle",
        VcfDomainStage::DamageFilter
        | VcfDomainStage::Filter
        | VcfDomainStage::GlPropagation
        | VcfDomainStage::Postprocess => "vcf_single_sample",
        VcfDomainStage::Qc
        | VcfDomainStage::PopulationStructure
        | VcfDomainStage::Pca
        | VcfDomainStage::Admixture
        | VcfDomainStage::Roh
        | VcfDomainStage::Ibd
        | VcfDomainStage::Stats => "vcf_cohort",
        VcfDomainStage::Phasing | VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => {
            "vcf_cohort_with_panel"
        }
        VcfDomainStage::Demography => "json_ibd_segments",
    }
}

fn required_assets(stage: VcfDomainStage, required_indices: &[&str]) -> Vec<String> {
    let mut assets = BTreeSet::new();
    if required_indices.contains(&BAM_INDEX_DATA_TYPE) {
        for asset in ["bam_index", "reference_dict", "reference_fai", "reference_fasta"] {
            assets.insert(asset.to_string());
        }
    }
    if required_indices.contains(&VCF_INDEX_DATA_TYPE) {
        assets.insert("vcf_index".to_string());
    }
    match stage {
        VcfDomainStage::PrepareReferencePanel => {
            for asset in [
                "genetic_map",
                "reference_dict",
                "reference_fai",
                "reference_fasta",
                "reference_panel_lock",
            ] {
                assets.insert(asset.to_string());
            }
        }
        VcfDomainStage::Phasing | VcfDomainStage::ImputationMetrics | VcfDomainStage::Impute => {
            assets.insert("genetic_map".to_string());
            assets.insert("reference_panel_lock".to_string());
        }
        VcfDomainStage::PopulationStructure | VcfDomainStage::Pca | VcfDomainStage::Admixture => {
            assets.insert("sample_metadata_manifest".to_string());
        }
        _ => {}
    }
    assets.into_iter().collect()
}

fn unique_data_types<'a>(values: impl Iterator<Item = &'a str>) -> Vec<String> {
    values.collect::<BTreeSet<_>>().into_iter().map(str::to_string).collect()
}

fn validate_stage_support_status(stage: VcfDomainStage, spec: &VcfStageSpec) -> Result<()> {
    let expected = match stage.taxonomy().status {
        DomainSupportStatus::Supported => "supported",
        DomainSupportStatus::Planned => "planned",
    };
    if spec.status != expected {
        return Err(anyhow!(
            "VCF stage `{}` declares status `{}` in stage specs but taxonomy requires `{expected}`",
            stage.as_str(),
            spec.status
        ));
    }
    Ok(())
}

fn ensure_stage_set_parity(
    rows: &[VcfStageCatalogRow],
    spec_by_stage_id: &BTreeMap<&'static str, &VcfStageSpec>,
) -> Result<()> {
    let observed = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let specified = spec_by_stage_id.keys().copied().collect::<BTreeSet<_>>();
    if observed != specified {
        let missing = specified.difference(&observed).copied().collect::<Vec<_>>();
        let stale = observed.difference(&specified).copied().collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF stage catalog rows drifted from stage specs; missing={missing:?} stale={stale:?}"
        ));
    }
    let domain = VcfDomainStage::all().iter().map(|stage| stage.as_str()).collect::<BTreeSet<_>>();
    if observed != domain {
        let missing = domain.difference(&observed).copied().collect::<Vec<_>>();
        let stale = observed.difference(&domain).copied().collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF stage catalog rows drifted from domain stage set; missing={missing:?} stale={stale:?}"
        ));
    }
    Ok(())
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
