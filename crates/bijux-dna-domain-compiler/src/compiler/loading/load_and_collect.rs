use super::super::*;
use super::index_defaults::validate_index_defaults;
use super::stage_loading::load_domain_stages;
use super::tool_loading::load_domain_tools;

#[allow(clippy::too_many_lines)]
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
    let mut tools: ToolMap = BTreeMap::new();
    let mut stage_to_tools: StageToolMap = BTreeMap::new();
    let mut stage_planned: StagePlannedMap = BTreeMap::new();
    let mut stage_defaults: StageDefaultMap = BTreeMap::new();
    let mut stage_default_rationale: StageDefaultRationaleMap = BTreeMap::new();
    let mut stage_statuses: StageStatusMap = BTreeMap::new();
    let mut stage_output_kinds: StageOutputKindsMap = BTreeMap::new();
    for domain in ["fastq", "bam"] {
        let index_path = domain_dir.join(domain).join("index.yaml");
        let index: DomainIndex = read_yaml(&index_path)?;
        if index.domain != domain {
            return Err(anyhow!(
                "{} has domain {} but expected {}",
                index_path.display(),
                index.domain,
                domain
            ));
        }
        load_domain_tools(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut tools,
            &mut stage_to_tools,
        )?;
        load_domain_stages(
            domain_dir,
            domain,
            &index,
            active_scope,
            &mut stage_to_tools,
            &mut stage_planned,
            &mut stage_statuses,
            &mut stage_output_kinds,
        )?;
        for (stage_id, tool_ids) in &index.stage_tool_compatibility {
            if !stage_to_tools.contains_key(stage_id) {
                continue;
            }
            let active_tools = stage_to_tools.entry(stage_id.clone()).or_default();
            active_tools.clear();
            for tool_id in tool_ids {
                if tools.contains_key(tool_id) {
                    active_tools.insert(tool_id.clone());
                }
            }
        }
        validate_index_defaults(
            &index,
            &stage_to_tools,
            &mut stage_defaults,
            &mut stage_default_rationale,
        )?;
        if index.pipeline_compositions.is_empty() {
            return Err(anyhow!("index missing pipeline_compositions"));
        }
        if !index.pipeline_compositions.contains_key("pre_hpc_best") {
            return Err(anyhow!(
                "index pipeline_compositions must include pre_hpc_best"
            ));
        }
        for (pipeline_name, stages) in &index.pipeline_compositions {
            if stages.is_empty() {
                return Err(anyhow!(
                    "index pipeline {pipeline_name} has empty stage list"
                ));
            }
            for s in stages {
                if !index.stage_ids.contains(s) {
                    return Err(anyhow!(
                        "index pipeline {pipeline_name} references unknown stage {s}"
                    ));
                }
            }
        }
        if index.benchmark_scenarios.is_empty() {
            return Err(anyhow!("index missing benchmark_scenarios"));
        }
        for (scenario_id, scenario) in &index.benchmark_scenarios {
            if scenario.stage_id.trim().is_empty()
                || scenario.description.trim().is_empty()
                || scenario.fairness_rules.is_empty()
            {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} missing stage/description/fairness_rules"
                ));
            }
            if !index.stage_ids.contains(&scenario.stage_id) {
                return Err(anyhow!(
                    "index benchmark scenario {scenario_id} references unknown stage {}",
                    scenario.stage_id
                ));
            }
        }
    }
    for tool in tools.values() {
        for stage in &tool.stage_ids {
            if !stage_to_tools.contains_key(stage) {
                return Err(anyhow!(
                    "tool {} references unknown stage {}",
                    tool.id,
                    stage
                ));
            }
        }
    }
    Ok((
        tools,
        stage_to_tools,
        stage_planned,
        stage_defaults,
        stage_default_rationale,
        stage_statuses,
        stage_output_kinds,
    ))
}
