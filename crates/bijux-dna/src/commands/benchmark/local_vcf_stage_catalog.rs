use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bijux_dna_domain_vcf::contracts::stage_io_contract;
use bijux_dna_domain_vcf::{DomainSupportStatus, VcfDomainStage, VCF_STAGE_ORDER_DOWNSTREAM};
use bijux_dna_stages_vcf::stage_specs::{vcf_stage_catalog, VcfStageSpec};
use serde::{Deserialize, Serialize};

use crate::commands::cli::parse;
use crate::commands::cli::render;

pub(crate) const DEFAULT_VCF_STAGE_CATALOG_PATH: &str =
    "benchmarks/configs/local/vcf-stage-catalog.toml";
const LOCAL_VCF_STAGE_CATALOG_SCHEMA_VERSION: &str = "bijux.bench.vcf.local_stage_catalog.v1";
const LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_VERSION: &str =
    "bijux.bench.local_vcf_stage_catalog.v1";

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct LocalVcfStageCatalogConfig {
    pub(crate) schema_version: String,
    pub(crate) rows: Vec<VcfStageCatalogRow>,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VcfStageCatalogRow {
    pub(crate) stage_id: String,
    pub(crate) stage_name: String,
    pub(crate) support_status: String,
    pub(crate) default_tool_id: String,
    pub(crate) metrics_schema_id: String,
    pub(crate) input_types: Vec<String>,
    pub(crate) output_types: Vec<String>,
    pub(crate) required_assets: Vec<String>,
    pub(crate) benchmark_category: String,
    pub(crate) local_smoke_mode: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfStageCatalogReport {
    pub(crate) schema_version: &'static str,
    pub(crate) config_path: String,
    pub(crate) stage_count: usize,
    pub(crate) supported_stage_count: usize,
    pub(crate) planned_stage_count: usize,
    pub(crate) rows: Vec<VcfStageCatalogRow>,
}

pub(crate) fn run_render_vcf_stage_catalog(
    args: &parse::BenchLocalRenderVcfStageCatalogArgs,
) -> Result<()> {
    let repo_root = std::env::current_dir().context("resolve current directory")?;
    let report = render_vcf_stage_catalog(
        &repo_root,
        args.output.clone().unwrap_or_else(|| PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_PATH)),
    )?;

    if args.json {
        render::json::print_pretty(&report)?;
    } else {
        println!("{}", report.config_path);
    }
    Ok(())
}

pub(crate) fn render_vcf_stage_catalog(
    repo_root: &Path,
    output_path: PathBuf,
) -> Result<LocalVcfStageCatalogReport> {
    let output_path = repo_relative_path(repo_root, &output_path);
    let rows = build_vcf_stage_catalog_rows()?;
    let config = LocalVcfStageCatalogConfig {
        schema_version: LOCAL_VCF_STAGE_CATALOG_SCHEMA_VERSION.to_string(),
        rows: rows.clone(),
    };
    let rendered = toml::to_string_pretty(&config).context("serialize VCF stage catalog TOML")?;

    if let Some(parent) = output_path.parent() {
        fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::atomic_write_bytes(&output_path, rendered.as_bytes())?;

    let supported_stage_count = rows.iter().filter(|row| row.support_status == "supported").count();
    let planned_stage_count = rows.iter().filter(|row| row.support_status == "planned").count();

    Ok(LocalVcfStageCatalogReport {
        schema_version: LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_VERSION,
        config_path: path_relative_to_repo(repo_root, &output_path),
        stage_count: rows.len(),
        supported_stage_count,
        planned_stage_count,
        rows,
    })
}

pub(crate) fn build_vcf_stage_catalog_rows() -> Result<Vec<VcfStageCatalogRow>> {
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
    let mut assets = BTreeSet::<String>::new();

    if required_indices.contains(&"bam.bai") {
        assets.insert("bam_index".to_string());
        assets.insert("reference_dict".to_string());
        assets.insert("reference_fai".to_string());
        assets.insert("reference_fasta".to_string());
    }

    if required_indices.contains(&"vcf.tbi") {
        assets.insert("vcf_index".to_string());
    }

    match stage {
        VcfDomainStage::PrepareReferencePanel => {
            assets.insert("genetic_map".to_string());
            assets.insert("reference_dict".to_string());
            assets.insert("reference_fai".to_string());
            assets.insert("reference_fasta".to_string());
            assets.insert("reference_panel_lock".to_string());
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

fn unique_data_types<'a>(data_types: impl Iterator<Item = &'a str>) -> Vec<String> {
    data_types.collect::<BTreeSet<_>>().into_iter().map(str::to_string).collect()
}

fn validate_stage_support_status(stage: VcfDomainStage, spec: &VcfStageSpec) -> Result<()> {
    let expected_status = match stage.taxonomy().status {
        DomainSupportStatus::Supported => "supported",
        DomainSupportStatus::Planned => "planned",
    };
    if spec.status != expected_status {
        return Err(anyhow!(
            "VCF stage `{}` declares status `{}` in stage specs but taxonomy requires `{expected_status}`",
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
    let row_stage_ids = rows.iter().map(|row| row.stage_id.as_str()).collect::<BTreeSet<_>>();
    let spec_stage_ids = spec_by_stage_id.keys().copied().collect::<BTreeSet<_>>();
    if row_stage_ids != spec_stage_ids {
        let missing = spec_stage_ids.difference(&row_stage_ids).copied().collect::<Vec<_>>();
        let stale = row_stage_ids.difference(&spec_stage_ids).copied().collect::<Vec<_>>();
        return Err(anyhow!(
            "VCF stage catalog rows drifted from stage specs; missing={missing:?} stale={stale:?}"
        ));
    }

    let domain_stage_ids =
        VcfDomainStage::all().iter().map(|stage| stage.as_str()).collect::<BTreeSet<_>>();
    if row_stage_ids != domain_stage_ids {
        let missing = domain_stage_ids.difference(&row_stage_ids).copied().collect::<Vec<_>>();
        let stale = row_stage_ids.difference(&domain_stage_ids).copied().collect::<Vec<_>>();
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::{
        build_vcf_stage_catalog_rows, render_vcf_stage_catalog, DEFAULT_VCF_STAGE_CATALOG_PATH,
        LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_VERSION,
    };

    fn repo_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../..")
            .canonicalize()
            .expect("canonicalize repo root")
    }

    #[test]
    fn vcf_stage_catalog_rows_cover_domain_and_stage_specs() {
        let rows = build_vcf_stage_catalog_rows().expect("build VCF stage catalog rows");

        assert_eq!(rows.len(), 20);
        assert_eq!(
            rows.first().map(|row| row.stage_id.as_str()),
            Some("vcf.prepare_reference_panel")
        );
        assert_eq!(rows.last().map(|row| row.stage_id.as_str()), Some("vcf.stats"));

        let prepare_reference_panel = rows
            .iter()
            .find(|row| row.stage_id == "vcf.prepare_reference_panel")
            .expect("prepare reference panel row");
        assert_eq!(prepare_reference_panel.stage_name, "Reference Panel Preparation");
        assert_eq!(prepare_reference_panel.input_types, vec!["vcf".to_string()]);
        assert_eq!(prepare_reference_panel.output_types, vec!["vcf".to_string()]);
        assert_eq!(
            prepare_reference_panel.required_assets,
            vec![
                "genetic_map".to_string(),
                "reference_dict".to_string(),
                "reference_fai".to_string(),
                "reference_fasta".to_string(),
                "reference_panel_lock".to_string(),
                "vcf_index".to_string(),
            ]
        );
        assert_eq!(prepare_reference_panel.benchmark_category, "reference_panel_preparation");
        assert_eq!(prepare_reference_panel.local_smoke_mode, "vcf_reference_panel");

        let call_gl = rows.iter().find(|row| row.stage_id == "vcf.call_gl").expect("call_gl row");
        assert_eq!(call_gl.input_types, vec!["bam".to_string()]);
        assert_eq!(call_gl.output_types, vec!["vcf".to_string()]);
        assert_eq!(
            call_gl.required_assets,
            vec![
                "bam_index".to_string(),
                "reference_dict".to_string(),
                "reference_fai".to_string(),
                "reference_fasta".to_string(),
            ]
        );
        assert_eq!(call_gl.benchmark_category, "variant_calling");
        assert_eq!(call_gl.local_smoke_mode, "bam_bundle");

        let phasing = rows.iter().find(|row| row.stage_id == "vcf.phasing").expect("phasing row");
        assert_eq!(phasing.local_smoke_mode, "vcf_cohort_with_panel");
        assert!(
            phasing.required_assets.contains(&"genetic_map".to_string())
                && phasing.required_assets.contains(&"reference_panel_lock".to_string()),
            "phasing rows must keep panel-aware smoke inputs explicit"
        );

        let population_structure = rows
            .iter()
            .find(|row| row.stage_id == "vcf.population_structure")
            .expect("population structure row");
        assert_eq!(population_structure.output_types, vec!["json".to_string()]);
        assert!(
            population_structure.required_assets.contains(&"sample_metadata_manifest".to_string()),
            "population structure rows must keep sample metadata explicit"
        );
    }

    #[test]
    fn vcf_stage_catalog_report_writes_governed_config() {
        let repo_root = repo_root();
        let report =
            render_vcf_stage_catalog(&repo_root, PathBuf::from(DEFAULT_VCF_STAGE_CATALOG_PATH))
                .expect("render VCF stage catalog");

        assert_eq!(report.schema_version, LOCAL_VCF_STAGE_CATALOG_REPORT_SCHEMA_VERSION);
        assert_eq!(report.config_path, DEFAULT_VCF_STAGE_CATALOG_PATH);
        assert_eq!(report.stage_count, 20);
        assert_eq!(report.supported_stage_count, 8);
        assert_eq!(report.planned_stage_count, 12);
    }
}
