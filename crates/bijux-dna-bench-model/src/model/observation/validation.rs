use crate::diagnostics::BenchError;

use super::BenchmarkObservation;

impl BenchmarkObservation {
    /// # Errors
    /// Returns an error if required confounders are missing.
    pub fn validate(&self) -> Result<(), BenchError> {
        if self.schema_version.trim().is_empty() {
            return Err(BenchError::InvalidObservation {
                reason: "missing schema_version".to_string(),
            });
        }
        if self.dataset_class.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "dataset_class" });
        }
        if self.read_layout.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "read_layout" });
        }
        if self.platform.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "platform" });
        }
        if self.stage_instance_id.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(BenchError::MissingConfounder { field: "stage_instance_id" });
        }
        if self.lineage_id.as_deref().is_some_and(|value| value.trim().is_empty()) {
            return Err(BenchError::MissingConfounder { field: "lineage_id" });
        }
        if self.cpu.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "cpu" });
        }
        if self.threads == 0 {
            return Err(BenchError::MissingConfounder { field: "threads" });
        }
        if self.runner.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "runner" });
        }
        if self.io_mode.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "io_mode" });
        }
        if self.image_digest.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "image_digest" });
        }
        if self.container_digest.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "container_digest" });
        }
        if self.warmup_policy.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "warmup_policy" });
        }
        if self.seed_policy.trim().is_empty() {
            return Err(BenchError::MissingConfounder { field: "seed_policy" });
        }
        Ok(())
    }
}
