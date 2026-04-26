use anyhow::Result;
use bijux_dna_domain_vcf::taxonomy::{CoverageRegime, VcfDomainStage};
use std::collections::BTreeSet;

/// # Errors
/// Returns an error if requested stage ids cannot be mapped to known VCF stages.
pub fn resolve_requested_stages(
    requested_stages: &Option<Vec<String>>,
    resolved_coverage: CoverageRegime,
) -> Result<Vec<VcfDomainStage>> {
    if let Some(requested) = requested_stages {
        let mut out = Vec::new();
        let mut seen = BTreeSet::new();
        for stage_id in requested {
            let stage = VcfDomainStage::try_from(stage_id.as_str())?;
            if !seen.insert(stage.as_str()) {
                anyhow::bail!("requested_stages contains duplicate stage {}", stage.as_str());
            }
            out.push(stage);
        }
        if out.is_empty() {
            anyhow::bail!("requested_stages resolved to empty set");
        }
        return Ok(out);
    }
    Ok(match resolved_coverage {
        CoverageRegime::LowCovGl => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallGl,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::GlPropagation,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Diploid => vec![
            VcfDomainStage::PrepareReferencePanel,
            VcfDomainStage::CallDiploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Phasing,
            VcfDomainStage::Impute,
            VcfDomainStage::Postprocess,
            VcfDomainStage::PopulationStructure,
            VcfDomainStage::Roh,
            VcfDomainStage::Ibd,
            VcfDomainStage::Demography,
            VcfDomainStage::Stats,
        ],
        CoverageRegime::Pseudohaploid => vec![
            VcfDomainStage::CallPseudohaploid,
            VcfDomainStage::DamageFilter,
            VcfDomainStage::Filter,
            VcfDomainStage::Roh,
            VcfDomainStage::Stats,
        ],
    })
}
