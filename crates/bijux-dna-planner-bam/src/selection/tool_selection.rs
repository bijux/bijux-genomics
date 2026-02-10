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
pub fn default_tool(stage: BamStage) -> &'static str {
    match stage {
        BamStage::Align => "bwa",
        BamStage::Validate => "samtools",
        BamStage::QcPre => "samtools",
        BamStage::Filter => "samtools",
        BamStage::Markdup => "gatk",
        BamStage::Complexity => "preseq",
        BamStage::Coverage => "mosdepth",
        BamStage::Damage => "pydamage",
        BamStage::Authenticity => "authenticct",
        BamStage::Contamination => "authenticct",
        BamStage::Sex => "rxy",
        BamStage::BiasMitigation => "angsd",
        BamStage::Recalibration => "gatk",
        BamStage::Haplogroups => "yleaf",
        BamStage::Genotyping => "angsd",
        BamStage::Kinship => "king",
    }
}
