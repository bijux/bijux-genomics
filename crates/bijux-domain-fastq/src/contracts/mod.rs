mod contract;
pub mod failure;
pub mod model;
pub mod pipeline_contract;
pub mod stages;

pub use contract::{
    contract_for_stage, ensure_umi_headers, inspect_headers, log_header_warnings,
    normalize_outputs, preflight_stage, HeaderInspection, NormalizedOutputs,
};
pub use failure::RawFailure;
pub use model::{
    FastqArtifact, FastqArtifactKind, FastqPE, FastqPairedEnd, FastqSE, FastqSingleEnd, FastqStats,
};
pub use stages::{infer_input_kind, qc_class_for_stage, FastqStageContract, QcClass};
