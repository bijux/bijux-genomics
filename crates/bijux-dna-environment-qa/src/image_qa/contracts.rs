use std::path::PathBuf;

use bijux_dna_domain_fastq::{
    admitted_execution_tools_for_stage, STAGE_CORRECT_ERRORS, STAGE_EXTRACT_UMIS,
    STAGE_FILTER_READS, STAGE_MERGE_PAIRS, STAGE_PROFILE_READS, STAGE_REPORT_QC,
    STAGE_SCREEN_TAXONOMY, STAGE_TRIM_READS, STAGE_VALIDATE_READS,
};

use super::support::SeqkitMetrics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum QaStage {
    Trim,
    Validate,
    Filter,
    Merge,
    Correct,
    ReportQc,
    Umi,
    Stats,
    Screen,
}

impl QaStage {
    pub(crate) fn stage_id(self) -> bijux_dna_core::ids::StageId {
        match self {
            QaStage::Trim => STAGE_TRIM_READS.clone(),
            QaStage::Validate => STAGE_VALIDATE_READS.clone(),
            QaStage::Filter => STAGE_FILTER_READS.clone(),
            QaStage::Merge => STAGE_MERGE_PAIRS.clone(),
            QaStage::Correct => STAGE_CORRECT_ERRORS.clone(),
            QaStage::ReportQc => STAGE_REPORT_QC.clone(),
            QaStage::Umi => STAGE_EXTRACT_UMIS.clone(),
            QaStage::Stats => STAGE_PROFILE_READS.clone(),
            QaStage::Screen => STAGE_SCREEN_TAXONOMY.clone(),
        }
    }

    pub(crate) fn tools(self) -> Vec<String> {
        admitted_execution_tools_for_stage(&self.stage_id())
            .into_iter()
            .map(|tool| tool.as_str().to_string())
            .collect()
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

#[cfg(test)]
mod tests {
    use super::QaStage;

    #[test]
    fn qa_tool_rosters_come_from_fastq_execution_support() {
        let validate_tools = QaStage::Validate.tools();
        assert_eq!(
            validate_tools.iter().filter(|tool| tool.as_str() == "fastqvalidator").count(),
            1
        );
        assert!(validate_tools.iter().any(|tool| tool == "fastq_scan"));

        let screen_tools = QaStage::Screen.tools();
        assert_eq!(screen_tools, vec!["kraken2", "krakenuniq", "centrifuge", "kaiju"]);
        assert!(!screen_tools.iter().any(|tool| tool == "metaphlan" || tool == "fastq_screen"));
    }
}
