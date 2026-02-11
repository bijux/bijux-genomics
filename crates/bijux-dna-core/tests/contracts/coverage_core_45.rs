use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::execution::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::contract::{
    objective_spec, select_stage, BenchResultRecord, BenchResultStatus, Objective,
};
use bijux_dna_core::contract::{ArtifactRole, PlanPolicy, StageIO, ToolConstraints};
use bijux_dna_core::ids::{
    parse_pipeline_id, parse_stage_id, parse_tool_id, validate_pipeline_id_str,
};
use bijux_dna_core::metrics::{parse_derived_metric_id, parse_metric_id, validate_metric_id_str};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

fn mk_step(step_id: &str, stage_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new(stage_id),
        command: CommandSpecV1 {
            template: vec!["echo".to_string(), "ok".to_string()],
        },
        image: ContainerImageRefV1 {
            image: "local/tool:latest".to_string(),
            digest: Some("sha256:abc".to_string()),
        },
        resources: ToolConstraints::default(),
        io: StageIO {
            inputs: vec![bijux_dna_core::contract::ArtifactSpec::required(
                ArtifactId::new("in"),
                PathBuf::from("in.fastq"),
                ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::ArtifactSpec::required(
                ArtifactId::new("out"),
                PathBuf::from("out.fastq"),
                ArtifactRole::Reads,
            )],
        },
        out_dir: PathBuf::from("out"),
        aux_images: BTreeMap::new(),
        expected_artifact_ids: Vec::new(),
        metrics_schema_ids: Vec::new(),
    }
}

#[test]
fn metric_id_parsers_cover_known_and_unknown_values() {
    let ids = [
        "runtime_s",
        "memory_mb",
        "exit_code",
        "reads_in",
        "reads_out",
        "reads_dropped",
        "reads_removed_by_n",
        "reads_removed_by_entropy",
        "reads_removed_low_complexity",
        "reads_removed_by_kmer",
        "reads_removed_contaminant_kmer",
        "reads_removed_by_length",
        "reads_total",
        "reads_valid",
        "reads_invalid",
        "bases_in",
        "bases_out",
        "bases_total",
        "pairs_in",
        "pairs_out",
        "reads_r1",
        "reads_r2",
        "reads_merged",
        "reads_unmerged",
        "mean_q_before",
        "mean_q_after",
        "mean_q",
        "merge_rate",
        "dedup_rate",
        "kmer_fix_rate",
        "contamination_rate",
        "contamination_summary",
        "gc_percent",
        "length_histogram",
        "delta_metrics",
        "adapter_preset",
        "adapter_bank_id",
        "adapter_bank_hash",
        "adapter_overrides",
        "qc_raw_dir",
        "qc_trimmed_dir",
        "multiqc_report",
        "multiqc_data",
    ];
    for id in ids {
        assert!(parse_metric_id(id).is_some(), "expected known metric id: {id}");
        assert!(validate_metric_id_str(id).is_ok());
    }
    assert!(parse_metric_id("unknown_metric").is_none());
    assert!(validate_metric_id_str("unknown_metric").is_err());

    let derived = [
        "read_retention",
        "base_retention",
        "merge_efficiency",
        "error_reduction_proxy",
    ];
    for id in derived {
        assert!(
            parse_derived_metric_id(id).is_some(),
            "expected known derived metric id: {id}"
        );
    }
    assert!(parse_derived_metric_id("unknown_derived").is_none());
}

#[test]
fn selection_objectives_and_partial_behavior_are_deterministic() {
    for objective in [
        Objective::Speed,
        Objective::Memory,
        Objective::Retention,
        Objective::Balanced,
    ] {
        let spec = objective_spec(objective);
        assert_eq!(spec.name, objective.as_str());
    }

    let stage = StageId::new("fastq.trim");
    let fast = vec![BenchResultRecord {
        dataset_id: "d1".to_string(),
        tool: "fast".to_string(),
        runtime_s: Some(1.0),
        memory_mb: Some(100.0),
        exit_code: Some(0),
        metrics: Some(serde_json::json!({"retention": {"value": 0.95}})),
        status: BenchResultStatus::Success,
    }];
    let slow = vec![BenchResultRecord {
        dataset_id: "d1".to_string(),
        tool: "slow".to_string(),
        runtime_s: Some(5.0),
        memory_mb: Some(150.0),
        exit_code: Some(0),
        metrics: Some(serde_json::json!({"retention": 0.90})),
        status: BenchResultStatus::Success,
    }];
    let failing = vec![BenchResultRecord {
        dataset_id: "d1".to_string(),
        tool: "bad".to_string(),
        runtime_s: None,
        memory_mb: None,
        exit_code: Some(1),
        metrics: None,
        status: BenchResultStatus::Failure,
    }];
    let records = vec![
        ("fast".to_string(), fast),
        ("slow".to_string(), slow),
        ("bad".to_string(), failing),
    ];

    let selected_speed = select_stage(&stage, &records, &objective_spec(Objective::Speed), false);
    assert_eq!(selected_speed.selected.as_deref(), Some("fast"));
    assert!(!selected_speed.disqualified.is_empty());

    let selected_allow_partial =
        select_stage(&stage, &records, &objective_spec(Objective::Balanced), true);
    assert!(selected_allow_partial.selected.is_some());
    assert_eq!(selected_allow_partial.stage.as_str(), "fastq.trim");
}

#[test]
fn ids_validation_covers_success_and_failure_paths() {
    assert!(parse_stage_id("fastq.trim").is_ok());
    assert!(parse_tool_id("fastp").is_ok());
    assert!(parse_pipeline_id("fastq-to-fastq__default__v1").is_ok());

    assert!(parse_stage_id("invalid").is_err());
    assert!(parse_tool_id("FASTP").is_err());
    assert!(validate_pipeline_id_str("fastq-to-fastq__default").is_err());
    assert!(validate_pipeline_id_str("badgraph__default__v1").is_err());
}

#[test]
fn execution_graph_option_helpers_and_validation_paths() {
    let a = mk_step("b", "fastq.trim");
    let b = mk_step("a", "fastq.qc_post");
    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![a, b],
        vec![ExecutionEdge::new(StepId::new("a"), StepId::new("b"))],
    )
    .expect("graph should build");

    let graph = graph
        .with_retry_policy(Default::default())
        .with_deterministic_scheduler(false)
        .with_step_timeout(Some(30));
    assert_eq!(graph.step_timeout_s(), Some(30));
    assert!(!graph.deterministic_scheduler());
    assert_eq!(graph.steps()[0].step_id.as_str(), "a");
    assert!(graph.hash().is_ok());
    assert!(graph.normalize().is_ok());
    assert!(graph.validate_strict().is_ok());

    let bad_step = ExecutionStep {
        io: StageIO {
            inputs: Vec::new(),
            outputs: Vec::new(),
        },
        ..mk_step("x", "fastq.trim")
    };
    let bad_graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v1",
        PlanPolicy::default(),
        vec![bad_step],
        Vec::new(),
    );
    assert!(bad_graph.is_err());
}
