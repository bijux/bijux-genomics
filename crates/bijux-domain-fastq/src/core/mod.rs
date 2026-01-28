pub mod contracts;
pub mod correct;
pub mod failure;
pub mod filter;
pub mod merge;
pub mod model;
pub mod stages;
pub mod stats;
pub mod trim;
pub mod validate;

pub use contracts::{
    contract_for_stage, ensure_umi_headers, inspect_headers, log_header_warnings,
    normalize_outputs, preflight_stage, HeaderInspection, NormalizedOutputs,
};
pub use failure::RawFailure;
pub use model::{FastqArtifact, FastqArtifactKind, FastqPE, FastqSE, FastqStats};
pub use stages::{infer_input_kind, qc_class_for_stage, FastqStageContract, QcClass};
