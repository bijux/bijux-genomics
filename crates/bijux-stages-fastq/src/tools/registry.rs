use bijux_core::ids::StageId;
use bijux_domain_fastq::stage_registry::{
    STAGE_CORRECT, STAGE_DETECT_ADAPTERS, STAGE_FILTER, STAGE_MERGE, STAGE_PREPROCESS,
    STAGE_QC_POST, STAGE_SCREEN, STAGE_STATS_NEUTRAL, STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

#[must_use]
pub fn allowed_tools_for_stage(stage_id: &StageId) -> Vec<String> {
    let tools: &[&str] = if stage_id == &STAGE_PREPROCESS {
        &["planner"]
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
    };
    tools.iter().map(|tool| (*tool).to_string()).collect()
}

#[must_use]
pub fn default_tool_for_stage(stage_id: &StageId) -> Option<String> {
    if stage_id == &STAGE_PREPROCESS {
        return Some("planner".to_string());
    }
    None
}
