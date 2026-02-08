use bijux_dna_core::ids::StageId;
use bijux_dna_domain_fastq::{
    STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS,
    STAGE_QC_POST, STAGE_SCREEN, STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};
use bijux_dna_pipelines::STAGE_CORE_PREPARE_REFERENCE;

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<String> {
    canonical_tools_for_stage(stage_id)
        .iter()
        .map(|tool| (*tool).to_string())
        .collect()
}

#[must_use]
pub fn canonical_tools_for_stage(stage_id: &StageId) -> &'static [&'static str] {
    if stage_id == &STAGE_PREPROCESS {
        &["planner"]
    } else if stage_id.as_str() == STAGE_CORE_PREPARE_REFERENCE {
        &["samtools"]
    } else if stage_id == &STAGE_VALIDATE_PRE {
        &[
            "seqtk",
            "fastqc",
            "fastqvalidator",
            "fastqvalidator_official",
            "fqtools",
        ]
    } else if stage_id == &STAGE_DETECT_ADAPTERS {
        &["fastqc"]
    } else if stage_id == &STAGE_TRIM {
        &[
            "fastp",
            "cutadapt",
            "atropos",
            "bbduk",
            "adapterremoval",
            "trimmomatic",
            "trim_galore",
            "seqpurge",
            "prinseq",
            "seqkit",
        ]
    } else if stage_id == &STAGE_FILTER {
        &["prinseq", "fastp", "seqkit", "bbduk"]
    } else if stage_id == &STAGE_STATS_NEUTRAL {
        &["seqkit_stats"]
    } else if stage_id == &STAGE_QC_POST {
        &["fastqc", "multiqc"]
    } else if stage_id == &STAGE_MERGE {
        &["pear", "vsearch", "bbmerge", "flash2"]
    } else if stage_id == &STAGE_CORRECT {
        &["rcorrector", "spades", "bayeshammer", "lighter", "musket"]
    } else if stage_id == &STAGE_UMI {
        &["umi_tools"]
    } else if stage_id == &STAGE_SCREEN {
        &[
            "kraken2",
            "centrifuge",
            "metaphlan",
            "kaiju",
            "fastq_screen",
        ]
    } else {
        &[]
    }
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<String> {
    if stage_id == &STAGE_PREPROCESS {
        Some("planner".to_string())
    } else if stage_id.as_str() == STAGE_CORE_PREPARE_REFERENCE {
        Some("samtools".to_string())
    } else if stage_id == &STAGE_VALIDATE_PRE {
        Some("fastqvalidator_official".to_string())
    } else if stage_id == &STAGE_DETECT_ADAPTERS {
        Some("fastqc".to_string())
    } else if stage_id == &STAGE_TRIM {
        Some("fastp".to_string())
    } else if stage_id == &STAGE_FILTER {
        Some("seqkit".to_string())
    } else if stage_id == &STAGE_STATS_NEUTRAL {
        Some("seqkit_stats".to_string())
    } else if stage_id == &STAGE_QC_POST {
        Some("multiqc".to_string())
    } else if stage_id == &STAGE_MERGE {
        Some("vsearch".to_string())
    } else if stage_id == &STAGE_CORRECT {
        Some("rcorrector".to_string())
    } else if stage_id == &STAGE_UMI {
        Some("umi_tools".to_string())
    } else if stage_id == &STAGE_SCREEN {
        Some("kraken2".to_string())
    } else {
        None
    }
}
