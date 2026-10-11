use anyhow::Result;
use bijux_dna_bench_bam_evidence::consistency::BamActiveRowConsistencyReport;
use bijux_dna_bench_bam_proofs::retained_tools::observations::MicroSmokeObservation;
use bijux_dna_bench_bam_proofs::retained_tools::observations::ScienceThresholdsObservation;
use bijux_dna_bench_bam_proofs::retained_tools::ports::QualificationEvidence;
use std::path::Path;
pub(super) struct QualificationAdapter;
impl QualificationEvidence for QualificationAdapter {
    fn active_row_consistency(&self, root: &Path) -> Result<BamActiveRowConsistencyReport> {
        use super::super::bam_active_row_consistency::{
            render_bam_active_row_consistency, DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH,
        };
        render_bam_active_row_consistency(root, DEFAULT_BAM_ACTIVE_ROW_CONSISTENCY_PATH.into())
    }
    fn micro_smoke_summary(&self, root: &Path) -> Result<MicroSmokeObservation> {
        use crate::commands::benchmark::local_bam_micro_smoke_subset::{
            render_bam_micro_smoke_subset, DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH,
        };
        let report =
            render_bam_micro_smoke_subset(root, DEFAULT_BAM_MICRO_SMOKE_SUMMARY_PATH.into())?;
        Ok(MicroSmokeObservation {
            container_needed_count: report.container_needed_count,
            family_count: report.family_count,
            local_smoke_count: report.local_smoke_count,
            passes_behavior_test: report.passes_behavior_test,
            unavailable_count: report.unavailable_count,
        })
    }
    fn science_thresholds(&self, root: &Path) -> Result<ScienceThresholdsObservation> {
        use super::super::bam_science_thresholds_ready::{
            render_bam_science_thresholds_ready, DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH,
        };
        let report = render_bam_science_thresholds_ready(
            root,
            DEFAULT_BAM_SCIENCE_THRESHOLDS_READY_PATH.into(),
        )?;
        Ok(ScienceThresholdsObservation {
            comparable_stage_count: report.comparable_stage_count,
            governed_metric_count: report.governed_metric_count,
            missing_threshold_metric_count: report.missing_threshold_metric_count,
            missing_threshold_stage_count: report.missing_threshold_stage_count,
            stage_row_count: report.stage_row_count,
            threshold_declared_metric_count: report.threshold_declared_metric_count,
            threshold_declared_stage_count: report.threshold_declared_stage_count,
        })
    }
}
