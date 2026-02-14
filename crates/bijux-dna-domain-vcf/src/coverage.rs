use std::collections::BTreeSet;

use serde::Serialize;

use crate::taxonomy::VcfDomainStage;

pub const VCF_EXECUTION_IMPLEMENTED_STAGES: &[VcfDomainStage] = &[
    VcfDomainStage::Call,
    VcfDomainStage::Filter,
    VcfDomainStage::Stats,
];

pub const VCF_EXECUTION_IMPLEMENTED_TOOLS: &[&str] = &["bcftools"];
pub const VCF_DOMAIN_TOOL_CATALOG: &[&str] = &[
    "angsd",
    "bcftools",
    "beagle",
    "eagle",
    "eigensoft",
    "germline",
    "glimpse",
    "ibdhap",
    "ibdne",
    "impute5",
    "minimac4",
    "plink",
    "plink2",
    "shapeit5",
];

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfStageCoverageRow {
    pub stage_id: String,
    pub contract_in_code: bool,
    pub execution_in_code: bool,
    pub domain_only: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfToolCoverageRow {
    pub tool_id: String,
    pub contract_in_code: bool,
    pub execution_in_code: bool,
    pub domain_only: bool,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct VcfDomainCoverageReport {
    pub schema_version: String,
    pub stages: Vec<VcfStageCoverageRow>,
    pub tools: Vec<VcfToolCoverageRow>,
}

#[must_use]
pub fn domain_coverage_report() -> VcfDomainCoverageReport {
    let contract_stage_ids = VcfDomainStage::all()
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();
    let execution_stage_ids = VCF_EXECUTION_IMPLEMENTED_STAGES
        .iter()
        .map(|stage| stage.as_str().to_string())
        .collect::<BTreeSet<_>>();

    let mut stages = contract_stage_ids
        .iter()
        .map(|stage_id| {
            let contract_in_code = true;
            let execution_in_code = execution_stage_ids.contains(stage_id);
            VcfStageCoverageRow {
                stage_id: stage_id.clone(),
                contract_in_code,
                execution_in_code,
                domain_only: contract_in_code && !execution_in_code,
            }
        })
        .collect::<Vec<_>>();
    stages.sort_by(|a, b| a.stage_id.cmp(&b.stage_id));

    let contract_tools = VCF_DOMAIN_TOOL_CATALOG
        .iter()
        .map(|tool| (*tool).to_string())
        .collect::<BTreeSet<_>>();
    let execution_tools = VCF_EXECUTION_IMPLEMENTED_TOOLS
        .iter()
        .map(|tool| (*tool).to_string())
        .collect::<BTreeSet<_>>();
    let all_tools = contract_tools
        .union(&execution_tools)
        .cloned()
        .collect::<BTreeSet<_>>();

    let mut tools = all_tools
        .iter()
        .map(|tool_id| {
            let contract_in_code = contract_tools.contains(tool_id);
            let execution_in_code = execution_tools.contains(tool_id);
            VcfToolCoverageRow {
                tool_id: tool_id.clone(),
                contract_in_code,
                execution_in_code,
                domain_only: contract_in_code && !execution_in_code,
            }
        })
        .collect::<Vec<_>>();
    tools.sort_by(|a, b| a.tool_id.cmp(&b.tool_id));

    VcfDomainCoverageReport {
        schema_version: "bijux.vcf.domain_coverage.v1".to_string(),
        stages,
        tools,
    }
}
