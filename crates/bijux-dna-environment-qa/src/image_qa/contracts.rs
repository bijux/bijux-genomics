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
    pub(crate) fn core_stages() -> Vec<Self> {
        vec![
            QaStage::Trim,
            QaStage::Validate,
            QaStage::Filter,
            QaStage::Merge,
            QaStage::Correct,
            QaStage::ReportQc,
            QaStage::Umi,
            QaStage::Stats,
        ]
    }

    pub(crate) fn enabled_stages(screen_db_available: bool) -> Vec<Self> {
        let mut stages = Self::core_stages();
        if screen_db_available {
            stages.push(QaStage::Screen);
        }
        stages
    }

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
    use std::collections::BTreeSet;
    use std::path::{Path, PathBuf};

    use bijux_dna_domain_fastq::all_stage_execution_support;

    use super::QaStage;

    fn workspace_root() -> PathBuf {
        let Some(root) = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .map(Path::to_path_buf)
        else {
            panic!("workspace root");
        };
        root
    }

    fn qa_coverage_blocker_stage_ids() -> BTreeSet<String> {
        let path = workspace_root().join("science/docs/upstream/fastq/QA_COVERAGE_BLOCKERS.tsv");
        let raw = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        raw.lines()
            .enumerate()
            .filter(|(index, line)| *index > 0 && !line.trim().is_empty())
            .map(|(index, line)| {
                let stage_id = line.split('\t').next().unwrap_or_default().trim();
                assert!(!stage_id.is_empty(), "missing stage_id column at line {}", index + 1);
                stage_id.to_string()
            })
            .collect()
    }

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

    #[test]
    fn qa_stage_selection_is_centralized_and_keeps_screen_database_gated() {
        assert_eq!(
            QaStage::enabled_stages(false),
            vec![
                QaStage::Trim,
                QaStage::Validate,
                QaStage::Filter,
                QaStage::Merge,
                QaStage::Correct,
                QaStage::ReportQc,
                QaStage::Umi,
                QaStage::Stats,
            ]
        );

        let with_screen = QaStage::enabled_stages(true);
        assert_eq!(with_screen.last(), Some(&QaStage::Screen));
        assert_eq!(with_screen.len(), QaStage::enabled_stages(false).len() + 1);
    }

    #[test]
    fn qa_coverage_blockers_match_uncovered_execution_support_stages() {
        let admitted = all_stage_execution_support()
            .into_iter()
            .map(|support| support.stage_id.as_str().to_string())
            .collect::<BTreeSet<_>>();
        let covered = QaStage::enabled_stages(true)
            .into_iter()
            .map(|stage| stage.stage_id().as_str().to_string())
            .collect::<BTreeSet<_>>();
        let missing = admitted.difference(&covered).cloned().collect::<BTreeSet<_>>();

        assert_eq!(
            qa_coverage_blocker_stage_ids(),
            missing,
            "science/docs/upstream/fastq/QA_COVERAGE_BLOCKERS.tsv must match admitted execution-support stages without environment-QA coverage"
        );
    }
}
