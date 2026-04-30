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

pub use header_inspection::{
    ensure_umi_headers, inspect_headers, log_header_warnings, HeaderInspection,
};
pub use layout_classification::{classify_layout, ensure_layout_is_coherent};
pub use merge_suitability::{assess_merge_suitability, MergeSuitability};
pub use output_normalization::{find_first_fastq, normalize_outputs, NormalizedOutputs};

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
