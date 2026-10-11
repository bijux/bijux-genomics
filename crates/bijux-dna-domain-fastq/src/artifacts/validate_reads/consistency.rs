use super::{
    ValidatedReadsManifestV1, ValidationReportV1, VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
    VALIDATION_REPORT_SCHEMA_VERSION,
};
use crate::params::{validate::PairSyncPolicy, PairedMode};
use anyhow::{ensure, Result};
use bijux_dna_core::id_catalog::FASTQ_VALIDATE_READS;
impl ValidationReportV1 {
    /// Validate the report's version, identity, layout, cardinality and outcome consistency.
    /// This checks reported evidence; it does not re-read inputs or prove backend execution.
    ///
    /// # Errors
    /// Rejects contradictory or unsupported validation evidence.
    pub fn validate_consistency(&self) -> Result<()> {
        ensure!(
            self.schema_version == VALIDATION_REPORT_SCHEMA_VERSION,
            "unsupported validation report schema"
        );
        ensure!(
            self.stage == FASTQ_VALIDATE_READS && self.stage_id == FASTQ_VALIDATE_READS,
            "validation report stage identity mismatch"
        );
        ensure!(
            !self.tool_id.trim().is_empty() && self.tool_id.trim() == self.tool_id,
            "validation report tool identity is empty or padded"
        );
        ensure!(
            !self.input_r1.is_empty() && !self.validation_log_r1.is_empty(),
            "validation report requires input and log addresses"
        );
        self.validate_layout()?;
        ensure!(
            self.has_consistent_outcome(),
            "validation report failure class, strict result and exit policy disagree"
        );
        Ok(())
    }
    fn validate_layout(&self) -> Result<()> {
        if let Some(right) = &self.input_r2 {
            ensure!(
                self.pair_sync_policy != PairSyncPolicy::NotApplicable,
                "paired validation requires an applicable header-sync policy"
            );
            ensure!(
                !right.is_empty()
                    && self.validation_log_r2.as_ref().is_some_and(|path| !path.is_empty()),
                "paired validation requires right input and log addresses"
            );
            let count = self
                .validated_reads_r2
                .ok_or_else(|| anyhow::anyhow!("paired validation requires right read count"))?;
            ensure!(
                self.validated_inputs == 2
                    && self.validated_pairs == Some(self.validated_reads_r1.min(count)),
                "paired validation input or pair cardinality mismatch"
            );
            ensure!(
                self.validated_reads_r1.checked_add(count).is_some(),
                "validation read total overflows"
            );
            ensure!(
                self.pair_count_match == Some(self.validated_reads_r1 == count),
                "validation pair-count observation contradicts read counts"
            );
            let checked = self.pair_sync_policy == PairSyncPolicy::RequireHeaderSync;
            ensure!(self.pair_sync_checked == checked, "validation header-sync policy mismatch");
            ensure!(
                checked || self.pair_sync_pass.is_none(),
                "unchecked header sync cannot report a result"
            );
            ensure!(
                !self.strict_pass || !checked || self.pair_sync_pass == Some(true),
                "strict paired validation lacks successful header-sync evidence"
            );
        } else {
            ensure!(
                self.validated_inputs == 1
                    && self.validated_reads_r2.is_none()
                    && self.validation_log_r2.is_none()
                    && self.status_r2 == 0,
                "single-end validation contains right-stream evidence"
            );
            ensure!(
                self.validated_pairs.is_none()
                    && self.pair_count_match.is_none()
                    && !self.pair_sync_checked
                    && self.pair_sync_pass.is_none()
                    && self.pair_sync_policy == PairSyncPolicy::NotApplicable,
                "single-end validation contains paired evidence"
            );
        }
        Ok(())
    }
}
impl ValidatedReadsManifestV1 {
    /// Validate lineage against the typed report whose evidence it describes.
    /// Callers separately bind `validation_report` to their admitted publication address.
    ///
    /// # Errors
    /// Rejects schema, identity, layout, parameter or observation mismatches.
    pub fn validate_against(&self, report: &ValidationReportV1) -> Result<()> {
        report.validate_consistency()?;
        ensure!(
            self.schema_version == VALIDATED_READS_MANIFEST_SCHEMA_VERSION,
            "unsupported validation lineage schema"
        );
        ensure!(
            self.stage_id == report.stage_id && self.tool_id == report.tool_id,
            "validation lineage stage/tool identity mismatch"
        );
        ensure!(
            self.input_r1 == report.input_r1
                && self.input_r2 == report.input_r2
                && !self.validation_report.is_empty(),
            "validation lineage input or report address mismatch"
        );
        ensure!(
            self.validation_mode == report.validation_mode
                && self.pair_sync_policy == report.pair_sync_policy,
            "validation lineage parameter mismatch"
        );
        let paired = report.input_r2.is_some();
        let mode = if paired { PairedMode::PairedEnd } else { PairedMode::SingleEnd };
        let streams = if paired { &["reads_r1", "reads_r2"][..] } else { &["reads_r1"][..] };
        ensure!(
            self.paired_mode == mode && self.validated_stream_ids == streams,
            "validation lineage stream identity or order mismatch"
        );
        ensure!(
            self.pair_sync_checked == report.pair_sync_checked
                && self.pair_sync_pass == report.pair_sync_pass
                && self.validated_pairs == report.validated_pairs,
            "validation lineage pair observations mismatch"
        );
        Ok(())
    }
}
