mod catalog;
mod export;
mod runtime;

pub use catalog::contract_for_stage;
pub use export::{stage_contract_hash, stage_contract_json};
pub use runtime::{
    assess_merge_suitability, classify_layout, ensure_layout_is_coherent, ensure_umi_headers,
    find_first_fastq, inspect_headers, log_header_warnings, materialize_qc_manifest,
    normalize_outputs, preflight_stage, HeaderInspection, MergeSuitability, NormalizedOutputs,
};
