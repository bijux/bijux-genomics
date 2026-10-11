//! Preserve genotype, phase, sample and backend observations before publication.
use crate::panel_contracts::GovernedVcfPanelWorkflowSmokeContract;
use crate::phasing::model::PhaseSummary;
use crate::phasing::model::EXPECTED_INPUT_GENOTYPES;
use crate::phasing::model::EXPECTED_PHASED_GENOTYPES;
use crate::phasing::model::EXPECTED_PHASE_SET_COUNT;
use crate::phasing::model::EXPECTED_SAMPLE_IDS;
use crate::phasing::model::EXPECTED_UNPHASED_GENOTYPES;
use crate::phasing::observations::reports::read_json;
use crate::phasing::observations::variants::summarize_phasing_genotypes;
use anyhow::anyhow;
use anyhow::bail;
use anyhow::Context;
use anyhow::Result;
use bijux_dna_stages_vcf::pipeline::assert_bgzip_tabix_artifacts;
use bijux_dna_stages_vcf::vcf_io::vcf_validate_input;
use bijux_dna_stages_vcf::vcf_io::VcfFieldRequirement;
use bijux_dna_stages_vcf::vcf_io::VcfValidationSummary;
use std::path::Path;

pub(crate) struct PhasingEvidence {
    pub(crate) input_summary: PhaseSummary,
    pub(crate) output_summary: PhaseSummary,
    pub(crate) validation: VcfValidationSummary,
    pub(crate) sample_count: u64,
}

pub(crate) fn observe_stage(
    artifacts: &super::super::execution::artifacts::PhasingArtifacts,
    input_vcf_path: &Path,
    contract: &GovernedVcfPanelWorkflowSmokeContract,
) -> Result<PhasingEvidence> {
    let super::super::execution::artifacts::PhasingArtifacts {
        output_vcf: output_vcf_path,
        output_tbi: output_tbi_path,
        phasing_qc: phasing_qc_path,
        phasing_manifest: phasing_manifest_path,
        ..
    } = artifacts;
    assert_bgzip_tabix_artifacts(output_vcf_path, output_tbi_path)?;
    let validation = vcf_validate_input(
        output_vcf_path,
        VcfFieldRequirement { require_gt: true, require_gl: false },
    )
    .with_context(|| format!("validate {}", output_vcf_path.display()))?;
    let input_summary = summarize_phasing_genotypes(input_vcf_path)?;
    let output_summary = summarize_phasing_genotypes(output_vcf_path)?;

    if input_summary.genotype_count != EXPECTED_INPUT_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} input genotypes, found {}",
            EXPECTED_INPUT_GENOTYPES,
            input_summary.genotype_count
        );
    }
    if output_summary.phased_genotypes != EXPECTED_PHASED_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} phased genotypes, found {}",
            EXPECTED_PHASED_GENOTYPES,
            output_summary.phased_genotypes
        );
    }
    if output_summary.unphased_genotypes != EXPECTED_UNPHASED_GENOTYPES {
        bail!(
            "governed VCF phasing smoke expected {} unphased genotypes after phasing, found {}",
            EXPECTED_UNPHASED_GENOTYPES,
            output_summary.unphased_genotypes
        );
    }
    if output_summary.phase_set_count != EXPECTED_PHASE_SET_COUNT {
        bail!(
            "governed VCF phasing smoke expected {} phase sets, found {}",
            EXPECTED_PHASE_SET_COUNT,
            output_summary.phase_set_count
        );
    }
    if output_summary.sample_ids
        != EXPECTED_SAMPLE_IDS.iter().map(|value| (*value).to_string()).collect::<Vec<_>>()
    {
        bail!(
            "governed VCF phasing smoke expected sample ids {:?}, found {:?}",
            EXPECTED_SAMPLE_IDS,
            output_summary.sample_ids
        );
    }

    let phasing_qc = read_json(phasing_qc_path)?;
    if phasing_qc.get("backend").and_then(serde_json::Value::as_str) != Some(contract.tool.as_str())
    {
        bail!("phasing QC report drifted away from requested backend `{}`", contract.tool);
    }
    let phasing_manifest = read_json(phasing_manifest_path)?;
    if phasing_manifest.get("backend").and_then(serde_json::Value::as_str)
        != Some(contract.tool.as_str())
    {
        bail!("phasing manifest drifted away from requested backend `{}`", contract.tool);
    }
    if phasing_manifest.pointer("/map/map_id").and_then(serde_json::Value::as_str)
        != Some(contract.map.as_str())
    {
        bail!("phasing manifest map identity drifted from governed map contract");
    }

    let sample_count = u64::try_from(output_summary.sample_ids.len())
        .map_err(|_| anyhow!("sample count overflow"))?;
    Ok(PhasingEvidence { input_summary, output_summary, validation, sample_count })
}
