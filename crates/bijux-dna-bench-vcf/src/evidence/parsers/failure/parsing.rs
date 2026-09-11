use super::cases::VcfParserFailureCase;
use anyhow::{anyhow, Result};
use bijux_dna_domain_vcf::{
    parse_bcftools_stage_metrics, parse_eigensoft_stage_metrics, parse_imputation_stage_metrics,
    parse_phasing_stage_metrics, parse_segment_stage_metrics, VcfDomainStage,
};
use std::path::Path;
pub(super) fn parse_case(case: &VcfParserFailureCase, probe_root: &Path) -> Result<()> {
    match (case.tool_id, case.stage) {
        ("bcftools", VcfDomainStage::Call) => {
            parse_bcftools_stage_metrics(VcfDomainStage::Call, probe_root)?;
        }
        ("bcftools", VcfDomainStage::Postprocess) => {
            parse_bcftools_stage_metrics(VcfDomainStage::Postprocess, probe_root)?;
        }
        ("shapeit5", VcfDomainStage::Phasing) => {
            parse_phasing_stage_metrics("shapeit5", probe_root)?;
        }
        ("eigensoft", VcfDomainStage::Pca) => {
            parse_eigensoft_stage_metrics(VcfDomainStage::Pca, probe_root)?;
        }
        ("beagle", VcfDomainStage::Impute) => {
            parse_imputation_stage_metrics("beagle", VcfDomainStage::Impute, probe_root)?;
        }
        ("germline", VcfDomainStage::Ibd) => {
            parse_segment_stage_metrics("germline", VcfDomainStage::Ibd, probe_root)?;
        }
        _ => {
            return Err(anyhow!(
                "unsupported VCF parser failure case `{}` / `{}`",
                case.tool_id,
                case.stage.as_str()
            ));
        }
    }
    Ok(())
}
