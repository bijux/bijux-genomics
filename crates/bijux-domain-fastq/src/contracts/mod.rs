mod contract;
pub mod model;
pub mod pipeline_contract;
pub mod stages;

pub use bijux_core::RawFailure;
pub use contract::{
    assess_merge_suitability, contract_for_stage, ensure_umi_headers, inspect_headers,
    log_header_warnings, normalize_outputs, preflight_stage, HeaderInspection, MergeSuitability,
    NormalizedOutputs,
};
pub use model::{
    FastqArtifact, FastqArtifactKind, FastqLayout, FastqPE, FastqPairedEnd, FastqSE, FastqSampleId,
    FastqSingleEnd, FastqStats,
};
pub use stages::{infer_input_kind, qc_class_for_stage, FastqStageContract, QcClass};
