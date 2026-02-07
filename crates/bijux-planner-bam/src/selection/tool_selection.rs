use bijux_domain_bam::BamStage;

#[must_use]
pub fn allowed_tools_for_stage(stage: BamStage) -> Vec<String> {
    canonical_tools_for_stage(stage)
        .iter()
        .map(|tool| (*tool).to_string())
        .collect()
}

#[must_use]
#[allow(dead_code)]
pub fn default_tool_for_stage(stage: BamStage) -> String {
    default_tool(stage).to_string()
}

#[must_use]
pub fn canonical_tools_for_stage(stage: BamStage) -> &'static [&'static str] {
    match stage {
        BamStage::Align => &["bwa", "bowtie2"],
        BamStage::Validate => &["samtools"],
        BamStage::QcPre => &["samtools"],
        BamStage::Filter => &["samtools"],
        BamStage::Markdup => &["gatk", "samtools"],
        BamStage::Complexity => &["preseq"],
        BamStage::Coverage => &["mosdepth", "samtools"],
        BamStage::Damage => &["pydamage", "mapdamage2"],
        BamStage::Authenticity => &["authenticity"],
        BamStage::Contamination => &["authenticct"],
        BamStage::Sex => &["rxy"],
        BamStage::BiasMitigation => &["angsd"],
        BamStage::Recalibration => &["gatk"],
        BamStage::Haplogroups => &["yleaf"],
        BamStage::Genotyping => &["angsd"],
        BamStage::Kinship => &["king"],
    }
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
        BamStage::Authenticity => "authenticity",
        BamStage::Contamination => "authenticct",
        BamStage::Sex => "rxy",
        BamStage::BiasMitigation => "angsd",
        BamStage::Recalibration => "gatk",
        BamStage::Haplogroups => "yleaf",
        BamStage::Genotyping => "angsd",
        BamStage::Kinship => "king",
    }
}
