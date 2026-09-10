//! Owner: bijux-dna-bench-vcf
//! Panel authority observations and explicitly metadata-only marker fixtures.
use anyhow::{anyhow, Result};
use bijux_dna_db_ref::public_api::{
    resolve_map, resolve_map_lock, resolve_panel, resolve_panel_lock,
    validate_imputation_tool_compatibility,
};
use serde::Serialize;
use std::path::Path;
use std::sync::{Mutex, OnceLock};
pub const GOVERNED_VCF_PANEL_SPECIES_ID: &str = "Homo sapiens";
pub const GOVERNED_VCF_PANEL_BUILD_ID: &str = "GRCh38";
pub const GOVERNED_VCF_PANEL_ID: &str = "hsapiens_grch38_1kg_3202_chr22";
pub const GOVERNED_VCF_MAP_ID: &str = "hsapiens_grch38_glimpse_chr22";
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedVcfPanelAssetContract {
    pub schema_version: &'static str,
    pub fixture_class: &'static str,
    pub species_id: String,
    pub build_id: String,
    pub panel_id: String,
    pub map_id: String,
    pub panel_files: Vec<String>,
    pub panel_derivations: Vec<ResolvedVcfPanelDerivationContract>,
    pub map_files: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ResolvedVcfPanelDerivationContract {
    pub path: String,
    pub source_file_name: String,
    pub tool_id: String,
    pub tool_version: String,
    pub tool_artifact_sha256: String,
    pub argv: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct LocalVcfPanelFixtureReport {
    pub schema_version: &'static str,
    pub fixture_class: &'static str,
    pub panel_id: String,
    pub map_id: String,
    pub materialized_files: Vec<String>,
}

/// Resolve or materialize the governed local panel metadata fixture.
///
/// # Errors
/// Returns an error for invalid reference authority, incompatible assets, poisoned locks or failed IO.
pub fn resolve_governed_vcf_panel_asset_contract() -> Result<ResolvedVcfPanelAssetContract> {
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
    if panel.compatibility.minimac_reference_format == "msav" {
        validate_imputation_tool_compatibility("minimac4", &panel, &map)?;
    }

    let panel_derivations = panel_lock
        .files
        .iter()
        .filter_map(|file| {
            file.derivation.as_ref().map(|derivation| ResolvedVcfPanelDerivationContract {
                path: file.path.clone(),
                source_file_name: derivation.source_file_name.clone(),
                tool_id: derivation.tool_id.clone(),
                tool_version: derivation.tool_version.clone(),
                tool_artifact_sha256: derivation.tool_artifact_sha256.clone(),
                argv: derivation.argv.clone(),
            })
        })
        .collect::<Vec<_>>();

    Ok(ResolvedVcfPanelAssetContract {
        schema_version: "bijux.local_vcf_panel_asset_contract.v1",
        fixture_class: "local_smoke_metadata_only",
        species_id: panel.species_id,
        build_id: panel.build_id,
        panel_id: panel.id,
        map_id: map.id,
        panel_files: panel_lock.files.into_iter().map(|file| file.path).collect(),
        panel_derivations,
        map_files: map_lock.files.into_iter().map(|file| file.path).collect(),
    })
}

/// Resolve or materialize the governed local panel metadata fixture.
///
/// # Errors
/// Returns an error for invalid reference authority, incompatible assets, poisoned locks or failed IO.
pub fn materialize_local_vcf_panel_fixture(
    materialization_root: &Path,
) -> Result<LocalVcfPanelFixtureReport> {
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    let fixture_root = materialization_root.join("fixtures");
    let panel_root = fixture_root.join("panels").join(&contract.panel_id);
    let map_root = fixture_root.join("maps").join(&contract.map_id);
    let mut materialized_files = Vec::new();

    for relative in &contract.panel_files {
        let target = panel_root.join(relative);
        if let Some(derivation) =
            contract.panel_derivations.iter().find(|derivation| derivation.path == *relative)
        {
            write_local_smoke_derived_marker(&target, derivation)?;
        } else {
            write_local_smoke_marker(&target, "panel", relative)?;
        }
        materialized_files.push(target.display().to_string());
    }
    if !contract.panel_files.iter().any(|path| path.ends_with("panel.msav")) {
        let target = panel_root.join("panel.msav");
        let derivation = ResolvedVcfPanelDerivationContract {
            path: "panel.msav".to_string(),
            source_file_name: "panel_vcf".to_string(),
            tool_id: "minimac4".to_string(),
            tool_version: "4.1.6".to_string(),
            tool_artifact_sha256:
                "e627d6955c1c103caa0bd2ccc2ecf674cafc4a974694fc38b91c0a6bf000b478".to_string(),
            argv: vec![
                "minimac4".to_string(),
                "--compress-reference".to_string(),
                "panel.vcf.gz".to_string(),
                "--output".to_string(),
                "panel.msav".to_string(),
                "--threads".to_string(),
                "8".to_string(),
            ],
        };
        write_local_smoke_derived_marker(&target, &derivation)?;
        materialized_files.push(target.display().to_string());
    }
    for relative in &contract.map_files {
        let target = map_root.join(relative);
        write_local_smoke_marker(&target, "map", relative)?;
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
    bijux_dna_infra::ensure_dir(parent)?;
    bijux_dna_infra::atomic_write_bytes(
        target,
        format!("fixture_class=local_smoke\nasset_kind={asset_kind}\nrelative_path={relative}\n")
            .as_bytes(),
    )?;
    Ok(())
}

fn write_local_smoke_derived_marker(
    target: &Path,
    derivation: &ResolvedVcfPanelDerivationContract,
) -> Result<()> {
    let parent = target.parent().ok_or_else(|| {
        anyhow!("local VCF derived fixture path has no parent: {}", target.display())
    })?;
    bijux_dna_infra::ensure_dir(parent)?;
    bijux_dna_infra::atomic_write_bytes(
        target,
        format!(
            "fixture_class=local_smoke\nasset_kind=derived_panel\nderived_from={}\nderivation_tool={}\nderivation_tool_version={}\nderivation_tool_artifact_sha256={}\nderivation_argv={}\n",
            derivation.source_file_name,
            derivation.tool_id,
            derivation.tool_version,
            derivation.tool_artifact_sha256,
            derivation.argv.join(" "),
        ).as_bytes(),
    )?;
    Ok(())
}

/// Resolve or materialize the governed local panel metadata fixture.
///
/// # Errors
/// Returns an error for invalid reference authority, incompatible assets, poisoned locks or failed IO.
pub fn with_local_vcf_panel_fixture_lock<T>(action: impl FnOnce() -> Result<T>) -> Result<T> {
    static MATERIALIZATION_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    let lock = MATERIALIZATION_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock.lock().map_err(|_| anyhow!("local VCF panel fixture lock is poisoned"))?;
    action()
}
