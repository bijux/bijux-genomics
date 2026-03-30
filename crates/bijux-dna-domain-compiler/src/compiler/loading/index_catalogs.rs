use super::super::{anyhow, DomainIndex, Result};

pub(super) fn validate_pipeline_compositions(index: &DomainIndex) -> Result<()> {
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
        for stage_id in stages {
            if !index.stage_ids.contains(stage_id) {
                return Err(anyhow!(
                    "index pipeline {pipeline_name} references unknown stage {stage_id}"
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn validate_benchmark_scenarios(index: &DomainIndex) -> Result<()> {
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
    Ok(())
}
