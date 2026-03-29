mod load_and_collect;
mod registry_emitters;
mod stage_loading;
mod tool_loading;
mod tool_registries;

use super::*;

pub(super) struct ToolRegistryOutputs {
    pub(super) production_registry: String,
    pub(super) experimental_registry: String,
    pub(super) required_tools: String,
    pub(super) production_tool_ids: BTreeSet<String>,
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
    tools: &ToolMap,
    stage_to_tools: &StageToolMap,
    stage_planned: &StagePlannedMap,
    stage_defaults: &StageDefaultMap,
    stage_default_rationale: &StageDefaultRationaleMap,
    source_commit: &str,
) -> ToolRegistryOutputs {
    let outputs = tool_registries::build_tool_registries_toml(
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        source_commit,
    );
    ToolRegistryOutputs {
        production_registry: outputs.production_registry,
        experimental_registry: outputs.experimental_registry,
        required_tools: outputs.required_tools,
        production_tool_ids: outputs.production_tool_ids,
    }
}

pub(super) fn collect_vcf_image_versions(domain_dir: &Path) -> Result<BTreeMap<String, String>> {
    registry_emitters::collect_vcf_image_versions(domain_dir)
}

pub(super) fn build_images_toml(
    tools: &ToolMap,
    vcf_image_versions: &BTreeMap<String, String>,
    source_commit: &str,
) -> String {
    registry_emitters::build_images_toml(tools, vcf_image_versions, source_commit)
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
    registry_emitters::build_stages_toml(
        tools,
        stage_to_tools,
        stage_statuses,
        stage_output_kinds,
        production_tool_ids,
        domain_dir,
        source_commit,
    )
}
