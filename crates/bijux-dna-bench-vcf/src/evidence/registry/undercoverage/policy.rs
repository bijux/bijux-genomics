use super::super::admission::RegistryToolRecord;
use super::model::VcfUndercoveredStageRow;
use anyhow::{anyhow, bail, Result};
use std::collections::BTreeMap;
pub(super) fn classify_undercoverage_decision<'a>(
    stage_id: &str,
    missing_tools: &[String],
    registry_by_tool: &'a BTreeMap<String, RegistryToolRecord>,
) -> Result<&'a str> {
    for tool_id in missing_tools {
        let record = registry_by_tool.get(tool_id.as_str()).ok_or_else(|| {
            anyhow!("VCF undercovered-stage detector is missing registry data for `{tool_id}`")
        })?;
        if record.production_stage_ids.contains(stage_id) {
            return Ok("limit_to_specialized_tool");
        }
    }
    Ok("future_not_benchmark_ready")
}
pub(super) fn vcf_tool_class_label(tool_id: &str) -> Result<&'static str> {
    let label = match tool_id {
        "angsd" => "genotype_likelihood_calling",
        "bcftools" => "variant_processing",
        "beagle" | "eagle" | "shapeit" | "shapeit5" => "phasing",
        "eigensoft" => "population_structure",
        "germline" | "ibdhap" | "ibdseq" => "relatedness",
        "glimpse" | "impute5" | "minimac4" => "imputation",
        "ibdne" => "demography",
        "plink" | "plink2" => "cohort_analysis",
        other => {
            return Err(anyhow!(
                "VCF undercovered-stage detector encountered unknown tool `{other}`"
            ))
        }
    };
    Ok(label)
}
pub(super) fn ensure_vcf_undercovered_stage_contract(
    rows: &[VcfUndercoveredStageRow],
) -> Result<()> {
    if !rows.is_empty() {
        bail!(
            "every multi-tool VCF stage must serve its complete supported tool set; undercovered stages: {:?}",
            rows.iter().map(|row| row.stage_id.as_str()).collect::<Vec<_>>()
        );
    }
    Ok(())
}
