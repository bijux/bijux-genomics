//! Acquire observed panel records and native normalization evidence.
use crate::reference_panel::observations::reports::read_json;
use crate::reference_panel::observations::variants::parse_output_sample_ids;
use crate::reference_panel::observations::variants::parse_record_count;
use anyhow::anyhow;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::assert_bgzip_tabix_artifacts;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;
use std::path::Path;

pub(crate) struct PanelEvidence {
    pub(crate) sample_consistent: bool,
    pub(crate) validation: VcfValidationSummary,
    pub(crate) sample_ids: Vec<String>,
    pub(crate) sample_count: u64,
    pub(crate) input_variants: u64,
    pub(crate) output_variants: u64,
    pub(crate) duplicate_sites_removed: u64,
    pub(crate) normalization_status: String,
}
pub(crate) struct PanelManifestEvidence {
    pub(crate) input_variants: u64,
    pub(crate) output_variants: u64,
    pub(crate) sample_count: u64,
    pub(crate) sample_ids: Vec<String>,
}
pub(crate) fn observe_panel(
    panel_vcf_path: &Path,
    panel_tbi_path: &Path,
    panel_manifest_path: &Path,
    raw_panel_path: &Path,
) -> Result<(PanelEvidence, PanelManifestEvidence)> {
    assert_bgzip_tabix_artifacts(panel_vcf_path, panel_tbi_path)?;
    let validation = vcf_validate_input(
        panel_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", panel_vcf_path.display()))?;
    let sample_ids = parse_output_sample_ids(panel_vcf_path)?;
    let sample_count =
        u64::try_from(sample_ids.len()).map_err(|_| anyhow!("sample count overflow"))?;
    let input_variants = parse_record_count(raw_panel_path)?;
    let output_variants = parse_record_count(panel_vcf_path)?;
    let panel_manifest = read_json(panel_manifest_path)?;
    let duplicate_sites_removed = panel_manifest
        .pointer("/normalization/duplicate_sites_removed")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| {
            anyhow!("panel manifest is missing normalization.duplicate_sites_removed")
        })?;
    let normalization_status = panel_manifest
        .pointer("/normalization/status")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.status"))?
        .to_string();
    let manifest_input_variants = panel_manifest
        .pointer("/normalization/input_variant_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.input_variant_count"))?;
    let manifest_output_variants = panel_manifest
        .pointer("/normalization/output_variant_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.output_variant_count"))?;
    let manifest_sample_count = panel_manifest
        .pointer("/normalization/sample_count")
        .and_then(serde_json::Value::as_u64)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.sample_count"))?;
    let manifest_sample_ids = panel_manifest
        .pointer("/normalization/sample_ids")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| anyhow!("panel manifest is missing normalization.sample_ids"))?
        .iter()
        .filter_map(|value| value.as_str().map(str::to_string))
        .collect::<Vec<_>>();

    Ok((
        PanelEvidence {
            sample_consistent: false,
            validation,
            sample_ids,
            sample_count,
            input_variants,
            output_variants,
            duplicate_sites_removed,
            normalization_status,
        },
        PanelManifestEvidence {
            input_variants: manifest_input_variants,
            output_variants: manifest_output_variants,
            sample_count: manifest_sample_count,
            sample_ids: manifest_sample_ids,
        },
    ))
}
