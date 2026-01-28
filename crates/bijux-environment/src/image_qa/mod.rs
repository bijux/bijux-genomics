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

use std::path::PathBuf;

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
    pub(crate) fn stage_id(self) -> &'static str {
        match self {
            QaStage::Trim => "fastq.trim",
            QaStage::Validate => "fastq.validate_pre",
            QaStage::Filter => "fastq.filter",
            QaStage::Merge => "fastq.merge",
            QaStage::Correct => "fastq.correct",
            QaStage::QcPost => "fastq.qc_post",
            QaStage::Umi => "fastq.umi",
            QaStage::Stats => "fastq.stats_neutral",
            QaStage::Screen => "fastq.screen",
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
