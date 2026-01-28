pub mod contracts;
pub mod model;
pub mod stages;

pub use contracts::{
    contract_for_stage, inspect_headers, log_header_warnings, normalize_outputs, preflight_stage,
    HeaderInspection, NormalizedOutputs,
};
pub use model::{FastqArtifact, FastqArtifactKind};
pub use stages::{infer_input_kind, qc_class_for_stage, FastqStageContract, QcClass};
