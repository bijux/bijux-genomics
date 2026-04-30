use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::execution::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::contract::{
    ArtifactRole, ContractVersion, PlanPolicy, StageIO, StageParameterSpec, StageSpec,
    ToolConstraints,
};
use bijux_dna_core::metrics::{
    parse_derived_metric_id, parse_metric_id, validate_derived_metric_id_str,
    validate_metric_id_str, DerivedMetricId, MetricId,
};
use bijux_dna_core::prelude::{ArtifactId, CommandSpecV1, ContainerImageRefV1, StageId, StepId};

fn mk_step(step_id: &str, stage_id: &str) -> ExecutionStep {
    ExecutionStep {
        step_id: StepId::new(step_id),
        stage_id: StageId::new(stage_id),
        command: CommandSpecV1 { template: vec!["echo".to_string(), "ok".to_string()] },
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
fn artifact_roles_and_specs_cover_enum_and_constructor_paths() {
    let all_roles = [
        ArtifactRole::Reads,
        ArtifactRole::TrimmedReads,
        ArtifactRole::Bam,
        ArtifactRole::DedupBam,
        ArtifactRole::Alignment,
        ArtifactRole::Variant,
        ArtifactRole::ReportJson,
        ArtifactRole::Report,
        ArtifactRole::Log,
        ArtifactRole::Reference,
        ArtifactRole::Index,
        ArtifactRole::MetricsJson,
        ArtifactRole::MetricsEnvelope,
        ArtifactRole::Metrics,
        ArtifactRole::StageReport,
        ArtifactRole::Provenance,
        ArtifactRole::Evidence,
        ArtifactRole::SummaryJson,
        ArtifactRole::SummaryTsv,
        ArtifactRole::ReportHtml,
        ArtifactRole::Unknown,
    ];
    let expected = [
        "reads",
        "trimmed_reads",
        "bam",
        "dedup_bam",
        "alignment",
        "variant",
        "report_json",
        "report",
        "log",
        "reference",
        "index",
        "metrics_json",
        "metrics_envelope",
        "metrics",
        "stage_report",
        "provenance",
        "evidence",
        "summary_json",
        "summary_tsv",
        "report_html",
        "unknown",
    ];
    for (role, expected_str) in all_roles.into_iter().zip(expected) {
        assert_eq!(role.as_str(), expected_str);
    }

    let required = bijux_dna_core::contract::ArtifactSpec::required(
        ArtifactId::new("reads"),
        PathBuf::from("reads.fastq.gz"),
        ArtifactRole::Reads,
    );
    assert!(!required.optional);
    assert_eq!(required.name.as_str(), "reads");

    let optional = bijux_dna_core::contract::ArtifactSpec::optional(
        ArtifactId::new("report"),
        PathBuf::from("report.json"),
        ArtifactRole::ReportJson,
    );
    assert!(optional.optional);
    assert_eq!(optional.path, PathBuf::from("report.json"));
}

#[test]
fn stage_parameter_canonicalization_resolves_aliases_and_defaults() {
    let stage = StageSpec {
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        stage_family: bijux_dna_core::contract::StageFamily::Fastq,
        semantic_kind: bijux_dna_core::contract::StageSemanticKind::Filter,
        input_kind: bijux_dna_core::contract::ArtifactKind::Fastq,
        output_kind: bijux_dna_core::contract::ArtifactKind::Fastq,
        produced_artifacts: vec!["trimmed_reads".to_string()],
        stage_semver: "1.0.0".to_string(),
        runtime_scale: bijux_dna_core::contract::RuntimeScale::Small,
        inputs: Vec::new(),
        outputs: Vec::new(),
        parameters: vec![
            StageParameterSpec {
                name: "min_quality".to_string(),
                param_type: "integer".to_string(),
                default: Some("20".to_string()),
                aliases: vec!["quality".to_string()],
            },
            StageParameterSpec {
                name: "trim_poly_g".to_string(),
                param_type: "bool".to_string(),
                default: Some("false".to_string()),
                aliases: Vec::new(),
            },
        ],
        metrics: Vec::new(),
        description: None,
        environment_requirements: Default::default(),
        report_contracts: Vec::new(),
        capability_contract: Default::default(),
        refusal_codes: Vec::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        behavior: Default::default(),
        image_requirements: None,
        extends: None,
    };

    let canonical = stage
        .canonicalize_parameters(&BTreeMap::from([("quality".to_string(), "30".to_string())]))
        .unwrap_or_else(|err| panic!("canonicalize parameters: {err}"));

    assert_eq!(
        canonical.normalized_json,
        serde_json::json!({"min_quality": "30", "trim_poly_g": "false"})
    );
    assert_eq!(canonical.resolved_aliases.get("quality").map(String::as_str), Some("min_quality"));
    assert_eq!(canonical.applied_defaults, vec!["trim_poly_g".to_string()]);
}

#[test]
fn metric_id_as_str_roundtrip_covers_all_variants() {
    let all_metrics = [
        MetricId::RuntimeS,
        MetricId::MemoryMb,
        MetricId::ExitCode,
        MetricId::ReadsIn,
        MetricId::ReadsOut,
        MetricId::ReadsDropped,
        MetricId::ReadsRemovedByN,
        MetricId::ReadsRemovedByEntropy,
        MetricId::ReadsRemovedLowComplexity,
        MetricId::ReadsRemovedByKmer,
        MetricId::ReadsRemovedContaminantKmer,
        MetricId::ReadsRemovedByLength,
        MetricId::ReadsTotal,
        MetricId::ReadsValid,
        MetricId::ReadsInvalid,
        MetricId::BasesIn,
        MetricId::BasesOut,
        MetricId::BasesTotal,
        MetricId::PairsIn,
        MetricId::PairsOut,
        MetricId::ReadsR1,
        MetricId::ReadsR2,
        MetricId::ReadsMerged,
        MetricId::ReadsUnmerged,
        MetricId::MeanQBefore,
        MetricId::MeanQAfter,
        MetricId::MeanQ,
        MetricId::MergeRate,
        MetricId::DedupRate,
        MetricId::KmerFixRate,
        MetricId::ContaminationRate,
        MetricId::ContaminationSummary,
        MetricId::GcPercent,
        MetricId::LengthHistogram,
        MetricId::DeltaMetrics,
        MetricId::AdapterPreset,
        MetricId::AdapterBankId,
        MetricId::AdapterBankHash,
        MetricId::AdapterOverrides,
        MetricId::QcRawDir,
        MetricId::QcTrimmedDir,
        MetricId::MultiqcReport,
        MetricId::MultiqcData,
    ];
    for metric in all_metrics {
        let id = metric.as_str();
        assert!(parse_metric_id(id).is_some());
        assert!(validate_metric_id_str(id).is_ok());
    }
    assert!(parse_metric_id("unknown").is_none());

    let all_derived = [
        DerivedMetricId::ReadRetention,
        DerivedMetricId::BaseRetention,
        DerivedMetricId::MergeEfficiency,
        DerivedMetricId::ErrorReductionProxy,
    ];
    for metric in all_derived {
        let id = metric.as_str();
        assert!(parse_derived_metric_id(id).is_some());
        assert!(validate_derived_metric_id_str(id).is_ok());
    }
    assert!(validate_derived_metric_id_str("unknown").is_err());
}

#[test]
fn execution_graph_getters_and_edge_accessors_are_stable() {
    let policy = PlanPolicy::default();
    let edge = ExecutionEdge::with_artifact_binding(
        StepId::new("a"),
        StepId::new("b"),
        ArtifactId::new("out"),
        ArtifactId::new("in"),
    );
    assert_eq!(edge.from().as_str(), "a");
    assert_eq!(edge.to().as_str(), "b");
    assert_eq!(
        edge.from_output_id().map(bijux_dna_core::contract::ArtifactId::as_str),
        Some("out")
    );
    assert_eq!(edge.to_input_id().map(bijux_dna_core::contract::ArtifactId::as_str), Some("in"));

    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v2",
        policy,
        vec![mk_step("b", "fastq.trim_reads"), mk_step("a", "fastq.report_qc")],
        vec![edge],
    );
    assert!(graph.is_ok());
    let graph = graph.unwrap_or_else(|err| panic!("graph builds: {err}"));

    assert_eq!(graph.schema_version(), "bijux.execution_graph.v1");
    assert_eq!(graph.contract_version(), ContractVersion::v1());
    assert_eq!(graph.pipeline_id().as_str(), "fastq-to-fastq__default__v1");
    assert_eq!(graph.planner_version(), "planner-v2");
    assert_eq!(graph.policy(), policy);
    assert!(graph.deterministic_scheduler());
    assert_eq!(graph.step_timeout_s(), None);
    assert_eq!(graph.steps().len(), 2);
    assert_eq!(graph.edges().len(), 1);
    assert_eq!(graph.edges()[0].from().as_str(), "a");
    assert_eq!(graph.edges()[0].to().as_str(), "b");
    assert_eq!(
        graph.edges()[0].from_output_id().map(bijux_dna_core::contract::ArtifactId::as_str),
        Some("out")
    );
    assert_eq!(
        graph.edges()[0].to_input_id().map(bijux_dna_core::contract::ArtifactId::as_str),
        Some("in")
    );
    assert!(graph.retry_policy().max_attempts >= 1);
}
