use std::fs;
use std::path::Path;
use std::sync::{Mutex, OnceLock};

use anyhow::{anyhow, bail, Result};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_db_ref::public_api::{
    resolve_map, resolve_map_lock, resolve_panel, resolve_panel_lock,
    validate_imputation_tool_compatibility,
};
use bijux_dna_domain_vcf::contracts::{ContigSpec, SpeciesContext};
use bijux_dna_stage_contract::canonical_tool_bindings;
use serde::Serialize;

use super::local_stage_result_manifest::{
    path_relative_to_repo, BenchStageResultCommandV1, BenchStageResultManifestV1,
    BenchStageResultOutputV1, BenchStageResultResourceMetricSource,
    BenchStageResultResourceMetricsV1, BenchStageResultRuntimeV1, BenchStageResultStatus,
    BenchStageResultToolV1, BENCH_STAGE_RESULT_SCHEMA_VERSION,
};
use super::local_vcf_stage_matrix::build_vcf_stage_matrix_rows;

pub(crate) const GOVERNED_VCF_PANEL_SPECIES_ID: &str = "Homo sapiens";
pub(crate) const GOVERNED_VCF_PANEL_BUILD_ID: &str = "GRCh38";
pub(crate) const GOVERNED_VCF_PANEL_ID: &str = "hsapiens_grch38_1kg_3202_chr22";
pub(crate) const GOVERNED_VCF_MAP_ID: &str = "hsapiens_grch38_glimpse_chr22";
pub(crate) const DEFAULT_STAGE_RESULT_NAME: &str = "stage-result.json";

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct GovernedVcfPanelWorkflowSmokeContract {
    pub(crate) stage_id: String,
    pub(crate) tool_id: String,
    pub(crate) corpus_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct ResolvedVcfPanelAssetContract {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_class: &'static str,
    pub(crate) species_id: String,
    pub(crate) build_id: String,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) panel_files: Vec<String>,
    pub(crate) map_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub(crate) struct LocalVcfPanelFixtureReport {
    pub(crate) schema_version: &'static str,
    pub(crate) fixture_class: &'static str,
    pub(crate) panel_id: String,
    pub(crate) map_id: String,
    pub(crate) materialized_files: Vec<String>,
}

pub(crate) fn resolve_governed_vcf_panel_workflow_smoke_contract(
    requested_stage_id: &str,
    requested_tool_id: &str,
    expected_output_id: &str,
) -> Result<GovernedVcfPanelWorkflowSmokeContract> {
    let matrix_row = build_vcf_stage_matrix_rows()?
        .into_iter()
        .find(|row| row.stage_id == requested_stage_id)
        .ok_or_else(|| anyhow!("VCF stage matrix is missing `{requested_stage_id}`"))?;
    let bindings = canonical_tool_bindings()?;
    if bindings
        .binding(&StageId::new(requested_stage_id), &ToolId::new(requested_tool_id))
        .is_none()
    {
        bail!(
            "VCF panel workflow smoke has no canonical binding for `{requested_stage_id}` / `{requested_tool_id}`"
        );
    }
    let requires_governed_map_compatibility = requested_stage_id == "vcf.phasing"
        || (matches!(requested_stage_id, "vcf.impute" | "vcf.imputation_metrics")
            && requested_tool_id != "beagle");
    if requires_governed_map_compatibility {
        let panel = resolve_panel(
            GOVERNED_VCF_PANEL_SPECIES_ID,
            GOVERNED_VCF_PANEL_BUILD_ID,
            Some(GOVERNED_VCF_PANEL_ID),
        )?;
        let map = resolve_map(
            GOVERNED_VCF_PANEL_SPECIES_ID,
            GOVERNED_VCF_PANEL_BUILD_ID,
            Some(GOVERNED_VCF_MAP_ID),
        )?;
        validate_imputation_tool_compatibility(requested_tool_id, &panel, &map)?;
    }
    if matrix_row.corpus_id != "vcf_production_regression" {
        bail!(
            "VCF panel workflow smoke requires corpus `vcf_production_regression`, found `{}`",
            matrix_row.corpus_id
        );
    }
    if matrix_row.asset_profile_id != "vcf_cohort_with_panel" {
        bail!(
            "VCF panel workflow smoke requires asset profile `vcf_cohort_with_panel`, found `{}`",
            matrix_row.asset_profile_id
        );
    }
    if matrix_row.expected_outputs != vec![expected_output_id.to_string()] {
        bail!(
            "VCF panel workflow smoke expected outputs drifted for `{}`: {:?}",
            matrix_row.stage_id,
            matrix_row.expected_outputs
        );
    }

    Ok(GovernedVcfPanelWorkflowSmokeContract {
        stage_id: matrix_row.stage_id,
        tool_id: requested_tool_id.to_string(),
        corpus_id: matrix_row.corpus_id,
        panel_id: GOVERNED_VCF_PANEL_ID.to_string(),
        map_id: GOVERNED_VCF_MAP_ID.to_string(),
    })
}

pub(crate) fn resolve_governed_vcf_panel_asset_contract() -> Result<ResolvedVcfPanelAssetContract> {
    let panel = resolve_panel(
        GOVERNED_VCF_PANEL_SPECIES_ID,
        GOVERNED_VCF_PANEL_BUILD_ID,
        Some(GOVERNED_VCF_PANEL_ID),
    )?;
    let map = resolve_map(
        GOVERNED_VCF_PANEL_SPECIES_ID,
        GOVERNED_VCF_PANEL_BUILD_ID,
        Some(GOVERNED_VCF_MAP_ID),
    )?;
    let panel_lock = resolve_panel_lock(&panel)?;
    let map_lock = resolve_map_lock(&map)?;
    validate_imputation_tool_compatibility("glimpse", &panel, &map)?;

    Ok(ResolvedVcfPanelAssetContract {
        schema_version: "bijux.local_vcf_panel_asset_contract.v1",
        fixture_class: "local_smoke_metadata_only",
        species_id: panel.species_id,
        build_id: panel.build_id,
        panel_id: panel.id,
        map_id: map.id,
        panel_files: panel_lock.files.into_iter().map(|file| file.path).collect(),
        map_files: map_lock.files.into_iter().map(|file| file.path).collect(),
    })
}

pub(crate) fn materialize_local_vcf_panel_fixture(
    materialization_root: &Path,
) -> Result<LocalVcfPanelFixtureReport> {
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    let fixture_root = materialization_root.join("fixtures");
    let panel_root = fixture_root.join("panels").join(&contract.panel_id);
    let map_root = fixture_root.join("maps").join(&contract.map_id);
    let mut materialized_files = Vec::new();

    for (root, paths, asset_kind) in
        [(&panel_root, &contract.panel_files, "panel"), (&map_root, &contract.map_files, "map")]
    {
        for relative in paths {
            let target = root.join(relative);
            write_local_smoke_marker(&target, asset_kind, relative)?;
            materialized_files.push(target.display().to_string());
        }
    }
    if !contract.panel_files.iter().any(|path| path.ends_with("panel.m3vcf.gz")) {
        let target = panel_root.join("panel.m3vcf.gz");
        write_local_smoke_derived_marker(&target, "panel.vcf.gz", "minimac4_process_reference")?;
        materialized_files.push(target.display().to_string());
    }

    Ok(LocalVcfPanelFixtureReport {
        schema_version: "bijux.local_vcf_panel_fixture.v1",
        fixture_class: "local_smoke",
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        materialized_files,
    })
}

fn write_local_smoke_marker(target: &Path, asset_kind: &str, relative: &str) -> Result<()> {
    let parent = target
        .parent()
        .ok_or_else(|| anyhow!("local VCF fixture path has no parent: {}", target.display()))?;
    fs::create_dir_all(parent)?;
    fs::write(
        target,
        format!("fixture_class=local_smoke\nasset_kind={asset_kind}\nrelative_path={relative}\n"),
    )?;
    Ok(())
}

fn write_local_smoke_derived_marker(
    target: &Path,
    derived_from: &str,
    derivation: &str,
) -> Result<()> {
    let parent = target.parent().ok_or_else(|| {
        anyhow!("local VCF derived fixture path has no parent: {}", target.display())
    })?;
    fs::create_dir_all(parent)?;
    fs::write(
        target,
        format!(
            "fixture_class=local_smoke\nasset_kind=derived_panel\nderived_from={derived_from}\nderivation={derivation}\n"
        ),
    )?;
    Ok(())
}

pub(crate) fn with_local_vcf_panel_fixture_lock<T>(
    action: impl FnOnce() -> Result<T>,
) -> Result<T> {
    static MATERIALIZATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = MATERIALIZATION_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().map_err(|_| anyhow!("local VCF panel fixture lock is poisoned"))?;
    action()
}

pub(crate) fn governed_vcf_panel_species_context() -> SpeciesContext {
    SpeciesContext {
        species_id: GOVERNED_VCF_PANEL_SPECIES_ID.to_string(),
        build_id: GOVERNED_VCF_PANEL_BUILD_ID.to_string(),
        contig_set_digest: "3f2b2d7d76f3d8de2b8f0d6d9f0b1776c8b0f95f4135f2b5114634364b4f22cc"
            .to_string(),
        contigs: vec![
            ContigSpec { name: "1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "2".to_string(), length_bp: 242_193_529 },
            ContigSpec { name: "chr1".to_string(), length_bp: 248_956_422 },
            ContigSpec { name: "chr2".to_string(), length_bp: 242_193_529 },
        ],
        sex_system: "xy".to_string(),
        par_policy: "grch38_par".to_string(),
        default_coverage_regime: None,
    }
}

pub(crate) fn build_stage_result_manifest(
    repo_root: &Path,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
    command: &str,
    output_entries: &[(&str, String, &Path, &str)],
    started_at: &str,
    finished_at: &str,
    elapsed_seconds: f64,
) -> BenchStageResultManifestV1 {
    BenchStageResultManifestV1 {
        schema_version: BENCH_STAGE_RESULT_SCHEMA_VERSION.to_string(),
        stage_id: contract.stage_id.clone(),
        tool: BenchStageResultToolV1 { id: contract.tool_id.clone() },
        command: BenchStageResultCommandV1 { rendered: command.to_string() },
        runtime: BenchStageResultRuntimeV1 {
            mode: "local_smoke".to_string(),
            status: BenchStageResultStatus::Succeeded,
            started_at: started_at.to_string(),
            finished_at: finished_at.to_string(),
            elapsed_seconds,
            exit_code: 0,
        },
        resource_metrics: BenchStageResultResourceMetricsV1 {
            source: BenchStageResultResourceMetricSource::NotAvailable,
            memory_mb: None,
            cpu_threads: None,
        },
        outputs: output_entries
            .iter()
            .map(|(artifact_id, declared_path, realized_path, role)| BenchStageResultOutputV1 {
                artifact_id: (*artifact_id).to_string(),
                declared_path: declared_path.clone(),
                realized_path: path_relative_to_repo(repo_root, realized_path),
                role: (*role).to_string(),
                optional: false,
                exists: true,
            })
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        governed_vcf_panel_species_context, materialize_local_vcf_panel_fixture,
        resolve_governed_vcf_panel_asset_contract,
        resolve_governed_vcf_panel_workflow_smoke_contract,
    };

    #[test]
    fn governed_panel_workflow_contract_accepts_canonical_tool_substitution() {
        let contract = resolve_governed_vcf_panel_workflow_smoke_contract(
            "vcf.phasing",
            "shapeit5",
            "phased_vcf",
        )
        .expect("resolve contract");
        assert_eq!(contract.stage_id, "vcf.phasing");
        assert_eq!(contract.tool_id, "shapeit5");
        assert_eq!(contract.corpus_id, "vcf_production_regression");
        assert_eq!(contract.panel_id, "hsapiens_grch38_1kg_3202_chr22");
        assert_eq!(contract.map_id, "hsapiens_grch38_glimpse_chr22");

        let shapeit = resolve_governed_vcf_panel_workflow_smoke_contract(
            "vcf.phasing",
            "shapeit",
            "phased_vcf",
        )
        .expect("resolve Shapeit4 contract");
        assert_eq!(shapeit.stage_id, "vcf.phasing");
        assert_eq!(shapeit.tool_id, "shapeit");
    }

    #[test]
    fn governed_panel_workflow_contract_refuses_unknown_tool_binding() {
        let error = resolve_governed_vcf_panel_workflow_smoke_contract(
            "vcf.phasing",
            "not-a-phasing-tool",
            "phased_vcf",
        )
        .expect_err("unknown tool must be refused");
        assert!(error.to_string().contains("has no canonical binding"));
    }

    #[test]
    fn governed_imputation_contract_refuses_incompatible_panel_backend() {
        let error = resolve_governed_vcf_panel_workflow_smoke_contract(
            "vcf.impute",
            "minimac4",
            "imputed_vcf",
        )
        .expect_err("Minimac4 must be refused for a panel without M3VCF support");
        assert!(error.to_string().contains("not compatible with tool minimac4"));
    }

    #[test]
    fn governed_panel_workflow_asset_contract_resolves_without_materialization() {
        let report = resolve_governed_vcf_panel_asset_contract().expect("resolve asset contract");
        assert_eq!(report.panel_id, "hsapiens_grch38_1kg_3202_chr22");
        assert_eq!(report.map_id, "hsapiens_grch38_glimpse_chr22");
        assert_eq!(report.fixture_class, "local_smoke_metadata_only");
        assert!(!report.panel_files.is_empty());
        assert!(!report.map_files.is_empty());
    }

    #[test]
    fn local_panel_fixture_is_explicitly_classified() {
        let dir = tempfile::tempdir().expect("tempdir");
        let report = materialize_local_vcf_panel_fixture(dir.path()).expect("materialize fixture");
        assert_eq!(report.fixture_class, "local_smoke");
        assert!(report.materialized_files.iter().all(|path| std::path::Path::new(path).is_file()));
        let derived_m3vcf = report
            .materialized_files
            .iter()
            .find(|path| path.ends_with("/panel.m3vcf.gz"))
            .expect("derived M3VCF fixture");
        let marker = std::fs::read_to_string(derived_m3vcf).expect("read derived M3VCF marker");
        assert!(marker.contains("fixture_class=local_smoke"));
        assert!(marker.contains("derived_from=panel.vcf.gz"));
        assert!(marker.contains("derivation=minimac4_process_reference"));
    }

    #[test]
    fn governed_panel_species_context_keeps_chr_and_numeric_aliases() {
        let context = governed_vcf_panel_species_context();
        assert_eq!(context.species_id, "Homo sapiens");
        assert_eq!(context.build_id, "GRCh38");
        assert!(context.contigs.iter().any(|contig| contig.name == "1"));
        assert!(context.contigs.iter().any(|contig| contig.name == "chr1"));
    }
}
