//! Owner: bijux-dna-bench
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

use crate::error::BenchError;
use crate::model::{
    BenchmarkGraphNode, BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary,
};
use crate::policy::GateDecision;
use bijux_dna_core::ids::{StageId, ToolId};
use bijux_dna_domain_fastq::{
    admitted_execution_tools_for_stage, contract_for_stage, execution_support_for_stage,
    stage_input_ids, stage_output_ids, stage_parameter_ids, stage_tool_binding,
};
use bijux_dna_stage_contract::executor_registry::has_executor;

mod schemas;
pub use schemas::{DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1};

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.schema_version != SUITE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "suite schema mismatch: {}",
            suite.schema_version
        )));
    }
    if suite.datasets.is_empty() || suite.stages.is_empty() {
        return Err(BenchError::InvalidPolicy(
            "suite must include datasets and stages".to_string(),
        ));
    }
    if suite
        .datasets
        .iter()
        .any(|dataset| dataset.hash.trim().is_empty())
    {
        return Err(BenchError::InvalidPolicy(
            "suite datasets must include hash".to_string(),
        ));
    }
    if suite.datasets.len() < suite.diversity.min_dataset_count {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} datasets",
            suite.diversity.min_dataset_count
        )));
    }
    let mut seen_stage_nodes = std::collections::BTreeSet::new();
    let mut seen_stage_tool_nodes = std::collections::BTreeSet::new();
    let mut declared_stage_nodes = Vec::new();
    for stage in &suite.stages {
        if stage.stage.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(
                "suite stages must include non-empty stage ids".to_string(),
            ));
        }
        validate_stage_id(&stage.stage)?;
        let node_id = stage
            .stage_instance_id
            .as_deref()
            .unwrap_or(stage.stage.as_str());
        if node_id.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must not include blank stage_instance_id",
                stage.stage
            )));
        }
        if !seen_stage_nodes.insert(node_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite must not repeat stage node {}",
                node_id
            )));
        }
        declared_stage_nodes.push(node_id.to_string());
        if stage.tools.is_empty() && !planner_owned_graph_stage(&stage.stage) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must include at least one tool",
                stage.stage
            )));
        }
        if planner_owned_graph_stage(&stage.stage) && !stage.tools.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite planner-owned stage {} must not declare tool bindings",
                stage.stage
            )));
        }
        if !stage.params.is_empty() && !stage.param_bindings.is_empty() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} must use either params or param_bindings, not both",
                stage.stage
            )));
        }
        let mut seen_tools = std::collections::BTreeSet::new();
        for tool in &stage.tools {
            if tool.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank tool ids",
                    stage.stage
                )));
            }
            if !seen_tools.insert(tool.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not repeat tool {}",
                    stage.stage, tool
                )));
            }
            let tool_node_id = stage.tool_node_id(tool);
            if !seen_stage_tool_nodes.insert(tool_node_id.clone()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite must not repeat stage-tool node {}",
                    tool_node_id
                )));
            }
        }
        validate_stage_tools(&stage.stage, &stage.tools)?;
        let mut seen_params = std::collections::BTreeSet::new();
        for params in &stage.params {
            if params.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank params entries",
                    stage.stage
                )));
            }
            if !seen_params.insert(params.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not repeat params entry {}",
                    stage.stage, params
                )));
            }
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
                    let target_node = binding_targets.get(target_node_id).expect(
                        "validated stage param binding target must resolve to a declared graph node",
                    );
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
        for upstream in &stage.upstream_stage_instance_ids {
            if upstream.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank upstream_stage_instance_ids",
                    stage.stage
                )));
            }
            if upstream == node_id {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not reference itself as an upstream stage",
                    node_id
                )));
            }
        }
    }
    let declared_stage_nodes = declared_stage_nodes
        .into_iter()
        .collect::<std::collections::BTreeSet<_>>();
    let declared_graph_nodes = declared_graph_nodes(suite);
    for stage in &suite.stages {
        let node_id = stage
            .stage_instance_id
            .as_deref()
            .unwrap_or(stage.stage.as_str());
        for upstream in &stage.upstream_stage_instance_ids {
            if !declared_stage_nodes.contains(upstream) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} references unknown upstream stage node {}",
                    node_id, upstream
                )));
            }
        }
    }
    let mut seen_edges = std::collections::BTreeSet::new();
    for edge in &suite.edges {
        if edge.from.trim().is_empty() || edge.to.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(
                "suite edges must include non-empty from/to nodes".to_string(),
            ));
        }
        if edge.from == edge.to {
            return Err(BenchError::InvalidPolicy(format!(
                "suite edge {} -> {} must not reference itself",
                edge.from, edge.to
            )));
        }
        if !declared_graph_nodes.contains_key(&edge.from) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite edge references unknown source node {}",
                edge.from
            )));
        }
        if !declared_graph_nodes.contains_key(&edge.to) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite edge references unknown target node {}",
                edge.to
            )));
        }
        if !seen_edges.insert((
            edge.from.as_str(),
            edge.to.as_str(),
            edge.from_output_id.as_deref(),
            edge.to_input_id.as_deref(),
        )) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite must not repeat edge {} -> {} with identical artifact bindings",
                edge.from, edge.to
            )));
        }
        if let Some(from_output_id) = edge.from_output_id.as_ref() {
            if from_output_id.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite edge {} -> {} must not include blank from_output_id",
                    edge.from, edge.to
                )));
            }
        }
        if let Some(to_input_id) = edge.to_input_id.as_ref() {
            if to_input_id.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite edge {} -> {} must not include blank to_input_id",
                    edge.from, edge.to
                )));
            }
        }
        validate_edge_ports(edge, &declared_graph_nodes)?;
    }
    let mut classes = std::collections::BTreeSet::new();
    let mut layouts = std::collections::BTreeSet::new();
    for dataset in &suite.datasets {
        classes.insert(dataset.class_label.as_str());
        layouts.insert(dataset.read_layout.as_str());
    }
    if classes.len() < suite.diversity.min_classes {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} dataset classes",
            suite.diversity.min_classes
        )));
    }
    if layouts.len() < suite.diversity.min_read_layouts {
        return Err(BenchError::InvalidPolicy(format!(
            "suite must include at least {} read layouts",
            suite.diversity.min_read_layouts
        )));
    }
    for requirement in &suite.stratifications {
        let values: std::collections::BTreeSet<&str> = match requirement.key.as_str() {
            "dataset_class" => suite
                .datasets
                .iter()
                .map(|dataset| dataset.class_label.as_str())
                .collect(),
            "read_layout" => suite
                .datasets
                .iter()
                .map(|dataset| dataset.read_layout.as_str())
                .collect(),
            _ => {
                return Err(BenchError::InvalidPolicy(format!(
                    "unsupported stratification key {}",
                    requirement.key
                )))
            }
        };
        for required in &requirement.required_values {
            if !values.contains(required.as_str()) {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite missing required stratification value {} for {}",
                    required, requirement.key
                )));
            }
        }
    }
    if suite.analysis_requirements.require_bootstrap
        && suite.replicate_policy.count < suite.analysis_requirements.min_replicates_for_bootstrap
    {
        return Err(BenchError::InvalidPolicy(format!(
            "suite requires bootstrap with at least {} replicates",
            suite.analysis_requirements.min_replicates_for_bootstrap
        )));
    }
    if suite.analysis_requirements.require_outlier_detection && suite.replicate_policy.count < 3 {
        return Err(BenchError::InvalidPolicy(
            "suite requires outlier detection with at least 3 replicates".to_string(),
        ));
    }
    validate_suite_dag(suite)?;
    Ok(())
}

fn declared_graph_nodes(
    suite: &BenchmarkSuiteSpec,
) -> std::collections::BTreeMap<String, BenchmarkGraphNode> {
    suite.graph_nodes()
        .into_iter()
        .map(|node| (node.node_id.clone(), node))
        .collect()
}

fn validate_edge_ports(
    edge: &crate::model::BenchmarkStageEdge,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    match (&edge.from_output_id, &edge.to_input_id) {
        (Some(from_output_id), Some(to_input_id)) => {
            validate_stage_output_port(&edge.from, from_output_id, declared_graph_nodes)?;
            validate_stage_input_port(&edge.to, to_input_id, declared_graph_nodes)?;
            Ok(())
        }
        (None, None) => Ok(()),
        _ => Err(BenchError::InvalidPolicy(format!(
            "suite edge {} -> {} must set from_output_id and to_input_id together",
            edge.from, edge.to
        ))),
    }
}

fn validate_stage_output_port(
    node_id: &str,
    output_id: &str,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    let Some(node) = declared_graph_nodes.get(node_id) else {
        return Ok(());
    };
    let Some(output_ids) = stage_output_ids(&node.stage_id) else {
        return Ok(());
    };
    if output_ids.contains(output_id) {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite edge source node {} does not expose output {} in the governed {} contract",
        node_id, output_id, node.stage_id
    )))
}

fn validate_stage_input_port(
    node_id: &str,
    input_id: &str,
    declared_graph_nodes: &std::collections::BTreeMap<String, BenchmarkGraphNode>,
) -> Result<(), BenchError> {
    let Some(node) = declared_graph_nodes.get(node_id) else {
        return Ok(());
    };
    let Some(input_ids) = stage_input_ids(&node.stage_id) else {
        return Ok(());
    };
    if input_ids.contains(input_id) {
        return Ok(());
    }
    Err(BenchError::InvalidPolicy(format!(
        "suite edge target node {} does not accept input {} in the governed {} contract",
        node_id, input_id, node.stage_id
    )))
}

fn validate_stage_id(stage_id: &str) -> Result<(), BenchError> {
    if planner_owned_graph_stage(stage_id) {
        return Ok(());
    }
    if stage_id.starts_with("fastq.") {
        if contract_for_stage(stage_id).is_none() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} is not declared in the FASTQ domain catalog",
                stage_id
            )));
        }
        let stage_id = StageId::new(stage_id.to_string());
        let support = execution_support_for_stage(&stage_id).ok_or_else(|| {
            BenchError::InvalidPolicy(format!(
                "suite stage {} is missing FASTQ execution support",
                stage_id
            ))
        })?;
        if !support.is_plannable() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} is not plannable under FASTQ execution support",
                stage_id
            )));
        }
        return Ok(());
    }
    if !has_executor(stage_id) {
        return Err(BenchError::InvalidPolicy(format!(
            "suite stage {} is not registered in the stage executor catalog",
            stage_id
        )));
    }
    Ok(())
}

fn validate_stage_tools(stage_id: &str, tools: &[String]) -> Result<(), BenchError> {
    if planner_owned_graph_stage(stage_id) {
        if tools.is_empty() {
            return Ok(());
        }
        return Err(BenchError::InvalidPolicy(format!(
            "suite planner-owned stage {} must not declare tool bindings",
            stage_id
        )));
    }
    if !stage_id.starts_with("fastq.") {
        return Ok(());
    }
    let stage_id = StageId::new(stage_id.to_string());
    let admitted_tools = admitted_execution_tools_for_stage(&stage_id);
    for tool in tools {
        let tool_id = ToolId::new(tool.clone());
        if stage_tool_binding(&stage_id, &tool_id).is_none() {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} tool {} is not declared in the FASTQ stage-tool matrix",
                stage_id, tool
            )));
        }
        if !admitted_tools.iter().any(|admitted| admitted == &tool_id) {
            return Err(BenchError::InvalidPolicy(format!(
                "suite stage {} tool {} is not admitted by FASTQ execution support",
                stage_id, tool
            )));
        }
    }
    Ok(())
}

fn planner_owned_graph_stage(stage_id: &str) -> bool {
    matches!(
        stage_id,
        "benchmark.compare_stage_tools" | "benchmark.select_stage_tool"
    )
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
            "suite stage {} stage-scoped param binding {} is not declared in the governed stage parameter contract",
            stage_id, key
        )));
    }
    Ok(())
}

fn validate_suite_dag(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    let mut incoming = std::collections::BTreeMap::new();
    let mut outgoing = std::collections::BTreeMap::<String, Vec<String>>::new();
    for node_id in declared_graph_nodes(suite).into_keys() {
        incoming.entry(node_id).or_insert(0usize);
    }
    for stage in &suite.stages {
        let node_id = stage
            .stage_instance_id
            .as_deref()
            .unwrap_or(stage.stage.as_str())
            .to_string();
        for upstream in &stage.upstream_stage_instance_ids {
            outgoing
                .entry(upstream.clone())
                .or_default()
                .push(node_id.clone());
            *incoming.entry(node_id.clone()).or_insert(0) += 1;
        }
    }
    for edge in &suite.edges {
        outgoing
            .entry(edge.from.clone())
            .or_default()
            .push(edge.to.clone());
        *incoming.entry(edge.to.clone()).or_insert(0) += 1;
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(node, count)| {
            if *count == 0 {
                Some(node.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let mut visited = 0usize;
    while let Some(node) = ready.pop() {
        visited += 1;
        if let Some(children) = outgoing.get(&node) {
            for child in children {
                if let Some(count) = incoming.get_mut(child) {
                    *count -= 1;
                    if *count == 0 {
                        ready.push(child.clone());
                    }
                }
            }
        }
    }
    if visited != incoming.len() {
        return Err(BenchError::InvalidPolicy(
            "suite graph must be acyclic".to_string(),
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_suite;
    use crate::{
        AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageEdge, BenchmarkStageSpec,
        BenchmarkSuiteSpec, DatasetSpec, DiversityRequirements, ReplicatePolicy,
        StratificationRequirement,
    };

    fn suite_with_stage(stage: BenchmarkStageSpec) -> BenchmarkSuiteSpec {
        BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![stage],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        )
    }

    #[test]
    fn suite_validation_rejects_duplicate_stage_tools() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string(), "fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("duplicate tools must fail");
        assert!(error.to_string().contains("must not repeat tool"));
    }

    #[test]
    fn suite_validation_rejects_unknown_fastq_stage_ids() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.unknown_stage".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("unknown fastq stage must fail");
        assert!(error
            .to_string()
            .contains("is not declared in the FASTQ domain catalog"));
    }

    #[test]
    fn suite_validation_rejects_declared_only_fastq_stages() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.infer_asvs".to_string(),
            stage_instance_id: None,
            tools: vec!["dada2".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("declared-only fastq stage must fail");
        assert!(error.to_string().contains("is not plannable"));
    }

    #[test]
    fn suite_validation_rejects_fastq_tools_outside_execution_support() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["seqpurge".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error =
            validate_suite(&suite).expect_err("planned-only fastq tool should not validate");
        assert!(error
            .to_string()
            .contains("is not admitted by FASTQ execution support"));
    }

    #[test]
    fn suite_validation_accepts_registered_bam_stage_ids() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "bam.align".to_string(),
            stage_instance_id: None,
            tools: vec!["bwa".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        validate_suite(&suite).expect("registered BAM stages should validate");
    }

    #[test]
    fn suite_validation_accepts_planner_owned_select_nodes_without_tools() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "benchmark.select_stage_tool".to_string(),
            stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
            tools: Vec::new(),
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        validate_suite(&suite).expect("planner-owned select nodes should validate without tools");
    }

    #[test]
    fn suite_validation_rejects_tools_on_planner_owned_select_nodes() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "benchmark.select_stage_tool".to_string(),
            stage_instance_id: Some("benchmark.select_stage_tool.trim_reads".to_string()),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error =
            validate_suite(&suite).expect_err("planner-owned select nodes must not declare tools");
        assert!(error
            .to_string()
            .contains("must not declare tool bindings"));
    }

    #[test]
    fn suite_validation_rejects_blank_stage_params() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: vec!["".to_string()],
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("blank params must fail");
        assert!(error.to_string().contains("blank params entries"));
    }

    #[test]
    fn suite_validation_rejects_mixed_legacy_and_structured_stage_params() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: vec!["threads=4".to_string()],
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.tool.fastp".to_string()),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("mixed params must fail");
        assert!(error
            .to_string()
            .contains("either params or param_bindings"));
    }

    #[test]
    fn suite_validation_accepts_structured_stage_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.tool.fastp".to_string()),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        validate_suite(&suite).expect("structured param bindings should validate");
    }

    #[test]
    fn suite_validation_rejects_param_binding_targets_outside_stage_tool_nodes() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.detect_adapters.tool.fastqc".to_string()),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite).expect_err("foreign param binding targets must fail");
        assert!(error
            .to_string()
            .contains("must match the stage node or one of its stage-tool nodes"));
    }

    #[test]
    fn suite_validation_rejects_tool_scoped_binding_targeting_stage_node() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite)
            .expect_err("tool-scoped bindings must target a typed stage-tool node");
        assert!(error
            .to_string()
            .contains("must target its stage-tool node"));
    }

    #[test]
    fn suite_validation_rejects_tool_scoped_binding_with_mismatched_tool_node() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.cleanup".to_string()),
            tools: vec!["fastp".to_string(), "bbduk".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.cleanup.tool.bbduk".to_string()),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = validate_suite(&suite)
            .expect_err("tool-scoped bindings must match the targeted stage-tool node");
        assert!(error.to_string().contains("must match target tool node"));
    }

    #[test]
    fn suite_validation_accepts_governed_stage_scoped_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.tool_cohort".to_string()),
            tools: vec!["fastp".to_string(), "bbduk".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.tool_cohort".to_string()),
                tool: None,
                values: std::collections::BTreeMap::from([
                    ("min_length".to_string(), serde_json::json!(30)),
                    ("quality_cutoff".to_string(), serde_json::json!(20)),
                    ("adapter_policy".to_string(), serde_json::json!("auto")),
                ]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        validate_suite(&suite).expect("governed stage-scoped param bindings should validate");
    }

    #[test]
    fn suite_validation_rejects_unknown_stage_scoped_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.tool_cohort".to_string()),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some("fastq.trim_reads.tool_cohort".to_string()),
                tool: None,
                values: std::collections::BTreeMap::from([(
                    "adapter_bank_path".to_string(),
                    serde_json::json!("bank.json"),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error =
            validate_suite(&suite).expect_err("unknown stage-scoped param bindings must fail");
        assert!(error.to_string().contains("governed stage parameter contract"));
    }

    #[test]
    fn suite_validation_allows_repeated_stage_ids_with_distinct_stage_instance_ids() {
        let suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                    tools: vec!["fastp".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.cutadapt".to_string()),
                    tools: vec!["cutadapt".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: vec!["fastq.trim_reads.fastp".to_string()],
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        validate_suite(&suite).expect("distinct stage instance ids should validate");
    }

    #[test]
    fn suite_validation_rejects_unknown_upstream_stage_nodes() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: vec!["missing.stage.node".to_string()],
        });
        let error = validate_suite(&suite).expect_err("unknown upstream stage must fail");
        assert!(error.to_string().contains("unknown upstream stage node"));
    }

    #[test]
    fn suite_validation_rejects_unknown_explicit_edge_nodes() {
        let mut suite = suite_with_stage(BenchmarkStageSpec {
            stage: "fastq.trim_reads".to_string(),
            stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        suite.edges = vec![BenchmarkStageEdge {
            from: "fastq.trim_reads.fastp".to_string(),
            to: "missing.node".to_string(),
            from_output_id: None,
            to_input_id: None,
        }];
        let error = validate_suite(&suite).expect_err("unknown explicit edge target must fail");
        assert!(error.to_string().contains("unknown target node"));
    }

    #[test]
    fn suite_validation_rejects_duplicate_explicit_edges() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                    tools: vec!["fastp".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![
            BenchmarkStageEdge {
                from: "fastq.validate_reads.validator".to_string(),
                to: "fastq.trim_reads.fastp".to_string(),
                from_output_id: None,
                to_input_id: None,
            },
            BenchmarkStageEdge {
                from: "fastq.validate_reads.validator".to_string(),
                to: "fastq.trim_reads.fastp".to_string(),
                from_output_id: None,
                to_input_id: None,
            },
        ];
        let error = validate_suite(&suite).expect_err("duplicate explicit edges must fail");
        assert!(error.to_string().contains("must not repeat edge"));
    }

    #[test]
    fn suite_validation_allows_parallel_artifact_edges_between_same_nodes() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.profile_read_lengths".to_string(),
                    stage_instance_id: Some("fastq.profile_read_lengths.lengths".to_string()),
                    tools: vec!["seqkit_stats".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tools: vec!["multiqc".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![
            BenchmarkStageEdge {
                from: "fastq.profile_read_lengths.lengths".to_string(),
                to: "fastq.report_qc.aggregate".to_string(),
                from_output_id: Some("length_distribution_tsv".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
            BenchmarkStageEdge {
                from: "fastq.profile_read_lengths.lengths".to_string(),
                to: "fastq.report_qc.aggregate".to_string(),
                from_output_id: Some("length_distribution_json".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
        ];
        validate_suite(&suite)
            .expect("distinct artifact bindings between the same nodes should validate");
    }

    #[test]
    fn suite_validation_rejects_cycles_across_explicit_and_upstream_edges() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: vec!["fastq.report_qc.aggregate".to_string()],
                },
                BenchmarkStageSpec {
                    stage: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tools: vec!["multiqc".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![BenchmarkStageEdge {
            from: "fastq.validate_reads.validator".to_string(),
            to: "fastq.report_qc.aggregate".to_string(),
            from_output_id: None,
            to_input_id: None,
        }];
        let error = validate_suite(&suite).expect_err("cyclic suite graph must fail");
        assert!(error.to_string().contains("acyclic"));
    }

    #[test]
    fn suite_validation_accepts_artifact_aware_edges_with_stage_tool_nodes() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.report_qc".to_string(),
                    stage_instance_id: Some("fastq.report_qc.aggregate".to_string()),
                    tools: vec!["multiqc".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![BenchmarkStageEdge {
            from: "fastq.validate_reads.validator.tool.fastqvalidator".to_string(),
            to: "fastq.report_qc.aggregate.tool.multiqc".to_string(),
            from_output_id: Some("validation_report".to_string()),
            to_input_id: Some("qc_artifacts".to_string()),
        }];
        validate_suite(&suite)
            .expect("artifact-aware edges across stage-tool nodes should validate");
    }

    #[test]
    fn suite_validation_rejects_unknown_governed_output_ports() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                    tools: vec!["fastp".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![BenchmarkStageEdge {
            from: "fastq.validate_reads.validator".to_string(),
            to: "fastq.trim_reads.fastp".to_string(),
            from_output_id: Some("validated_reads_r1".to_string()),
            to_input_id: Some("reads_r1".to_string()),
        }];
        let error = validate_suite(&suite).expect_err("unknown governed output ports must fail");
        assert!(error
            .to_string()
            .contains("does not expose output validated_reads_r1"));
    }

    #[test]
    fn suite_validation_rejects_blank_edge_ports() {
        let mut suite = BenchmarkSuiteSpec::v1_stage_matrix(
            "suite".to_string(),
            vec![DatasetSpec {
                id: "dataset".to_string(),
                hash: "hash".to_string(),
                size: 1,
                origin: "synthetic".to_string(),
                class_label: "trueseq".to_string(),
                read_layout: "paired".to_string(),
            }],
            vec![
                BenchmarkStageSpec {
                    stage: "fastq.validate_reads".to_string(),
                    stage_instance_id: Some("fastq.validate_reads.validator".to_string()),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: "fastq.trim_reads".to_string(),
                    stage_instance_id: Some("fastq.trim_reads.fastp".to_string()),
                    tools: vec!["fastp".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
            ],
            ReplicatePolicy {
                count: 3,
                warmup: 0,
                seeds: vec![1, 2, 3],
            },
            DiversityRequirements {
                min_dataset_count: 1,
                min_classes: 1,
                min_read_layouts: 1,
            },
            vec![StratificationRequirement {
                key: "dataset_class".to_string(),
                required_values: vec!["trueseq".to_string()],
            }],
            AnalysisRequirements {
                require_bootstrap: false,
                require_outlier_detection: false,
                min_replicates_for_bootstrap: 5,
            },
        );
        suite.edges = vec![BenchmarkStageEdge {
            from: "fastq.validate_reads.validator".to_string(),
            to: "fastq.trim_reads.fastp".to_string(),
            from_output_id: Some("".to_string()),
            to_input_id: Some("reads_r1".to_string()),
        }];
        let error = validate_suite(&suite).expect_err("blank edge ports must fail");
        assert!(error.to_string().contains("blank from_output_id"));
    }
}

/// # Errors
/// Returns an error if required confounders are missing.
pub fn validate_observation(obs: &BenchmarkObservation) -> Result<(), BenchError> {
    if obs.schema_version != OBSERVATION_SCHEMA_V1 {
        return Err(BenchError::InvalidObservation {
            reason: format!("observation schema mismatch: {}", obs.schema_version),
        });
    }
    obs.validate()?;
    Ok(())
}

/// # Errors
/// Returns an error if summary schema is invalid.
pub fn validate_summary(summary: &BenchmarkSummary) -> Result<(), BenchError> {
    if summary.schema_version != SUMMARY_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "summary schema mismatch: {}",
            summary.schema_version
        )));
    }
    Ok(())
}

/// # Errors
/// Returns an error if decision schema is invalid.
pub fn validate_decision(decision: &GateDecision) -> Result<(), BenchError> {
    if decision.schema_version != DECISION_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "decision schema mismatch: {}",
            decision.schema_version
        )));
    }
    Ok(())
}
