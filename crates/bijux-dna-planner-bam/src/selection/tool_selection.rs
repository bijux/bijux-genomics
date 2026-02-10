use bijux_dna_core::ids::{id_catalog, ToolId};
use bijux_dna_domain_bam::BamStage;
use std::collections::BTreeSet;

#[must_use]
pub fn allowed_tools_for_stage(stage: BamStage) -> Vec<String> {
    canonical_tools_for_stage(stage)
}

#[must_use]
#[allow(dead_code)]
pub fn default_tool_for_stage(stage: BamStage) -> String {
    default_tool(stage).to_string()
}

#[must_use]
pub fn canonical_tools_for_stage(stage: BamStage) -> Vec<String> {
    let adapter = stage.as_str();
    let mut tools = crate::selection::tool_registry::tool_registry()
        .into_values()
        .filter(|entry| entry.adapter_id == adapter)
        .map(|entry| entry.tool_id.to_string())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    tools.sort();
    tools
}

#[must_use]
pub fn default_tool(stage: BamStage) -> ToolId {
    match stage {
        BamStage::Align => ToolId::from_static(id_catalog::TOOL_BWA),
        BamStage::Validate => ToolId::from_static(id_catalog::TOOL_SAMTOOLS),
        BamStage::QcPre => ToolId::from_static(id_catalog::TOOL_SAMTOOLS),
        BamStage::Filter => ToolId::from_static(id_catalog::TOOL_SAMTOOLS),
        BamStage::Markdup => ToolId::from_static(id_catalog::TOOL_GATK),
        BamStage::Complexity => ToolId::from_static(id_catalog::TOOL_PRESEQ),
        BamStage::Coverage => ToolId::from_static(id_catalog::TOOL_MOSDEPTH),
        BamStage::Damage => ToolId::from_static(id_catalog::TOOL_PYDAMAGE),
        BamStage::Authenticity => ToolId::from_static(id_catalog::TOOL_AUTHENTICCT),
        BamStage::Contamination => ToolId::from_static(id_catalog::TOOL_AUTHENTICCT),
        BamStage::Sex => ToolId::from_static(id_catalog::TOOL_RXY),
        BamStage::BiasMitigation => ToolId::from_static(id_catalog::TOOL_ANGSD),
        BamStage::Recalibration => ToolId::from_static(id_catalog::TOOL_GATK),
        BamStage::Haplogroups => ToolId::from_static(id_catalog::TOOL_YLEAF),
        BamStage::Genotyping => ToolId::from_static(id_catalog::TOOL_ANGSD),
        BamStage::Kinship => ToolId::from_static(id_catalog::TOOL_KING),
    }
}
