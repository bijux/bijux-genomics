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
    prepare_primer_bank, build_contaminant_db, build_rrna_db, build_taxonomy_db,
    normalize_read_names, repair_pairs, interleave_reads, deinterleave_reads,
    concatenate_lanes, demultiplex_reads, deplete_host, deplete_rrna, subsample_reads,
    detect_instrument_artifacts,
    detect_adapters, detect_duplicates_premerge, estimate_library_complexity_prealign,
    extract_umis, DemultiplexRule, LaneInput, SubsampleTarget, trim_reads, validate_reads,
    HeaderInspection, MergeSuitability, NormalizedOutputs,
};
