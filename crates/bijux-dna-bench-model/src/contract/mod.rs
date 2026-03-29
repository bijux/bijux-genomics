//! Owner: bijux-dna-bench
//! Schema versions and contract validators.
//! Owns validation for bench artifacts and inputs.
//! Must not perform IO.
#![allow(dead_code)]

use crate::error::BenchError;
use crate::model::{BenchmarkObservation, BenchmarkSuiteSpec, BenchmarkSummary};
use crate::policy::GateDecision;

mod edge_validation;
mod param_binding_validation;
mod schemas;
mod stage_governance;
mod suite_diversity;
mod suite_graph;
use edge_validation::validate_edge_ports;
use param_binding_validation::validate_stage_param_bindings;
pub use schemas::{DECISION_SCHEMA_V1, OBSERVATION_SCHEMA_V1, SUITE_SCHEMA_V1, SUMMARY_SCHEMA_V1};
use stage_governance::{ensure_supported_stage, planner_owned_graph_stage, validate_stage_tools};
use suite_diversity::validate_suite_diversity;
use suite_graph::{declared_graph_nodes, validate_suite_dag};

/// # Errors
/// Returns an error if the suite spec violates required fields.
pub fn validate_suite(suite: &BenchmarkSuiteSpec) -> Result<(), BenchError> {
    if suite.schema_version != SUITE_SCHEMA_V1 {
        return Err(BenchError::InvalidPolicy(format!(
            "suite schema mismatch: {}",
            suite.schema_version
        )));
    }
    validate_suite_diversity(suite)?;
    let mut seen_stage_nodes = std::collections::BTreeSet::new();
    let mut seen_stage_tool_nodes = std::collections::BTreeSet::new();
    let mut declared_stage_nodes = Vec::new();
    for stage in &suite.stages {
        if stage.stage.trim().is_empty() {
            return Err(BenchError::InvalidPolicy(
                "suite stages must include non-empty stage ids".to_string(),
            ));
        }
        ensure_supported_stage(&stage.stage)?;
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
                "suite must not repeat stage node {node_id}"
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
                    "suite must not repeat stage-tool node {tool_node_id}"
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
        validate_stage_param_bindings(stage)?;
        for upstream in &stage.upstream_stage_instance_ids {
            if upstream.trim().is_empty() {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {} must not include blank upstream_stage_instance_ids",
                    stage.stage
                )));
            }
            if upstream == node_id {
                return Err(BenchError::InvalidPolicy(format!(
                    "suite stage {node_id} must not reference itself as an upstream stage"
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
                    "suite stage {node_id} references unknown upstream stage node {upstream}"
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

#[cfg(test)]
mod tests {
    use super::validate_suite;
    use crate::{
        AnalysisRequirements, BenchmarkParamBinding, BenchmarkStageEdge, BenchmarkStageSpec,
        BenchmarkSuiteSpec, DatasetSpec, DiversityRequirements, ReplicatePolicy,
        StratificationRequirement,
    };
    use bijux_dna_core::id_catalog;

    fn fastq_stage(name: &str) -> String {
        format!("{}{}", id_catalog::FASTQ_PREFIX, name)
    }

    fn stage_instance(stage_id: &str, suffix: &str) -> String {
        format!("{stage_id}.{suffix}")
    }

    fn stage_tool_instance(stage_id: &str, suffix: &str, tool_id: &str) -> String {
        format!("{}.tool.{tool_id}", stage_instance(stage_id, suffix))
    }

    fn fastq_instance(name: &str, suffix: &str) -> String {
        stage_instance(&fastq_stage(name), suffix)
    }

    fn fastq_tool_instance(name: &str, suffix: &str, tool_id: &str) -> String {
        stage_tool_instance(&fastq_stage(name), suffix, tool_id)
    }

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

    fn suite_error(suite: &BenchmarkSuiteSpec) -> String {
        match validate_suite(suite) {
            Ok(()) => panic!("suite unexpectedly validated"),
            Err(error) => error.to_string(),
        }
    }

    fn assert_valid_suite(suite: &BenchmarkSuiteSpec) {
        if let Err(error) = validate_suite(suite) {
            panic!("suite unexpectedly failed validation: {error}");
        }
    }

    #[test]
    fn suite_validation_rejects_duplicate_stage_tools() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string(), "fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("must not repeat tool"));
    }

    #[test]
    fn suite_validation_rejects_unknown_fastq_stage_ids() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: fastq_stage("unknown_stage"),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("is not declared in the FASTQ domain catalog"));
    }

    #[test]
    fn suite_validation_accepts_governed_fastq_stages_with_planning_support() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: fastq_stage("infer_asvs"),
            stage_instance_id: None,
            tools: vec!["dada2".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        assert_valid_suite(&suite);
    }

    #[test]
    fn suite_validation_rejects_fastq_tools_outside_execution_support() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: fastq_stage("filter_low_complexity"),
            stage_instance_id: None,
            tools: vec!["dustmasker".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("is not admitted by FASTQ execution support"));
    }

    #[test]
    fn suite_validation_accepts_registered_bam_stage_ids() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::BAM_ALIGN.to_string(),
            stage_instance_id: None,
            tools: vec!["bwa".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        assert_valid_suite(&suite);
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
        assert_valid_suite(&suite);
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
        let error = suite_error(&suite);
        assert!(error.contains("must not declare tool bindings"));
    }

    #[test]
    fn suite_validation_rejects_blank_stage_params() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: vec![String::new()],
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("blank params entries"));
    }

    #[test]
    fn suite_validation_rejects_mixed_legacy_and_structured_stage_params() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: vec!["threads=4".to_string()],
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool.fastp")),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("either params or param_bindings"));
    }

    #[test]
    fn suite_validation_accepts_structured_stage_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool.fastp")),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        assert_valid_suite(&suite);
    }

    #[test]
    fn suite_validation_rejects_param_binding_targets_outside_stage_tool_nodes() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: None,
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(format!(
                    "{}.tool.{}",
                    id_catalog::FASTQ_DETECT_ADAPTERS,
                    id_catalog::TOOL_FASTQC
                )),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("must match the stage node or one of its stage-tool nodes"));
    }

    #[test]
    fn suite_validation_rejects_tool_scoped_binding_targeting_stage_node() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "cleanup")),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "cleanup")),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("must target its stage-tool node"));
    }

    #[test]
    fn suite_validation_rejects_tool_scoped_binding_with_mismatched_tool_node() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "cleanup")),
            tools: vec!["fastp".to_string(), "bbduk".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_tool_instance(
                    id_catalog::FASTQ_TRIM,
                    "cleanup",
                    "bbduk",
                )),
                tool: Some("fastp".to_string()),
                values: std::collections::BTreeMap::from([(
                    "threads".to_string(),
                    serde_json::json!(4),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("must match target tool node"));
    }

    #[test]
    fn suite_validation_accepts_governed_stage_scoped_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool_cohort")),
            tools: vec!["fastp".to_string(), "bbduk".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool_cohort")),
                tool: None,
                values: std::collections::BTreeMap::from([
                    ("min_length".to_string(), serde_json::json!(30)),
                    ("quality_cutoff".to_string(), serde_json::json!(20)),
                    ("adapter_policy".to_string(), serde_json::json!("auto")),
                ]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        assert_valid_suite(&suite);
    }

    #[test]
    fn suite_validation_rejects_unknown_stage_scoped_param_bindings() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool_cohort")),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: vec![BenchmarkParamBinding {
                stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "tool_cohort")),
                tool: None,
                values: std::collections::BTreeMap::from([(
                    "adapter_bank_path".to_string(),
                    serde_json::json!("bank.json"),
                )]),
            }],
            upstream_stage_instance_ids: Vec::new(),
        });
        let error = suite_error(&suite);
        assert!(error.contains("governed stage parameter contract"));
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
                    stage: id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
                    tools: vec!["fastp".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "cutadapt")),
                    tools: vec!["cutadapt".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: vec![stage_instance(
                        id_catalog::FASTQ_TRIM,
                        "fastp",
                    )],
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
        assert_valid_suite(&suite);
    }

    #[test]
    fn suite_validation_rejects_unknown_upstream_stage_nodes() {
        let suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: vec!["missing.stage.node".to_string()],
        });
        let error = suite_error(&suite);
        assert!(error.contains("unknown upstream stage node"));
    }

    #[test]
    fn suite_validation_rejects_unknown_explicit_edge_nodes() {
        let mut suite = suite_with_stage(BenchmarkStageSpec {
            stage: id_catalog::FASTQ_TRIM.to_string(),
            stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
            tools: vec!["fastp".to_string()],
            params: Vec::new(),
            param_bindings: Vec::new(),
            upstream_stage_instance_ids: Vec::new(),
        });
        suite.edges = vec![BenchmarkStageEdge {
            from: stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
            to: "missing.node".to_string(),
            from_output_id: None,
            to_input_id: None,
        }];
        let error = suite_error(&suite);
        assert!(error.contains("unknown target node"));
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
                    stage: id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        id_catalog::FASTQ_VALIDATE_READS,
                        "validator",
                    )),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
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
                from: stage_instance(id_catalog::FASTQ_VALIDATE_READS, "validator"),
                to: stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
                from_output_id: None,
                to_input_id: None,
            },
            BenchmarkStageEdge {
                from: stage_instance(id_catalog::FASTQ_VALIDATE_READS, "validator"),
                to: stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
                from_output_id: None,
                to_input_id: None,
            },
        ];
        let error = suite_error(&suite);
        assert!(error.contains("must not repeat edge"));
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
                    stage: fastq_stage("profile_read_lengths"),
                    stage_instance_id: Some(fastq_instance("profile_read_lengths", "lengths")),
                    tools: vec!["seqkit_stats".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_QC_POST.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_QC_POST, "aggregate")),
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
                from: fastq_instance("profile_read_lengths", "lengths"),
                to: stage_instance(id_catalog::FASTQ_QC_POST, "aggregate"),
                from_output_id: Some("length_distribution_tsv".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
            BenchmarkStageEdge {
                from: fastq_instance("profile_read_lengths", "lengths"),
                to: stage_instance(id_catalog::FASTQ_QC_POST, "aggregate"),
                from_output_id: Some("length_distribution_json".to_string()),
                to_input_id: Some("qc_artifacts".to_string()),
            },
        ];
        assert_valid_suite(&suite);
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
                    stage: id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        id_catalog::FASTQ_VALIDATE_READS,
                        "validator",
                    )),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: vec![stage_instance(
                        id_catalog::FASTQ_QC_POST,
                        "aggregate",
                    )],
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_QC_POST.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_QC_POST, "aggregate")),
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
            from: stage_instance(id_catalog::FASTQ_VALIDATE_READS, "validator"),
            to: stage_instance(id_catalog::FASTQ_QC_POST, "aggregate"),
            from_output_id: None,
            to_input_id: None,
        }];
        let error = suite_error(&suite);
        assert!(error.contains("acyclic"));
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
                    stage: id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        id_catalog::FASTQ_VALIDATE_READS,
                        "validator",
                    )),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_QC_POST.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_QC_POST, "aggregate")),
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
            from: stage_tool_instance(
                id_catalog::FASTQ_VALIDATE_READS,
                "validator",
                id_catalog::TOOL_FASTQVALIDATOR_OFFICIAL,
            ),
            to: stage_tool_instance(
                id_catalog::FASTQ_QC_POST,
                "aggregate",
                id_catalog::TOOL_MULTIQC,
            ),
            from_output_id: Some("validation_report".to_string()),
            to_input_id: Some("qc_artifacts".to_string()),
        }];
        assert_valid_suite(&suite);
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
                    stage: id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        id_catalog::FASTQ_VALIDATE_READS,
                        "validator",
                    )),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
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
            from: stage_instance(id_catalog::FASTQ_VALIDATE_READS, "validator"),
            to: stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
            from_output_id: Some("validated_reads_r1".to_string()),
            to_input_id: Some("reads_r1".to_string()),
        }];
        let error = suite_error(&suite);
        assert!(error.contains("does not expose output validated_reads_r1"));
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
                    stage: id_catalog::FASTQ_VALIDATE_READS.to_string(),
                    stage_instance_id: Some(stage_instance(
                        id_catalog::FASTQ_VALIDATE_READS,
                        "validator",
                    )),
                    tools: vec!["fastqvalidator".to_string()],
                    params: Vec::new(),
                    param_bindings: Vec::new(),
                    upstream_stage_instance_ids: Vec::new(),
                },
                BenchmarkStageSpec {
                    stage: id_catalog::FASTQ_TRIM.to_string(),
                    stage_instance_id: Some(stage_instance(id_catalog::FASTQ_TRIM, "fastp")),
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
            from: stage_instance(id_catalog::FASTQ_VALIDATE_READS, "validator"),
            to: stage_instance(id_catalog::FASTQ_TRIM, "fastp"),
            from_output_id: Some(String::new()),
            to_input_id: Some("reads_r1".to_string()),
        }];
        let error = suite_error(&suite);
        assert!(error.contains("blank from_output_id"));
    }
}
