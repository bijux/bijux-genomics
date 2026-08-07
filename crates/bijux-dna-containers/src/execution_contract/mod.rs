//! Typed declarations admitted before container execution.

mod credential;
mod pipeline;

pub use credential::{
    container_credential_reference_schema, load_container_credential_reference,
    validate_container_credential_reference, write_container_credential_reference_schema,
    ContainerCredentialRecord, ContainerCredentialRedaction, ContainerCredentialReference,
    CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1,
};
pub use pipeline::{
    admit_container_pipeline_site, load_container_pipeline, validate_container_pipeline,
    ContainerPipelineContract, ContainerPipelineInputAdmission, ContainerPipelineSiteAdmission,
    ContainerPipelineSiteAdmissionRequest, ContainerPipelineV1, ContainerPipelineV2,
    PipelineAcceptanceMetric, PipelineAcceptanceV1, PipelineAcceptanceV2, PipelineCachePolicy,
    PipelineCommandsV1, PipelineCommandsV2, PipelineDataV1, PipelineDataV2, PipelineDependency,
    PipelineDeterminism, PipelineDownstreamProbe, PipelineEvidenceKind, PipelineExecutionV1,
    PipelineExecutionV2, PipelineImage, PipelineInputKind, PipelineInputV1, PipelineInputV2,
    PipelineInterfaceEvidence, PipelineMeasurement, PipelineMetricOperator, PipelineOutputsV1,
    PipelineOutputsV2, PipelineParameterEvidence, PipelineRepository, PipelineResourcesV1,
    PipelineResourcesV2, PipelineRetry, PipelineRunV1, PipelineRunV2, CONTAINER_PIPELINE_SCHEMA_V1,
    CONTAINER_PIPELINE_SCHEMA_V2,
};
