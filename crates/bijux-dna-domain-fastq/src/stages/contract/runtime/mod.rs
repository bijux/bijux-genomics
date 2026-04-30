use std::io::Read;
use std::path::Path;

use anyhow::{anyhow, Result};

use super::super::canonical_contract_for_stage;
use super::catalog::stage_for_id;
use crate::types::FastqArtifactKind;

mod header_inspection;
mod layout_classification;
mod merge_suitability;
mod output_normalization;
mod provenance_snapshot;
mod qc_manifest;
mod asset_verification;
mod build_contaminant_db;
mod build_rrna_db;
mod build_taxonomy_db;
mod concatenate_lanes;
mod deinterleave_reads;
mod demultiplex_reads;
mod detect_duplicates_premerge;
mod detect_adapters;
mod detect_instrument_artifacts;
mod estimate_library_complexity_prealign;
mod fastq_io;
mod interleave_reads;
mod prepare_adapter_bank;
mod prepare_host_reference_bundle;
mod prepare_primer_bank;
mod normalize_read_names;
mod repair_pairs;
mod subsample_reads;
mod validate_reads;

pub use header_inspection::{
    ensure_umi_headers, inspect_headers, log_header_warnings, HeaderInspection,
};
pub use asset_verification::{ensure_assets_verified, verify_assets};
pub use build_contaminant_db::build_contaminant_db;
pub use build_rrna_db::build_rrna_db;
pub use build_taxonomy_db::build_taxonomy_db;
pub use concatenate_lanes::{concatenate_lanes, LaneInput};
pub use deinterleave_reads::deinterleave_reads;
pub use demultiplex_reads::{demultiplex_reads, DemultiplexRule};
pub use detect_adapters::detect_adapters;
pub use detect_duplicates_premerge::detect_duplicates_premerge;
pub use detect_instrument_artifacts::detect_instrument_artifacts;
pub use estimate_library_complexity_prealign::estimate_library_complexity_prealign;
pub use interleave_reads::interleave_reads;
pub use layout_classification::{classify_layout, ensure_layout_is_coherent};
pub use merge_suitability::{assess_merge_suitability, MergeSuitability};
pub use output_normalization::{find_first_fastq, normalize_outputs, NormalizedOutputs};
pub use normalize_read_names::normalize_read_names;
pub use prepare_adapter_bank::prepare_adapter_bank;
pub use prepare_host_reference_bundle::prepare_host_reference_bundle;
pub use prepare_primer_bank::prepare_primer_bank;
pub use provenance_snapshot::capture_provenance_snapshot;
pub use qc_manifest::materialize_qc_manifest;
pub use repair_pairs::repair_pairs;
pub use subsample_reads::{subsample_reads, SubsampleTarget};
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
