use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::super::canonical_contract_for_stage;
use super::catalog::stage_for_id;
use crate::types::FastqArtifactKind;

mod asset_verification;
mod build_contaminant_db;
mod build_rrna_db;
mod build_taxonomy_db;
mod chunked_preprocess;
mod concatenate_lanes;
mod deinterleave_reads;
mod demultiplex_reads;
mod deplete_host;
mod deplete_reference_contaminants;
mod deplete_rrna;
mod detect_adapters;
mod detect_duplicates_premerge;
mod detect_instrument_artifacts;
mod edna_branch;
mod estimate_library_complexity_prealign;
mod extract_umis;
mod fastq_io;
mod header_inspection;
mod interleave_reads;
mod layout_classification;
mod merge_pairs;
mod merge_suitability;
mod normalize_read_names;
mod output_normalization;
mod prepare_adapter_bank;
mod prepare_host_reference_bundle;
mod prepare_primer_bank;
mod provenance_snapshot;
mod qc_manifest;
mod repair_pairs;
mod screen_taxonomy;
mod subsample_reads;
mod remove_duplicates;
mod trim_reads;
mod validate_reads;

pub use asset_verification::{ensure_assets_verified, verify_assets};
pub use build_contaminant_db::build_contaminant_db;
pub use build_rrna_db::build_rrna_db;
pub use build_taxonomy_db::build_taxonomy_db;
pub use chunked_preprocess::{
    build_chunked_preprocess_contract, verify_chunked_preprocess_equivalence,
};
pub use concatenate_lanes::{concatenate_lanes, LaneInput};
pub use deinterleave_reads::deinterleave_reads;
pub use demultiplex_reads::{demultiplex_reads, DemultiplexRule};
pub use deplete_host::deplete_host;
pub use deplete_reference_contaminants::deplete_reference_contaminants;
pub use deplete_rrna::deplete_rrna;
pub use detect_adapters::detect_adapters;
pub use detect_duplicates_premerge::detect_duplicates_premerge;
pub use detect_instrument_artifacts::detect_instrument_artifacts;
pub use edna_branch::{
    cluster_otus, infer_asvs, normalize_abundance, normalize_primers, remove_chimeras,
};
pub use estimate_library_complexity_prealign::estimate_library_complexity_prealign;
pub use extract_umis::extract_umis;
pub use header_inspection::{
    ensure_umi_headers, inspect_headers, log_header_warnings, HeaderInspection,
};
pub use interleave_reads::interleave_reads;
pub use layout_classification::{classify_layout, ensure_layout_is_coherent};
#[allow(unused_imports)]
pub use merge_pairs::merge_pairs;
pub use merge_suitability::{assess_merge_suitability, MergeSuitability};
pub use normalize_read_names::normalize_read_names;
pub use output_normalization::{find_first_fastq, normalize_outputs, NormalizedOutputs};
pub use prepare_adapter_bank::prepare_adapter_bank;
pub use prepare_host_reference_bundle::prepare_host_reference_bundle;
pub use prepare_primer_bank::prepare_primer_bank;
pub use provenance_snapshot::capture_provenance_snapshot;
pub use qc_manifest::materialize_qc_manifest;
pub use repair_pairs::repair_pairs;
pub use remove_duplicates::remove_duplicates;
pub use screen_taxonomy::screen_taxonomy;
pub use subsample_reads::{subsample_reads, SubsampleTarget};
pub use trim_reads::trim_reads;
pub use validate_reads::validate_reads;

/// Validate that a stage can accept the provided input kind.
///
/// # Errors
/// Returns an error if the stage contract is violated.
pub fn preflight_stage(stage_id: &str, input_kind: FastqArtifactKind) -> Result<()> {
    let Some(stage) = stage_for_id(stage_id) else {
        return Ok(());
    };
    let canonical = canonical_contract_for_stage(stage);
    if canonical.io.inputs.contains(&input_kind) {
        return Ok(());
    }
    let accepted =
        canonical.io.inputs.iter().map(|kind| format!("{kind:?}")).collect::<Vec<_>>().join(", ");
    Err(anyhow!(
        "stage {stage_id} does not accept {input_kind:?} input; accepted kinds: {accepted}"
    ))
}

fn read_fastq_text(path: &Path) -> Result<String> {
    let file = std::fs::File::open(path)?;
    let mut data = String::new();
    if path
        .extension()
        .and_then(|ext| ext.to_str())
        .is_some_and(|ext| ext.eq_ignore_ascii_case("gz"))
    {
        let mut decoder = flate2::read::MultiGzDecoder::new(file);
        decoder.read_to_string(&mut data)?;
    } else {
        let mut reader = std::io::BufReader::new(file);
        reader.read_to_string(&mut data)?;
    }
    Ok(data)
}
