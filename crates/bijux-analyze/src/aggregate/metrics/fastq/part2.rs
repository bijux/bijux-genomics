//! Owner: bijux-analyze
//! FASTQ metrics (part 2).

use serde::{Deserialize, Serialize};
use tracing::warn;

use crate::aggregate::{BenchError, Result, StageMetricSchema};
use crate::model::JsonBlob;

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
    const STAGE: &'static str = "fastq.merge";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        let min_reads = self.reads_r1.min(self.reads_r2);
        if self.reads_merged + self.reads_unmerged > min_reads {
            return Err(BenchError::Validation(
                "reads_merged + reads_unmerged must be <= min(reads_r1, reads_r2)".to_string(),
            ));
        }
        if !self.merge_rate.is_finite() || !(0.0..=1.0).contains(&self.merge_rate) {
            return Err(BenchError::Validation(
                "merge_rate must be within [0, 1]".to_string(),
            ));
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
    const STAGE: &'static str = "fastq.correct";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out != self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must equal reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
        }
        if self.mean_q_after < self.mean_q_before {
            warn!(
                mean_q_before = self.mean_q_before,
                mean_q_after = self.mean_q_after,
                "mean_q_after is lower than mean_q_before"
            );
        }
        if !self.kmer_fix_rate.is_finite() || !(0.0..=1.0).contains(&self.kmer_fix_rate) {
            return Err(BenchError::Validation(
                "kmer_fix_rate must be within [0, 1]".to_string(),
            ));
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
    pub raw_fastqc_dir: Option<String>,
    #[serde(default)]
    pub trimmed_fastqc_dir: Option<String>,
    #[serde(default)]
    pub multiqc_report: Option<String>,
    #[serde(default)]
    pub multiqc_data: Option<String>,
}

impl StageMetricSchema for FastqQcPostMetrics {
    const STAGE: &'static str = "fastq.qc_post";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
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
    pub dedup_rate: f64,
}

impl StageMetricSchema for FastqUmiMetrics {
    const STAGE: &'static str = "fastq.umi";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if !self.dedup_rate.is_finite() || !(0.0..=1.0).contains(&self.dedup_rate) {
            return Err(BenchError::Validation(
                "dedup_rate must be within [0, 1]".to_string(),
            ));
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
    pub contamination_summary: JsonBlob,
}

impl StageMetricSchema for FastqScreenMetrics {
    const STAGE: &'static str = "fastq.screen";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_out > self.reads_in {
            return Err(BenchError::Validation(
                "reads_out must be <= reads_in".to_string(),
            ));
        }
        if self.bases_out > self.bases_in {
            return Err(BenchError::Validation(
                "bases_out must be <= bases_in".to_string(),
            ));
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
    const STAGE: &'static str = "fastq.stats_neutral";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
        }
        if !self.gc_percent.is_finite() || !(0.0..=100.0).contains(&self.gc_percent) {
            return Err(BenchError::Validation(
                "gc_percent must be within [0, 100]".to_string(),
            ));
        }
        Ok(())
    }
}
