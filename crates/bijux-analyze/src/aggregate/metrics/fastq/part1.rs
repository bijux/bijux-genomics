//! Owner: bijux-analyze
//! FASTQ metrics (part 1).

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
            return Err(BenchError::Validation(
                "mean_q_delta must be finite".to_string(),
            ));
        }
        if !self.gc_delta.is_finite() {
            return Err(BenchError::Validation(
                "gc_delta must be finite".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(BenchError::Validation(
                "read_retention must be within [0, 1]".to_string(),
            ));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(BenchError::Validation(
                "base_retention must be within [0, 1]".to_string(),
            ));
        }
        Ok(())
    }
}

impl Default for FastqDeltaMetrics {
    fn default() -> Self {
        Self {
            read_retention: 0.0,
            base_retention: 0.0,
            mean_q_delta: 0.0,
            gc_delta: 0.0,
        }
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
    pub adapter_preset: Option<String>,
    #[serde(default)]
    pub adapter_bank_id: Option<String>,
    #[serde(default)]
    pub adapter_bank_hash: Option<String>,
    #[serde(default)]
    pub adapter_overrides: Option<JsonBlob>,
}

impl StageMetricSchema for FastqTrimMetrics {
    const STAGE: &'static str = "fastq.trim";
    const VERSION: i32 = 2;

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
}

impl StageMetricSchema for FastqValidateMetrics {
    const STAGE: &'static str = "fastq.validate_pre";
    const VERSION: i32 = 1;

    fn validate(&self) -> Result<()> {
        if self.reads_valid + self.reads_invalid != self.reads_total {
            return Err(BenchError::Validation(
                "reads_valid + reads_invalid must equal reads_total".to_string(),
            ));
        }
        if !self.mean_q.is_finite() || !(0.0..=45.0).contains(&self.mean_q) {
            return Err(BenchError::Validation(
                "mean_q must be within [0, 45]".to_string(),
            ));
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
    const STAGE: &'static str = "fastq.filter";
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
