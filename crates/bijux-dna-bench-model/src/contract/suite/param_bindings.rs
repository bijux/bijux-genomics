//! Owner: bijux-dna-bench-model
//! Structured parameter binding validation for benchmark suites.

use crate::diagnostics::BenchError;
use crate::model::BenchmarkStageSpec;
use bijux_dna_domain_fastq::stage_parameter_ids;

pub(crate) fn validate_stage_param_bindings(stage: &BenchmarkStageSpec) -> Result<(), BenchError> {
    if !stage.params.is_empty() && !stage.param_bindings.is_empty() {
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {} must use either params or param_bindings, not both",
            stage.stage
        )));
    }
    for binding in &stage.param_bindings {
        let binding_targets = stage
            .graph_nodes()
            .into_iter()
            .map(|node| (node.node_id.clone(), node))
            .collect::<std::collections::BTreeMap<_, _>>();
        if let Some(stage_instance_id) = binding.stage_instance_id.as_ref() {
            if stage_instance_id.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank stage_instance_id in param_bindings",
                    stage.stage
                )));
            }
            if !binding_targets.contains_key(stage_instance_id) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} param binding target {} must match the stage node or one of its stage-tool nodes",
                    stage.stage, stage_instance_id
                )));
            }
        }
        if let Some(tool) = binding.tool.as_ref() {
            if tool.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank tool ids in param_bindings",
                    stage.stage
                )));
            }
            if !stage.tools.iter().any(|candidate| candidate == tool) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} param binding tool {} must belong to declared tools",
                    stage.stage, tool
                )));
            }
            if let Some(target_node_id) = binding.stage_instance_id.as_ref() {
                let target_node = binding_targets.get(target_node_id).ok_or_else(|| {
                    BenchError::InvalidPolicy(format!(
                        "suite stage {} param binding target {} must resolve to a declared graph node",
                        stage.stage, target_node_id
                    ))
                })?;
                match target_node.tool_id.as_deref() {
                    Some(target_tool) if target_tool == tool => {}
                    Some(target_tool) => {
                        return Err(BenchError::InvalidPolicy(format!(
                            "suite stage {} param binding tool {} must match target tool node {}",
                            stage.stage, tool, target_tool
                        )));
                    }
                    None => {
                        return Err(BenchError::InvalidPolicy(format!(
                            "suite stage {} tool-scoped param binding for {} must target its stage-tool node, not the stage node {}",
                            stage.stage, tool, target_node_id
                        )));
                    }
                }
            }
        }
        if binding.values.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} param_bindings must include at least one structured value",
                stage.stage
            )));
        }
        for key in binding.values.keys() {
            if key.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} param_bindings must not include blank parameter names",
                    stage.stage
                )));
            }
        }
        if binding.tool.is_none() {
            validate_stage_param_binding_keys(&stage.stage, binding.values.keys())?;
        }
    }
    Ok(())
}

fn validate_stage_param_binding_keys<'a>(
    stage_id: &str,
    keys: impl IntoIterator<Item = &'a String>,
) -> Result<(), BenchError> {
    let Some(parameter_ids) = stage_parameter_ids(stage_id) else {
        return Ok(());
    };
    for key in keys {
        if parameter_ids.contains(key) {
            continue;
        }
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {stage_id} stage-scoped param binding {key} is not declared in the governed stage parameter contract"
        )));
    }
    Ok(())
}
