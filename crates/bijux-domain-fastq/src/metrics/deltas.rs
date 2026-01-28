#![allow(dead_code)]

use anyhow::{anyhow, Result};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct FastqDelta {
    pub delta_mean_q: f64,
    pub delta_gc: f64,
    pub read_retention: f64,
    pub base_retention: f64,
}

impl FastqDelta {
    /// Validate delta metrics are sane.
    ///
    /// # Errors
    /// Returns an error if any delta values are invalid.
    pub fn validate(&self) -> Result<()> {
        if !self.delta_mean_q.is_finite() {
            return Err(anyhow!("delta_mean_q must be finite"));
        }
        if !self.delta_gc.is_finite() {
            return Err(anyhow!("delta_gc must be finite"));
        }
        if !(0.0..=1.0).contains(&self.read_retention) {
            return Err(anyhow!("read_retention must be within [0, 1]"));
        }
        if !(0.0..=1.0).contains(&self.base_retention) {
            return Err(anyhow!("base_retention must be within [0, 1]"));
        }
        Ok(())
    }
}

#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn ratio_u64(num: u64, denom: u64) -> f64 {
    if denom == 0 {
        0.0
    } else {
        (num as f64) / (denom as f64)
    }
}

#[must_use]
pub fn compute_delta(
    before: bijux_engine::api::SeqkitMetrics,
    after: bijux_engine::api::SeqkitMetrics,
) -> FastqDelta {
    let read_retention = if before.reads > 0 {
        ratio_u64(after.reads, before.reads)
    } else {
        0.0
    };
    let base_retention = if before.bases > 0 {
        ratio_u64(after.bases, before.bases)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: after.mean_q - before.mean_q,
        delta_gc: after.gc_percent - before.gc_percent,
        read_retention,
        base_retention,
    }
}

#[must_use]
#[allow(clippy::too_many_arguments)]
pub fn delta_from_counts(
    reads_in: u64,
    reads_out: u64,
    bases_in: u64,
    bases_out: u64,
    mean_q_before: f64,
    mean_q_after: f64,
    gc_before: f64,
    gc_after: f64,
) -> FastqDelta {
    let read_retention = if reads_in > 0 {
        ratio_u64(reads_out, reads_in)
    } else {
        0.0
    };
    let base_retention = if bases_in > 0 {
        ratio_u64(bases_out, bases_in)
    } else {
        0.0
    };
    FastqDelta {
        delta_mean_q: mean_q_after - mean_q_before,
        delta_gc: gc_after - gc_before,
        read_retention,
        base_retention,
    }
}
