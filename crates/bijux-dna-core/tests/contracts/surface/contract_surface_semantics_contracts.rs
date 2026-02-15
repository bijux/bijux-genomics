use std::collections::BTreeMap;
use std::path::PathBuf;

use bijux_dna_core::contract::execution::{ExecutionEdge, ExecutionGraph, ExecutionStep};
use bijux_dna_core::contract::{
    ArtifactRole, ContractVersion, PlanPolicy, StageIO, ToolConstraints,
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
fn artifact_roles_and_specs_cover_enum_and_constructor_paths() {
    let all_roles = [
        ArtifactRole::Reads,
        ArtifactRole::TrimmedReads,
        ArtifactRole::Bam,
        ArtifactRole::DedupBam,
        ArtifactRole::ReportJson,
        ArtifactRole::Log,
        ArtifactRole::Index,
        ArtifactRole::MetricsJson,
        ArtifactRole::MetricsEnvelope,
        ArtifactRole::StageReport,
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
        "report_json",
        "log",
        "index",
        "metrics_json",
        "metrics_envelope",
        "stage_report",
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
    let edge = ExecutionEdge::new(StepId::new("a"), StepId::new("b"));
    assert_eq!(edge.from().as_str(), "a");
    assert_eq!(edge.to().as_str(), "b");

    let graph = ExecutionGraph::new(
        "fastq-to-fastq__default__v1",
        "planner-v2",
        policy,
        vec![mk_step("b", "fastq.trim"), mk_step("a", "fastq.qc_post")],
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
    assert!(graph.retry_policy().max_attempts >= 1);
}
