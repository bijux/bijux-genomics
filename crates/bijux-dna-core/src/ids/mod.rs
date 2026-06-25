#![allow(missing_docs)]

mod domain_model;
mod parsing;
mod typed;

pub use crate::id_catalog;
pub use domain_model::{
    AssayKind, DomainKind, LibraryLayout, LibraryModel, PlatformHint, UdgTreatment,
};
pub use parsing::{
    build_asset_profile_benchmark_result_id, build_sample_scoped_benchmark_result_id,
    parse_benchmark_result_id, parse_pipeline_id, parse_stage_id, parse_tool_id, parse_variant_id,
    validate_artifact_id, validate_artifact_id_str, validate_pipeline_id, validate_pipeline_id_str,
    validate_profile_id, validate_profile_id_str, validate_stage_id, validate_stage_id_str,
    validate_tool_id, validate_tool_id_str, BenchmarkResultIdentity, BenchmarkResultScopeKind,
    VariantIdentity,
};
pub use typed::{
    ArtifactId, ImageDigest, PipelineId, ProfileId, RunId, StageId, StageVersion, StepId, ToolId,
    ToolVersion,
};
