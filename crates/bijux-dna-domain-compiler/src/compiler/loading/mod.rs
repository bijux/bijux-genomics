pub(super) mod artifact_vocabulary;
mod container_catalog;
mod image_registries;
mod index_catalogs;
mod index_defaults;
mod load_and_collect;
mod stage_loading;
mod stage_registries;
mod tool_loading;
mod tool_registries;

use super::{
    BTreeMap, BTreeSet, ContainerToolContract, Path, Result, StageDefaultMap,
    StageDefaultRationaleMap, StageOutputKindsMap, StagePlannedMap, StageStatusMap, StageToolMap,
    ToolMap,
};

pub(super) use container_catalog::load_container_catalog;

pub(super) struct ToolRegistryOutputs {
    pub(super) production_registry: String,
    pub(super) experimental_registry: String,
    pub(super) container_experimental_registry: String,
    pub(super) required_tools: String,
    pub(super) production_tool_ids: BTreeSet<String>,
}

pub(super) struct ToolRegistryInputs<'a> {
    pub(super) tools: &'a ToolMap,
    pub(super) stage_to_tools: &'a StageToolMap,
    pub(super) stage_planned: &'a StagePlannedMap,
    pub(super) stage_defaults: &'a StageDefaultMap,
    pub(super) stage_default_rationale: &'a StageDefaultRationaleMap,
    pub(super) container_catalog: &'a BTreeMap<String, ContainerToolContract>,
    pub(super) vcf_tool_ids: &'a BTreeSet<String>,
    pub(super) source_commit: &'a str,
}

pub(super) fn collect_domain_data(
    domain_dir: &Path,
    active_scope: &str,
) -> Result<(
    ToolMap,
    StageToolMap,
    StagePlannedMap,
    StageDefaultMap,
    StageDefaultRationaleMap,
    StageStatusMap,
    StageOutputKindsMap,
)> {
    load_and_collect::collect_domain_data(domain_dir, active_scope)
}

pub(super) fn build_tool_registries_toml(
    inputs: &ToolRegistryInputs<'_>,
) -> Result<ToolRegistryOutputs> {
    let outputs = tool_registries::build_tool_registries_toml(inputs)?;
    Ok(ToolRegistryOutputs {
        production_registry: outputs.production_registry,
        experimental_registry: outputs.experimental_registry,
        container_experimental_registry: outputs.container_experimental_registry,
        required_tools: outputs.required_tools,
        production_tool_ids: outputs.production_tool_ids,
    })
}

pub(super) fn collect_vcf_image_versions(
    domain_dir: &Path,
) -> Result<BTreeMap<String, (String, bool)>> {
    image_registries::collect_vcf_image_versions(domain_dir)
}

pub(super) fn build_images_toml(
    tools: &ToolMap,
    vcf_image_versions: &BTreeMap<String, (String, bool)>,
    container_catalog: &BTreeMap<String, ContainerToolContract>,
    source_commit: &str,
) -> String {
    image_registries::build_images_toml(tools, vcf_image_versions, container_catalog, source_commit)
}

pub(super) fn build_stages_toml(
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_statuses: &StageStatusMap,
    stage_output_kinds: &StageOutputKindsMap,
    production_tool_ids: &BTreeSet<String>,
    domain_dir: &Path,
    source_commit: &str,
) -> String {
    stage_registries::build_stages_toml(
        tools,
        stage_to_tools,
        stage_statuses,
        stage_output_kinds,
        production_tool_ids,
        domain_dir,
        source_commit,
    )
}
