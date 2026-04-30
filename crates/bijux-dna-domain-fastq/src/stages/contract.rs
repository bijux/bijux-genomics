mod catalog;
mod export;
mod runtime;

pub use catalog::contract_for_stage;
pub use export::{stage_contract_hash, stage_contract_json};
pub use runtime::{
    assess_merge_suitability, classify_layout, ensure_layout_is_coherent, ensure_umi_headers,
    find_first_fastq, inspect_headers, log_header_warnings, materialize_qc_manifest,
    normalize_outputs, preflight_stage, verify_assets, ensure_assets_verified,
    capture_provenance_snapshot, prepare_adapter_bank, prepare_host_reference_bundle,
    prepare_primer_bank, build_contaminant_db, build_rrna_db, HeaderInspection,
    MergeSuitability, NormalizedOutputs,
};
