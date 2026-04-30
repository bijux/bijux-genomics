//! Canonical pipeline profiles, defaults ledgers, manifests, and registry lookups
//! for FASTQ, BAM, VCF, and cross-domain workflows.

pub mod bam;
pub mod contract;
pub mod cross;
pub mod defaults;
pub mod fastq;
/// Curated mirror of the stable public surface.
pub mod public_api;
pub mod registry;
pub mod vcf;

pub const STAGE_CORE_PREPARE_REFERENCE: &str = "core.prepare_reference";
pub const STAGE_CROSS_ALIGN_STUB: &str = "cross.align_stub";

pub use contract::{
    build_batch_workflow_graph, evaluate_template_admission, parse_sample_sheet,
    sample_sheet_to_workflow_manifests, summarize_cross_domain_evidence,
    validate_template_overrides,
    ArtifactType, Domain, EffectiveDefaults, InvariantSeverity, InvariantViolationV1,
    InvariantsPreset, InvariantsReportV1, MetricsBundle, PipelineCapabilities, PipelineContract,
    PipelineProfile, ProfileManifestV1, ReportSection, StabilityTier,
    BatchNodeScopeV1, BatchWorkflowSemanticsV1, CrossDomainEvidenceSummaryV1,
    CrossDomainFailurePolicyV1, CrossWorkflowTemplateV1, FanArtifactRuleV1, FanPatternV1,
    SampleSheetFormatV1, SampleSheetRecordV1, SampleSheetV1, TemplateFailureActionV1,
    TemplateParameterPolicyV1, WorkflowBatchEdgeV1, WorkflowBatchGraphV1,
    WorkflowBatchNodeV1, WorkflowEvidenceSummarySectionV1, WorkflowEvidenceSummaryStoryV1,
    WorkflowTemplateAdmissionCheckV1, WorkflowTemplateAdmissionV1,
};
pub use defaults::merge_effective_defaults;
pub use defaults::{DefaultParams, DefaultProvenanceV1, DefaultsLedgerV1, EmptyParams};
pub use registry::{validate_pipeline_id, validate_pipeline_id_str, PipelineId};

pub type PipelineProfileV1 = PipelineProfile;
