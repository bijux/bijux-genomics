//! Enforce normalization counts, duplicate removal and sample identity.
use crate::reference_panel::model::EXPECTED_DUPLICATE_SITES_REMOVED;
use crate::reference_panel::model::EXPECTED_INPUT_VARIANT_COUNT;
use crate::reference_panel::model::EXPECTED_NORMALIZATION_STATUS;
use crate::reference_panel::model::EXPECTED_OUTPUT_VARIANT_COUNT;
use crate::reference_panel::model::EXPECTED_SAMPLE_ID;
use anyhow::bail;
use anyhow::Result;

pub(crate) fn validate_panel(
    evidence: &super::evidence::PanelEvidence,
    manifest: &super::evidence::PanelManifestEvidence,
) -> Result<bool> {
    let sample_count = evidence.sample_count;
    let input_variants = evidence.input_variants;
    let output_variants = evidence.output_variants;
    let duplicate_sites_removed = evidence.duplicate_sites_removed;
    let manifest_input_variants = manifest.input_variants;
    let manifest_output_variants = manifest.output_variants;
    let manifest_sample_count = manifest.sample_count;
    let normalization_status = evidence.normalization_status.as_str();
    let sample_ids = &evidence.sample_ids;
    let manifest_sample_ids = &manifest.sample_ids;
    if input_variants != EXPECTED_INPUT_VARIANT_COUNT {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_INPUT_VARIANT_COUNT} raw variants, found {input_variants}"
        );
    }
    if output_variants != EXPECTED_OUTPUT_VARIANT_COUNT {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_OUTPUT_VARIANT_COUNT} output variants, found {output_variants}"
        );
    }
    if duplicate_sites_removed != EXPECTED_DUPLICATE_SITES_REMOVED {
        bail!(
            "governed VCF prepare_reference_panel smoke expected {EXPECTED_DUPLICATE_SITES_REMOVED} duplicate sites removed, found {duplicate_sites_removed}"
        );
    }
    if normalization_status != EXPECTED_NORMALIZATION_STATUS {
        bail!(
            "governed VCF prepare_reference_panel smoke expected normalization status `{EXPECTED_NORMALIZATION_STATUS}`, found `{normalization_status}`"
        );
    }
    if input_variants.saturating_sub(output_variants) != duplicate_sites_removed {
        bail!(
            "governed VCF prepare_reference_panel smoke duplicate count drifted: input={input_variants} output={output_variants} removed={duplicate_sites_removed}"
        );
    }
    if manifest_input_variants != input_variants || manifest_output_variants != output_variants {
        bail!(
            "panel manifest normalization counts drifted from observed counts: manifest=({manifest_input_variants},{manifest_output_variants}) observed=({input_variants},{output_variants})"
        );
    }
    if manifest_sample_count != sample_count {
        bail!(
            "panel manifest sample count drifted from observed output: manifest={manifest_sample_count} observed={sample_count}"
        );
    }
    let sample_consistent =
        sample_ids == &vec![EXPECTED_SAMPLE_ID.to_string()] && manifest_sample_ids == sample_ids;
    if !sample_consistent {
        bail!(
            "governed VCF prepare_reference_panel smoke expected sample ids {:?}, found output {:?} and manifest {:?}",
            vec![EXPECTED_SAMPLE_ID.to_string()],
            sample_ids,
            manifest_sample_ids
        );
    }

    Ok(sample_consistent)
}
