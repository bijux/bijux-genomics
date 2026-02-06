mod apptainer;
mod behavioral;
mod datasets;
mod helpers;
mod logging;
mod runner;
mod static_qa;
mod support;

pub use helpers::{ensure_image_qa_passed, ensure_tool_qa_passed};
pub use runner::run_image_qa;
pub(crate) use support::SeqkitMetrics;
pub use support::{
    hash_file_sha256, image_qa_base_dir, image_qa_jsonl_path, image_qa_sqlite_path,
    validate_execution_outputs,
};

use std::path::PathBuf;

use bijux_domain_fastq::{
    STAGE_CORRECT, STAGE_FILTER, STAGE_MERGE, STAGE_QC_POST, STAGE_SCREEN, STAGE_STATS_NEUTRAL,
    STAGE_TRIM, STAGE_UMI, STAGE_VALIDATE_PRE,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QaStage {
    Trim,
    Validate,
    Filter,
    Merge,
    Correct,
    QcPost,
    Umi,
    Stats,
    Screen,
}

impl QaStage {
    pub(crate) fn stage_id(self) -> bijux_core::ids::StageId {
        match self {
            QaStage::Trim => STAGE_TRIM.clone(),
            QaStage::Validate => STAGE_VALIDATE_PRE.clone(),
            QaStage::Filter => STAGE_FILTER.clone(),
            QaStage::Merge => STAGE_MERGE.clone(),
            QaStage::Correct => STAGE_CORRECT.clone(),
            QaStage::QcPost => STAGE_QC_POST.clone(),
            QaStage::Umi => STAGE_UMI.clone(),
            QaStage::Stats => STAGE_STATS_NEUTRAL.clone(),
            QaStage::Screen => STAGE_SCREEN.clone(),
        }
    }

    pub(crate) fn tools(self) -> &'static [&'static str] {
        match self {
            QaStage::Trim => &[
                "fastp",
                "cutadapt",
                "bbduk",
                "adapterremoval",
                "trimmomatic",
                "trim_galore",
                "atropos",
            ],
            QaStage::Validate => &[
                "seqtk",
                "fastqc",
                "fastqvalidator",
                "fastqvalidator_official",
                "fqtools",
            ],
            QaStage::Filter => &["prinseq", "fastp", "seqkit"],
            QaStage::Merge => &["pear", "vsearch", "bbmerge", "flash2"],
            QaStage::Correct => &["rcorrector"],
            QaStage::QcPost => &["fastqc", "multiqc"],
            QaStage::Umi => &["umi_tools"],
            QaStage::Stats => &["seqkit_stats"],
            QaStage::Screen => &[
                "kraken2",
                "centrifuge",
                "metaphlan",
                "kaiju",
                "fastq_screen",
            ],
        }
    }
}

#[derive(Debug, Clone)]
pub(crate) struct QaDataset {
    pub(crate) name: String,
    pub(crate) r1: PathBuf,
    pub(crate) r2: Option<PathBuf>,
    pub(crate) r1_dir: PathBuf,
    pub(crate) input_hash_r1: String,
    pub(crate) input_hash_r2: Option<String>,
    pub(crate) input_stats_r1: SeqkitMetrics,
    pub(crate) input_stats_r2: Option<SeqkitMetrics>,
}
