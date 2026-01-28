mod datasets;
mod helpers;
mod logging;
mod runner;
mod stages;

pub use helpers::ensure_image_qa_passed;
pub use runner::run_image_qa;

use crate::observer::SeqkitMetrics;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QaStage {
    Trim,
    Validate,
    Filter,
    Merge,
    Correct,
    Qc2,
    Umi,
    Stats,
    Screen,
}

impl QaStage {
    pub(crate) fn stage_id(self) -> &'static str {
        match self {
            QaStage::Trim => "fastq.trim",
            QaStage::Validate => "fastq.validate",
            QaStage::Filter => "fastq.filter",
            QaStage::Merge => "fastq.merge",
            QaStage::Correct => "fastq.correct",
            QaStage::Qc2 => "fastq.qc2",
            QaStage::Umi => "fastq.umi",
            QaStage::Stats => "fastq.stats",
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
            QaStage::Qc2 => &["fastqc", "multiqc"],
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
