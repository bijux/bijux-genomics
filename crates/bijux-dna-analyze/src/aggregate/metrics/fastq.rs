//! Owner: bijux-dna-analyze
//! FASTQ metric schemas and versions.

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::aggregate::{BenchError, Result, StageMetricSchema};
use crate::model::JsonBlob;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDeltaMetrics {
    pub read_retention: f64,
    pub base_retention: f64,
    pub mean_q_delta: f64,
    pub gc_delta: f64,
}

impl FastqDeltaMetrics {
    /// Validate delta metrics.
    ///
    /// # Errors
    /// Returns an error if delta values are invalid.
    pub fn validate(&self) -> Result<()> {
        if !self.mean_q_delta.is_finite() {
            return Err(BenchError::Validation("mean_q_delta must be finite".to_string()));
        }
        if !self.gc_delta.is_finite() {
            return Err(BenchError::Validation("gc_delta must be finite".to_string()));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(BenchError::Validation("read_retention must be within [0, 1]".to_string()));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(BenchError::Validation("base_retention must be within [0, 1]".to_string()));
        }
        Ok(())
    }
}

impl Default for FastqDeltaMetrics {
    fn default() -> Self {
        Self { read_retention: 0.0, base_retention: 0.0, mean_q_delta: 0.0, gc_delta: 0.0 }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
    #[serde(default)]
    pub paired_mode: Option<String>,
    #[serde(default)]
    pub adapter_policy: Option<String>,
    #[serde(default)]
    pub polyx_policy: Option<String>,
    #[serde(default)]
    pub n_policy: Option<String>,
    #[serde(default)]
    pub contaminant_policy: Option<String>,
    #[serde(default)]
    pub raw_backend_report_format: Option<String>,
    #[serde(default)]
    pub adapter_preset: Option<String>,
    #[serde(default)]
    pub adapter_bank_id: Option<String>,
    #[serde(default)]
    pub adapter_bank_hash: Option<String>,
    #[serde(default)]
    pub adapter_overrides: Option<JsonBlob>,
}

impl StageMetricSchema for FastqTrimMetrics {
    const STAGE: &'static str = "fastq.trim_reads";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimPolygMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
    #[serde(default)]
    pub paired_mode: Option<String>,
    #[serde(default)]
    pub threads: Option<u32>,
    #[serde(default)]
    pub trim_polyg: Option<bool>,
    #[serde(default)]
    pub min_polyg_run: Option<u32>,
    #[serde(default)]
    pub bases_trimmed_polyg: Option<u64>,
    #[serde(default)]
    pub raw_backend_report_format: Option<String>,
    #[serde(default)]
    pub polyx_bank_id: Option<String>,
    #[serde(default)]
    pub polyx_bank_hash: Option<String>,
    #[serde(default)]
    pub polyx_preset: Option<String>,
}

impl StageMetricSchema for FastqTrimPolygMetrics {
    const STAGE: &'static str = "fastq.trim_polyg_tails";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqTrimTerminalDamageMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub damage_mode: Option<String>,
    #[serde(default)]
    pub execution_policy: Option<String>,
    #[serde(default)]
    pub requested_trim_5p_bases: Option<u32>,
    #[serde(default)]
    pub requested_trim_3p_bases: Option<u32>,
    #[serde(default)]
    pub udg_classification: Option<String>,
    #[serde(default)]
    pub ct_ga_asymmetry_pre: Option<f64>,
    #[serde(default)]
    pub ct_ga_asymmetry_post: Option<f64>,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
}

impl StageMetricSchema for FastqTrimTerminalDamageMetrics {
    const STAGE: &'static str = "fastq.trim_terminal_damage";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if let Some(value) = self.ct_ga_asymmetry_pre {
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return Err(BenchError::Validation(
                    "ct_ga_asymmetry_pre must be finite and within [-1, 1]".to_string(),
                ));
            }
        }
        if let Some(value) = self.ct_ga_asymmetry_post {
            if !value.is_finite() || !(-1.0..=1.0).contains(&value) {
                return Err(BenchError::Validation(
                    "ct_ga_asymmetry_post must be finite and within [-1, 1]".to_string(),
                ));
            }
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[must_use]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqValidateMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_total: u64,
    pub reads_valid: u64,
    pub reads_invalid: u64,
    pub mean_q: f64,
    #[serde(default)]
    pub validated_inputs: Option<u64>,
    #[serde(default)]
    pub validated_pairs: Option<u64>,
    #[serde(default)]
    pub pair_sync_checked: Option<bool>,
    #[serde(default)]
    pub pair_sync_pass: Option<bool>,
    #[serde(default)]
    pub pair_count_match: Option<bool>,
    #[serde(default)]
    pub strict_pass: Option<bool>,
    #[serde(default)]
    pub failure_class: Option<String>,
}

impl StageMetricSchema for FastqValidateMetrics {
    const STAGE: &'static str = "fastq.validate_reads";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_valid + self.reads_invalid != self.reads_total {
            return Err(BenchError::Validation(
                "reads_valid + reads_invalid must equal reads_total".to_string(),
            ));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation("mean_q must be within [0, 45]".to_string()));
        }
        if matches!(self.pair_sync_checked, Some(true)) && self.pair_sync_pass.is_none() {
            return Err(BenchError::Validation(
                "pair_sync_pass must be present when pair_sync_checked is true".to_string(),
            ));
        }
        if matches!(self.pair_sync_checked, Some(false)) && self.pair_sync_pass.is_some() {
            return Err(BenchError::Validation(
                "pair_sync_pass must be absent when pair_sync_checked is false".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDetectAdaptersMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub mean_q: f64,
    #[serde(default)]
    pub adapter_report: Option<String>,
    pub candidate_adapter_count: u64,
    #[serde(default)]
    pub detected_adapter_ids: Vec<String>,
    #[serde(default)]
    pub detection_confidence: Option<f64>,
    #[serde(default)]
    pub detection_threshold: Option<f64>,
    #[serde(default)]
    pub adapter_trimmed_fraction: Option<f64>,
}

impl StageMetricSchema for FastqDetectAdaptersMetrics {
    const STAGE: &'static str = "fastq.detect_adapters";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if self.reads_out != self.reads_in {
            return Err(BenchError::Validation("reads_out must equal reads_in".to_string()));
        }
        if self.bases_out != self.bases_in {
            return Err(BenchError::Validation("bases_out must equal bases_in".to_string()));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation("mean_q must be within [0, 45]".to_string()));
        }
        if let Some(path) = &self.adapter_report {
            if path.trim().is_empty() {
                return Err(BenchError::Validation(
                    "adapter_report must not be empty when present".to_string(),
                ));
            }
        }
        if self.detected_adapter_ids.len() as u64 != self.candidate_adapter_count {
            return Err(BenchError::Validation(
                "detected_adapter_ids length must equal candidate_adapter_count".to_string(),
            ));
        }
        if let Some(confidence) = self.detection_confidence {
            if !confidence.is_finite() || !(0.0..=1.0).contains(&confidence) {
                return Err(BenchError::Validation(
                    "detection_confidence must be within [0, 1]".to_string(),
                ));
            }
        }
        if let Some(threshold) = self.detection_threshold {
            if !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
                return Err(BenchError::Validation(
                    "detection_threshold must be within [0, 1]".to_string(),
                ));
            }
        }
        if let (Some(confidence), Some(threshold)) = (self.detection_confidence, self.detection_threshold) {
            if self.candidate_adapter_count > 0 && confidence < threshold {
                return Err(BenchError::Validation(
                    "detection_confidence must be >= detection_threshold when adapters are detected"
                        .to_string(),
                ));
            }
        }
        if let Some(fraction) = self.adapter_trimmed_fraction {
            if !fraction.is_finite() || !(0.0..=1.0).contains(&fraction) {
                return Err(BenchError::Validation(
                    "adapter_trimmed_fraction must be within [0, 1]".to_string(),
                ));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqFilterMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub reads_dropped: u64,
    #[serde(default)]
    pub reads_removed_by_n: u64,
    #[serde(default)]
    pub reads_removed_by_entropy: u64,
    #[serde(default)]
    pub reads_removed_low_complexity: u64,
    #[serde(default)]
    pub reads_removed_by_kmer: u64,
    #[serde(default)]
    pub reads_removed_contaminant_kmer: u64,
    #[serde(default)]
    pub reads_removed_by_length: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
}

impl StageMetricSchema for FastqFilterMetrics {
    const STAGE: &'static str = "fastq.filter_reads";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if self.reads_out + self.reads_dropped != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out + reads_dropped must equal reads_in".to_string(),
            ));
        }
        let removed_breakdown = self.reads_removed_by_n
            + self.reads_removed_by_entropy
            + self.reads_removed_low_complexity
            + self.reads_removed_by_kmer
            + self.reads_removed_contaminant_kmer
            + self.reads_removed_by_length;
        if removed_breakdown > self.reads_dropped {
            return Err(BenchError::Validation(
                "reads_removed_by_* must be <= reads_dropped".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqLowComplexityMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub reads_removed_low_complexity: u64,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    #[serde(default)]
    pub delta_metrics: FastqDeltaMetrics,
}

impl StageMetricSchema for FastqLowComplexityMetrics {
    const STAGE: &'static str = "fastq.filter_low_complexity";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out + self.reads_removed_low_complexity != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out + reads_removed_low_complexity must equal reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        self.delta_metrics.validate()?;
        Ok(())
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqMergeMetrics {
    #[serde(default)]
    pub reads_in: u64,
    #[serde(default)]
    pub reads_out: u64,
    #[serde(default)]
    pub bases_in: u64,
    #[serde(default)]
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: u64,
    #[serde(default)]
    pub pairs_out: u64,
    pub reads_r1: u64,
    pub reads_r2: u64,
    pub reads_merged: u64,
    pub reads_unmerged: u64,
    pub merge_rate: f64,
}

impl StageMetricSchema for FastqMergeMetrics {
    const STAGE: &'static str = "fastq.merge_pairs";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        let min_reads = self.reads_r1.min(self.reads_r2);
        if self.reads_merged + self.reads_unmerged > min_reads {
            return Err(BenchError::Validation(
                "reads_merged + reads_unmerged must be <= min(reads_r1, reads_r2)".to_string(),
            ));
        }
        if !self.merge_rate.is_finite() || !(0.0..=1.0).contains(&self.merge_rate) {
            return Err(BenchError::Validation("merge_rate must be within [0, 1]".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqCorrectMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q_before: f64,
    pub mean_q_after: f64,
    pub kmer_fix_rate: f64,
}

impl StageMetricSchema for FastqCorrectMetrics {
    const STAGE: &'static str = "fastq.correct_errors";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out != self.reads_in {
            return Err(BenchError::Validation("reads_out must equal reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        if !self.kmer_fix_rate.is_finite() || !(0.0..=1.0).contains(&self.kmer_fix_rate) {
            return Err(BenchError::Validation("kmer_fix_rate must be within [0, 1]".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqQcPostMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub mean_q: f64,
    pub contamination_rate: f64,
    #[serde(default)]
    pub aggregation_engine: Option<String>,
    #[serde(default)]
    pub aggregation_scope: Option<String>,
    #[serde(default)]
    pub governed_qc_input_count: Option<u64>,
    #[serde(default)]
    pub governed_qc_contributor_stage_ids: JsonBlob,
    #[serde(default)]
    pub governed_qc_contributor_tool_ids: JsonBlob,
    #[serde(default)]
    pub governed_qc_lineage_hash: Option<String>,
    #[serde(default)]
    pub multiqc_sample_count: Option<u64>,
    #[serde(default)]
    pub multiqc_module_count: Option<u64>,
    #[serde(default)]
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
}

impl StageMetricSchema for FastqQcPostMetrics {
    const STAGE: &'static str = "fastq.report_qc";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation("mean_q must be within [0, 45]".to_string()));
        }
        if !self.contamination_rate.is_finite() || !(0.0..=1.0).contains(&self.contamination_rate) {
            return Err(BenchError::Validation(
                "contamination_rate must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqUmiMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    #[serde(default)]
    pub bases_in: u64,
    #[serde(default)]
    pub bases_out: u64,
    #[serde(default)]
    pub pairs_in: Option<u64>,
    #[serde(default)]
    pub pairs_out: Option<u64>,
    pub reads_with_umi: u64,
}

impl StageMetricSchema for FastqUmiMetrics {
    const STAGE: &'static str = "fastq.extract_umis";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.reads_with_umi > self.reads_out {
            return Err(BenchError::Validation("reads_with_umi must be <= reads_out".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqScreenMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub contamination_rate: f64,
    #[serde(default)]
    pub classified_fraction: Option<f64>,
    #[serde(default)]
    pub unclassified_fraction: Option<f64>,
    #[serde(default)]
    pub classifier: Option<String>,
    #[serde(default)]
    pub report_format: Option<String>,
    #[serde(default)]
    pub database_catalog_id: Option<String>,
    #[serde(default)]
    pub database_artifact_id: Option<String>,
    #[serde(default)]
    pub minimum_confidence: Option<f64>,
    #[serde(default)]
    pub emit_unclassified: Option<bool>,
    #[serde(default)]
    pub contamination_summary: JsonBlob,
    #[serde(default)]
    pub top_taxa: JsonBlob,
}

impl StageMetricSchema for FastqScreenMetrics {
    const STAGE: &'static str = "fastq.screen_taxonomy";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if !self.contamination_rate.is_finite() || !(0.0..=1.0).contains(&self.contamination_rate) {
            return Err(BenchError::Validation(
                "contamination_rate must be within [0, 1]".to_string(),
            ));
        }
        for (name, value) in [
            ("classified_fraction", self.classified_fraction),
            ("unclassified_fraction", self.unclassified_fraction),
            ("minimum_confidence", self.minimum_confidence),
        ] {
            if let Some(value) = value {
                if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                    return Err(BenchError::Validation(format!("{name} must be within [0, 1]")));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDepleteHostMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub host_fraction_removed: f64,
    #[serde(default)]
    pub depletion_summary: JsonBlob,
}

impl StageMetricSchema for FastqDepleteHostMetrics {
    const STAGE: &'static str = "fastq.deplete_host";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if !self.host_fraction_removed.is_finite()
            || !(0.0..=1.0).contains(&self.host_fraction_removed)
        {
            return Err(BenchError::Validation(
                "host_fraction_removed must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDepleteReferenceContaminantsMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub contaminant_fraction_removed: f64,
    #[serde(default)]
    pub depletion_summary: JsonBlob,
}

impl StageMetricSchema for FastqDepleteReferenceContaminantsMetrics {
    const STAGE: &'static str = "fastq.deplete_reference_contaminants";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if !self.contaminant_fraction_removed.is_finite()
            || !(0.0..=1.0).contains(&self.contaminant_fraction_removed)
        {
            return Err(BenchError::Validation(
                "contaminant_fraction_removed must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDepleteRrnaMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub bases_in: u64,
    pub bases_out: u64,
    pub pairs_in: u64,
    pub pairs_out: u64,
    pub rrna_fraction_removed: f64,
    #[serde(default)]
    pub depletion_summary: JsonBlob,
}

impl StageMetricSchema for FastqDepleteRrnaMetrics {
    const STAGE: &'static str = "fastq.deplete_rrna";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation("bases_out must be <= bases_in".to_string()));
        }
        if !self.rrna_fraction_removed.is_finite()
            || !(0.0..=1.0).contains(&self.rrna_fraction_removed)
        {
            return Err(BenchError::Validation(
                "rrna_fraction_removed must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqClusterOtusMetrics {
    pub otu_count: u64,
    pub representative_count: u64,
}

impl StageMetricSchema for FastqClusterOtusMetrics {
    const STAGE: &'static str = "fastq.cluster_otus";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.representative_count > self.otu_count {
            return Err(BenchError::Validation(
                "representative_count must be <= otu_count".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LengthHistogramBin {
    pub length: u64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqStatsMetrics {
    pub reads_total: u64,
    pub bases_total: u64,
    pub mean_q: f64,
    pub gc_percent: f64,
    pub length_histogram: Vec<LengthHistogramBin>,
}

impl StageMetricSchema for FastqStatsMetrics {
    const STAGE: &'static str = "fastq.profile_reads";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation("mean_q must be within [0, 45]".to_string()));
        }
        if !self.gc_percent.is_finite() || !(0.0..=100.0).contains(&self.gc_percent) {
            return Err(BenchError::Validation("gc_percent must be within [0, 100]".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqReadLengthMetrics {
    pub read_count: u64,
    #[serde(default)]
    pub min_read_length: u64,
    pub mean_read_length: f64,
    #[serde(default)]
    pub median_read_length: f64,
    pub max_read_length: u64,
    pub distinct_lengths: u64,
}

impl StageMetricSchema for FastqReadLengthMetrics {
    const STAGE: &'static str = "fastq.profile_read_lengths";
    const VERSION: i32 = 2;

    fn validate(&self) -> Result<()> {
        if !self.mean_read_length.is_finite() || self.mean_read_length < 0.0 {
            return Err(BenchError::Validation(
                "mean_read_length must be finite and >= 0".to_string(),
            ));
        }
        if !self.median_read_length.is_finite() || self.median_read_length < 0.0 {
            return Err(BenchError::Validation(
                "median_read_length must be finite and >= 0".to_string(),
            ));
        }
        if self.read_count > 0 && self.min_read_length == 0 {
            return Err(BenchError::Validation(
                "min_read_length must be > 0 when reads are present".to_string(),
            ));
        }
        if self.read_count > 0 && self.max_read_length == 0 {
            return Err(BenchError::Validation(
                "max_read_length must be > 0 when reads are present".to_string(),
            ));
        }
        if self.read_count > 0 && self.median_read_length < self.min_read_length as f64 {
            return Err(BenchError::Validation(
                "median_read_length must be >= min_read_length when reads are present".to_string(),
            ));
        }
        if self.read_count > 0 && self.median_read_length > self.max_read_length as f64 {
            return Err(BenchError::Validation(
                "median_read_length must be <= max_read_length when reads are present".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqOverrepresentedMetrics {
    pub sequence_count: u64,
    pub flagged_sequences: u64,
    pub top_fraction: f64,
}

impl StageMetricSchema for FastqOverrepresentedMetrics {
    const STAGE: &'static str = "fastq.profile_overrepresented_sequences";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.flagged_sequences > self.sequence_count {
            return Err(BenchError::Validation(
                "flagged_sequences must be <= sequence_count".to_string(),
            ));
        }
        if !self.top_fraction.is_finite() || !(0.0..=1.0).contains(&self.top_fraction) {
            return Err(BenchError::Validation("top_fraction must be within [0, 1]".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqDuplicateMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    #[serde(alias = "duplicate_reads")]
    pub duplicates_removed: u64,
    pub dedup_rate: f64,
    #[serde(default)]
    pub tool: Option<String>,
    #[serde(default)]
    pub paired_mode: Option<String>,
    #[serde(default)]
    pub dedup_mode: Option<String>,
    #[serde(default)]
    pub keep_order: Option<bool>,
    #[serde(default)]
    pub pair_count_match: Option<bool>,
    #[serde(default)]
    pub duplicate_class_count: Option<u64>,
    #[serde(default)]
    pub duplicate_provenance_json: Option<String>,
    #[serde(default)]
    pub raw_backend_report_format: Option<String>,
}

impl StageMetricSchema for FastqDuplicateMetrics {
    const STAGE: &'static str = "fastq.remove_duplicates";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.duplicates_removed != self.reads_in.saturating_sub(self.reads_out) {
            return Err(BenchError::Validation(
                "duplicates_removed must equal reads_in - reads_out".to_string(),
            ));
        }
        if !self.dedup_rate.is_finite() || !(0.0..=1.0).contains(&self.dedup_rate) {
            return Err(BenchError::Validation("dedup_rate must be within [0, 1]".to_string()));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqChimeraMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub chimeras_removed: u64,
    pub chimera_fraction: f64,
}

impl StageMetricSchema for FastqChimeraMetrics {
    const STAGE: &'static str = "fastq.remove_chimeras";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        if self.chimeras_removed != self.reads_in.saturating_sub(self.reads_out) {
            return Err(BenchError::Validation(
                "chimeras_removed must equal reads_in - reads_out".to_string(),
            ));
        }
        if !self.chimera_fraction.is_finite() || !(0.0..=1.0).contains(&self.chimera_fraction) {
            return Err(BenchError::Validation(
                "chimera_fraction must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqNormalizePrimersMetrics {
    pub reads_in: u64,
    pub reads_out: u64,
    pub primer_trimmed_fraction: f64,
    pub orientation_forward_fraction: f64,
}

impl StageMetricSchema for FastqNormalizePrimersMetrics {
    const STAGE: &'static str = "fastq.normalize_primers";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation("reads_out must be <= reads_in".to_string()));
        }
        for (name, value) in [
            ("primer_trimmed_fraction", self.primer_trimmed_fraction),
            ("orientation_forward_fraction", self.orientation_forward_fraction),
        ] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(BenchError::Validation(format!("{name} must be within [0, 1]")));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqInferAsvsMetrics {
    pub asv_count: u64,
    pub sample_count: u64,
}

impl StageMetricSchema for FastqInferAsvsMetrics {
    const STAGE: &'static str = "fastq.infer_asvs";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqNormalizeAbundanceMetrics {
    pub table_rows: u64,
    pub sample_count: u64,
    pub zero_fraction: f64,
    pub normalization_method: String,
}

impl StageMetricSchema for FastqNormalizeAbundanceMetrics {
    const STAGE: &'static str = "fastq.normalize_abundance";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.zero_fraction.is_finite() || !(0.0..=1.0).contains(&self.zero_fraction) {
            return Err(BenchError::Validation("zero_fraction must be within [0, 1]".to_string()));
        }
        if self.normalization_method.trim().is_empty() {
            return Err(BenchError::Validation(
                "normalization_method must not be empty".to_string(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FastqIndexReferenceMetrics {
    pub reference_bytes: u64,
    pub index_bytes: u64,
    pub index_file_count: u64,
}

impl StageMetricSchema for FastqIndexReferenceMetrics {
    const STAGE: &'static str = "fastq.index_reference";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reference_bytes == 0 {
            return Err(BenchError::Validation("reference_bytes must be > 0".to_string()));
        }
        if self.index_file_count == 0 {
            return Err(BenchError::Validation("index_file_count must be > 0".to_string()));
        }
        Ok(())
    }
}
pub const FASTQ_TRIM_SCHEMA_VERSION: i32 = 2;
pub const FASTQ_VALIDATE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_DETECT_ADAPTERS_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_FILTER_SCHEMA_VERSION: i32 = 2;
pub const FASTQ_LOW_COMPLEXITY_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_MERGE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_CORRECT_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_QC_POST_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_UMI_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_SCREEN_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_STATS_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_READ_LENGTH_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_OVERREPRESENTED_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_DUPLICATE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_CHIMERA_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_NORMALIZE_PRIMERS_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_INFER_ASVS_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_NORMALIZE_ABUNDANCE_SCHEMA_VERSION: i32 = 1;
pub const FASTQ_INDEX_REFERENCE_SCHEMA_VERSION: i32 = 1;
