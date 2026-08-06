//! Owner: bijux-dna-bench
//! Exact source and container authority loading for executable benchmark attempts.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_defined_interface_campaign_alignment;
use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkCampaignPlan, BenchmarkExecutionSpec,
    BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog,
};
use bijux_dna_containers::api::{
    load_container_catalog, load_image_inventory_publication, resolve_current_apptainer_image,
    validate_container_inventory_catalog, ContainerCatalog, ContainerImageInventoryPublication,
    CurrentApptainerImage,
};

use super::verify_campaign_plan_source;
use crate::workflow::{load_stage_campaign_registry, load_stage_interface_catalog_from_repo};

const CAMPAIGN_REGISTRY_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const CONTAINER_CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const STAGE_INTERFACE_REGISTRY_PATH: &str = "benchmarks/contracts/stage-interfaces.toml";

/// Validated, source-locked authorities needed to construct executable benchmark attempts.
pub struct BenchmarkExecutionAuthorities {
    repository_path: PathBuf,
    registry: BenchmarkStageCampaignRegistry,
    interfaces: BenchmarkStageInterfaceCatalog,
    catalog: ContainerCatalog,
    source_contract_sha256: BTreeMap<String, String>,
    image_inventory: ContainerImageInventoryPublication,
}

impl BenchmarkExecutionAuthorities {
    /// Return the exact source digests admitted when this authority bundle was loaded.
    #[must_use]
    pub fn source_contract_sha256(&self) -> &BTreeMap<String, String> {
        &self.source_contract_sha256
    }

    /// Resolve one planned tool to its exact current SIF and recheck the on-disk identity.
    ///
    /// # Errors
    /// Returns an error when the inventory does not select one current image or its STUDY file
    /// differs from the source-locked evidence.
    pub fn resolve_current_image(
        &self,
        study_root: &Path,
        tool_id: &str,
    ) -> Result<CurrentApptainerImage> {
        resolve_current_apptainer_image(&self.image_inventory, study_root, tool_id)
    }

    /// Build one execution specification without exposing mutable loaded authorities.
    ///
    /// # Errors
    /// Returns an error when material source locks differ from this bundle or any campaign,
    /// interface, catalog, plan, image, identity, or material contract is invalid.
    pub fn build_execution_spec(
        &self,
        plan: &BenchmarkCampaignPlan,
        identity: &BenchmarkAttemptIdentity,
        image: &CurrentApptainerImage,
        material: super::BenchmarkExecutionMaterial,
    ) -> Result<BenchmarkExecutionSpec> {
        if material.source_contract_sha256 != self.source_contract_sha256 {
            bail!("execution material source locks differ from loaded authorities");
        }
        let material_repository_path = material
            .repository_path
            .canonicalize()
            .context("canonicalize execution material repository path")?;
        if material_repository_path != self.repository_path {
            bail!("execution material repository path differs from loaded authorities");
        }
        super::build_execution_spec(
            &self.registry,
            &self.interfaces,
            &self.catalog,
            plan,
            identity,
            image,
            material,
        )
    }
}

/// Load and cross-validate every repository and image authority for one campaign plan.
///
/// # Errors
/// Returns an error when a source file is missing or malformed, source hashes drift, the plan and
/// inventory name another repository commit, or catalog and inventory closure differs.
pub fn load_execution_authorities(
    repository_root: &Path,
    inventory_path: &Path,
    repository_commit: &str,
    plan: &BenchmarkCampaignPlan,
) -> Result<BenchmarkExecutionAuthorities> {
    let repository_path = repository_root
        .canonicalize()
        .context("canonicalize benchmark execution repository root")?;
    let campaign_path = repository_path.join(CAMPAIGN_REGISTRY_PATH);
    let interface_path = repository_path.join(STAGE_INTERFACE_REGISTRY_PATH);
    let catalog_path = repository_path.join(CONTAINER_CATALOG_PATH);
    require_source_file(&campaign_path)?;
    require_source_file(&interface_path)?;
    require_source_file(&catalog_path)?;
    require_source_file(inventory_path)?;

    let registry = load_stage_campaign_registry(&campaign_path)?;
    let interfaces = load_stage_interface_catalog_from_repo(&interface_path, &repository_path)?;
    validate_defined_interface_campaign_alignment(&interfaces, &registry)
        .context("align stage interfaces with campaign declarations")?;
    let catalog = load_container_catalog(&repository_path)?;
    let image_inventory = load_image_inventory_publication(inventory_path)?;

    let campaign_registry_sha256 = source_digest(&campaign_path)?;
    let container_catalog_sha256 = source_digest(&catalog_path)?;
    let stage_interface_registry_sha256 = source_digest(&interface_path)?;
    let image_inventory_sha256 = source_digest(inventory_path)?;
    verify_campaign_plan_source(plan, repository_commit, &campaign_registry_sha256)?;
    if image_inventory.repository_commit != repository_commit {
        bail!("image inventory repository commit differs from the campaign source lock");
    }
    validate_container_inventory_catalog(&container_catalog_sha256, &catalog, &image_inventory)
        .context("validate execution image inventory against container catalog")?;

    let mut source_contract_sha256 = BTreeMap::from([
        ("campaign_registry".to_string(), campaign_registry_sha256),
        ("container_catalog".to_string(), container_catalog_sha256),
        ("image_inventory".to_string(), image_inventory_sha256),
        ("stage_interface_registry".to_string(), stage_interface_registry_sha256),
    ]);
    let stage_id = plan
        .cycle
        .attempts
        .first()
        .context("benchmark campaign plan has no attempts")?
        .identity
        .run
        .stage_id
        .as_str();
    let interface = interfaces
        .interfaces
        .iter()
        .find(|interface| interface.stage_id == stage_id)
        .context("campaign stage has no loaded interface authority")?;
    if let Some(relative) = &interface.planner_contract {
        let planner_contract = repository_path.join(relative);
        require_source_file(&planner_contract)?;
        source_contract_sha256
            .insert("planner_contract".to_string(), source_digest(&planner_contract)?);
    }

    Ok(BenchmarkExecutionAuthorities {
        repository_path,
        registry,
        interfaces,
        catalog,
        source_contract_sha256,
        image_inventory,
    })
}

fn require_source_file(path: &Path) -> Result<()> {
    let metadata = std::fs::symlink_metadata(path)
        .with_context(|| format!("inspect benchmark execution authority {}", path.display()))?;
    if metadata.file_type().is_symlink() || !metadata.is_file() {
        bail!("benchmark execution authority is not a regular non-symlink file");
    }
    Ok(())
}

fn source_digest(path: &Path) -> Result<String> {
    bijux_dna_infra::hash_file_sha256(path)
        .with_context(|| format!("hash benchmark execution authority {}", path.display()))
}
