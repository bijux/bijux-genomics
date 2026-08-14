//! Owner: bijux-dna-bench
//! Exact source and container authority loading for executable benchmark attempts.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bijux_dna_bench_model::contract::validate_defined_interface_campaign_alignment;
use bijux_dna_bench_model::{
    BenchmarkAttemptIdentity, BenchmarkCampaignPlan, BenchmarkCaseMatrixV2, BenchmarkExecutionSpec,
    BenchmarkStageCampaignRegistry, BenchmarkStageInterfaceCatalog,
};
use bijux_dna_containers::api::{
    load_container_catalog, load_image_inventory_publication,
    resolve_catalog_locked_apptainer_artifact, validate_container_inventory_catalog,
    ContainerCatalog, ContainerImageInventoryPublication, CurrentApptainerImage,
};
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_stage_contract::{
    canonical_tool_bindings, StagePlanV1, ToolBinding, ToolBindingRegistry,
};

use super::{verify_campaign_plan_source, verify_case_campaign_plan_source};
use crate::workflow::{
    benchmark_case_matrix_sha256, load_stage_campaign_registry,
    load_stage_interface_catalog_from_repo,
};

const CAMPAIGN_REGISTRY_PATH: &str = "benchmarks/contracts/stage-campaigns.toml";
const CONTAINER_CATALOG_PATH: &str = "containers/contracts/catalog.toml";
const STAGE_INTERFACE_REGISTRY_PATH: &str = "benchmarks/contracts/stage-interfaces.toml";

/// Validated, source-locked authorities needed to construct executable benchmark attempts.
pub struct BenchmarkExecutionAuthorities {
    repository_path: PathBuf,
    registry: BenchmarkStageCampaignRegistry,
    interfaces: BenchmarkStageInterfaceCatalog,
    catalog: ContainerCatalog,
    catalog_sha256: String,
    repository_commit: String,
    source_contract_sha256: BTreeMap<String, String>,
    image_inventory: ContainerImageInventoryPublication,
    bindings: ToolBindingRegistry,
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
        resolve_catalog_locked_apptainer_artifact(
            &self.image_inventory,
            &self.catalog_sha256,
            &self.catalog,
            &self.repository_commit,
            &self.repository_path,
            study_root,
            tool_id,
        )
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

    /// Construct every measured stage plan from the exact loaded binding and catalog authority.
    ///
    /// # Errors
    /// Returns an error when the attempt has no canonical binding or catalog tool, or unit material
    /// differs from the execution specification.
    pub fn build_stage_plans(
        &self,
        spec: &BenchmarkExecutionSpec,
        units: &[super::BenchmarkStagePlanUnitMaterial],
    ) -> Result<Vec<StagePlanV1>> {
        let binding = self
            .bindings
            .binding(
                &StageId::new(&spec.identity.run.stage_id),
                &ToolId::new(&spec.identity.tool_id),
            )
            .context("execution attempt has no canonical tool binding")?;
        let tool = self
            .catalog
            .tools
            .iter()
            .find(|tool| tool.id == spec.identity.tool_id)
            .context("execution attempt has no canonical catalog tool")?;
        super::build_benchmark_stage_plans(spec, binding, tool, units)
    }
}

/// Load and cross-validate every repository and image authority for one campaign plan.
///
/// # Errors
/// Returns an error when a source file is missing or malformed, campaign resources differ from
/// canonical bindings, source hashes drift, the plan and inventory name another repository commit,
/// or catalog and inventory closure differs.
pub fn load_execution_authorities(
    repository_root: &Path,
    inventory_path: &Path,
    repository_commit: &str,
    plan: &BenchmarkCampaignPlan,
) -> Result<BenchmarkExecutionAuthorities> {
    load_execution_authorities_inner(repository_root, inventory_path, repository_commit, plan, None)
}

/// Load source and image authorities for a case-scoped campaign plan.
///
/// # Errors
/// Returns an error when the matrix differs from the plan, checked-in campaign/interface/tool
/// authorities, or active lineage-tip case closure.
pub fn load_case_execution_authorities(
    repository_root: &Path,
    inventory_path: &Path,
    repository_commit: &str,
    plan: &BenchmarkCampaignPlan,
    matrix: &BenchmarkCaseMatrixV2,
) -> Result<BenchmarkExecutionAuthorities> {
    load_execution_authorities_inner(
        repository_root,
        inventory_path,
        repository_commit,
        plan,
        Some(matrix),
    )
}

fn load_execution_authorities_inner(
    repository_root: &Path,
    inventory_path: &Path,
    repository_commit: &str,
    plan: &BenchmarkCampaignPlan,
    matrix: Option<&BenchmarkCaseMatrixV2>,
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
    let bindings = canonical_tool_bindings().context("load canonical campaign tool bindings")?;
    validate_campaign_binding_resources(&registry, &bindings)?;
    let catalog = load_container_catalog(&repository_path)?;
    let image_inventory = load_image_inventory_publication(inventory_path)?;

    let campaign_registry_sha256 = source_digest(&campaign_path)?;
    let container_catalog_sha256 = source_digest(&catalog_path)?;
    let stage_interface_registry_sha256 = source_digest(&interface_path)?;
    let image_inventory_sha256 = source_digest(inventory_path)?;
    let binding_bytes = serde_json::to_vec(&bindings.bindings().cloned().collect::<Vec<_>>())?;
    let tool_binding_registry_sha256 = bijux_dna_infra::hash_bytes_sha256(&binding_bytes);
    if let Some(matrix) = matrix {
        verify_case_campaign_plan_source(plan, matrix)?;
        validate_case_source_authorities(
            matrix,
            plan,
            &registry,
            &interfaces,
            &bindings,
            repository_commit,
            &campaign_registry_sha256,
            &stage_interface_registry_sha256,
            &tool_binding_registry_sha256,
        )?;
    } else {
        verify_campaign_plan_source(plan, repository_commit, &campaign_registry_sha256)?;
    }
    if image_inventory.repository_commit != repository_commit {
        bail!("image inventory repository commit differs from the campaign source lock");
    }
    validate_container_inventory_catalog(&container_catalog_sha256, &catalog, &image_inventory)
        .context("validate execution image inventory against container catalog")?;

    let mut source_contract_sha256 = BTreeMap::from([
        ("campaign_registry".to_string(), campaign_registry_sha256),
        ("container_catalog".to_string(), container_catalog_sha256.clone()),
        ("image_inventory".to_string(), image_inventory_sha256),
        ("stage_interface_registry".to_string(), stage_interface_registry_sha256),
        ("tool_binding_registry".to_string(), tool_binding_registry_sha256),
    ]);
    if let Some(matrix) = matrix {
        source_contract_sha256
            .insert("case_matrix".to_string(), benchmark_case_matrix_sha256(matrix)?);
    }
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
        catalog_sha256: container_catalog_sha256,
        repository_commit: repository_commit.to_owned(),
        source_contract_sha256,
        image_inventory,
        bindings,
    })
}

#[allow(clippy::too_many_arguments)]
pub(super) fn validate_case_source_authorities(
    matrix: &BenchmarkCaseMatrixV2,
    plan: &BenchmarkCampaignPlan,
    registry: &BenchmarkStageCampaignRegistry,
    interfaces: &BenchmarkStageInterfaceCatalog,
    bindings: &ToolBindingRegistry,
    repository_commit: &str,
    campaign_registry_sha256: &str,
    interface_registry_sha256: &str,
    binding_registry_sha256: &str,
) -> Result<()> {
    if matrix.repository_commit != repository_commit
        || matrix.campaign_registry_sha256 != campaign_registry_sha256
        || matrix.interface_catalog_sha256 != interface_registry_sha256
        || matrix.tool_binding_registry_sha256 != binding_registry_sha256
    {
        bail!("benchmark case matrix differs from current execution source authorities");
    }
    for case in &matrix.cases {
        let campaign = registry
            .campaigns
            .iter()
            .find(|campaign| campaign.run_id == case.run_id)
            .context("benchmark case campaign is absent from current registry")?;
        let interface = interfaces
            .interfaces
            .iter()
            .find(|interface| interface.stage_id == case.stage_id)
            .context("benchmark case interface is absent from current registry")?;
        let binding = bindings
            .bindings()
            .find(|binding| binding.binding_id == case.binding.binding_id)
            .context("benchmark case binding is absent from current registry")?;
        if campaign != &case.campaign || interface != &case.interface || binding != &case.binding {
            bail!("benchmark case embeds noncanonical execution authorities");
        }
    }
    let expected = matrix
        .cases
        .iter()
        .map(|case| {
            (
                case.case_id.as_str(),
                case.run_id.as_str(),
                case.domain.as_str(),
                case.stage_id.as_str(),
                case.tool_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    let declared = plan
        .cycle
        .attempts
        .iter()
        .map(|attempt| {
            (
                attempt.identity.case_id.as_deref().unwrap_or_default(),
                attempt.identity.run.run_id.as_str(),
                attempt.identity.run.domain.as_str(),
                attempt.identity.run.stage_id.as_str(),
                attempt.identity.tool_id.as_str(),
            )
        })
        .collect::<BTreeSet<_>>();
    if !declared.is_subset(&expected) {
        bail!("benchmark campaign attempt history contains a case outside its matrix");
    }
    let referenced = plan
        .cycle
        .attempts
        .iter()
        .filter_map(|attempt| {
            attempt.supersedes_attempt_id.as_deref().map(|predecessor| {
                (
                    attempt.identity.case_id.as_deref().unwrap_or_default(),
                    attempt.identity.tool_id.as_str(),
                    predecessor,
                )
            })
        })
        .collect::<BTreeSet<_>>();
    let mut active = BTreeSet::new();
    for attempt in &plan.cycle.attempts {
        if referenced.contains(&(
            attempt.identity.case_id.as_deref().unwrap_or_default(),
            attempt.identity.tool_id.as_str(),
            attempt.identity.attempt_id.as_str(),
        )) {
            continue;
        }
        if !active.insert((
            attempt.identity.case_id.as_deref().unwrap_or_default(),
            attempt.identity.run.run_id.as_str(),
            attempt.identity.run.domain.as_str(),
            attempt.identity.run.stage_id.as_str(),
            attempt.identity.tool_id.as_str(),
        )) {
            bail!("benchmark campaign has multiple lineage tips for one matrix case");
        }
    }
    if active != expected {
        bail!("benchmark campaign active attempt closure differs from its case matrix");
    }
    Ok(())
}

fn validate_campaign_binding_resources(
    registry: &BenchmarkStageCampaignRegistry,
    bindings: &ToolBindingRegistry,
) -> Result<()> {
    for campaign in &registry.campaigns {
        let stage_id = StageId::new(&campaign.stage_id);
        for tool_id in &campaign.tool_ids {
            let binding =
                bindings.binding(&stage_id, &ToolId::new(tool_id)).with_context(|| {
                    format!(
                        "campaign {} has no canonical binding for {}/{}",
                        campaign.run_id, campaign.stage_id, tool_id
                    )
                })?;
            let allocated = campaign
                .threads_by_tool
                .get(tool_id)
                .copied()
                .or(campaign.threads)
                .with_context(|| {
                    format!("campaign {} does not allocate threads for {tool_id}", campaign.run_id)
                })?;
            if binding.resources().threads != allocated {
                bail!(
                    "campaign {} allocates {allocated} threads for {}/{} but its canonical binding requires {}",
                    campaign.run_id,
                    campaign.stage_id,
                    tool_id,
                    binding.resources().threads
                );
            }
        }
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use anyhow::Context;
    use bijux_dna_stage_contract::canonical_tool_bindings;

    use super::{validate_campaign_binding_resources, CAMPAIGN_REGISTRY_PATH};
    use crate::workflow::load_stage_campaign_registry;

    #[test]
    fn checked_in_campaign_resources_close_canonical_bindings() -> anyhow::Result<()> {
        let repository_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?;
        let mut registry =
            load_stage_campaign_registry(&repository_root.join(CAMPAIGN_REGISTRY_PATH))?;
        let bindings = canonical_tool_bindings()?;
        validate_campaign_binding_resources(&registry, &bindings)?;

        let campaign = registry
            .campaigns
            .iter_mut()
            .find(|campaign| campaign.run_id == "run-2042")
            .context("checked-in campaign registry omits run-2042")?;
        campaign.threads_by_tool.insert("seqkit".to_owned(), 3);
        let error = validate_campaign_binding_resources(&registry, &bindings)
            .err()
            .context("campaign/binding resource drift was accepted")?;
        assert!(error.to_string().contains("run-2042 allocates 3 threads"));
        Ok(())
    }
}
