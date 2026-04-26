use crate::diagnostics::BenchError;

use super::BenchmarkObservation;

fn validate_nonnegative_finite(value: f64, field: &str) -> Result<(), BenchError> {
    if !value.is_finite() {
        return Err(BenchError::InvalidObservation { reason: format!("{field} must be finite") });
    }
    if value < 0.0 {
        return Err(BenchError::InvalidObservation {
            reason: format!("{field} must be non-negative"),
        });
    }
    Ok(())
}

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
        validate_nonnegative_finite(self.runtime_s, "runtime_s")?;
        validate_nonnegative_finite(self.memory_mb, "memory_mb")?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use crate::model::observation::MetricsEnvelope;

    use super::BenchmarkObservation;

    fn valid_observation() -> BenchmarkObservation {
        BenchmarkObservation {
            schema_version: "bijux.bench.observation.v1".to_string(),
            run_id: "run-1".to_string(),
            dataset_id: "dataset-1".to_string(),
            dataset_class: "trueseq".to_string(),
            read_layout: "paired".to_string(),
            stage_id: "fastq.trim_reads".to_string(),
            stage_instance_id: None,
            lineage_id: None,
            tool_id: "fastp".to_string(),
            tool_version: "0.23.4".to_string(),
            image_digest: "sha256:abc".to_string(),
            container_digest: "sha256:abc".to_string(),
            params_hash: "params-a".to_string(),
            input_hash: "input-a".to_string(),
            runtime_s: 1.0,
            memory_mb: 128.0,
            exit_code: 0,
            failure_kind: None,
            metrics: MetricsEnvelope {
                stage_id: "fastq.trim_reads".to_string(),
                schema_version: "metrics.v1".to_string(),
                values: BTreeMap::new(),
            },
            replicate_id: "r1".to_string(),
            replicate_index: 0,
            warmup_policy: "none".to_string(),
            seed_policy: "fixed".to_string(),
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            cpu: "x86_64".to_string(),
            threads: 4,
            io_mode: "local".to_string(),
        }
    }

    #[test]
    fn observation_rejects_nonfinite_runtime() {
        let mut obs = valid_observation();
        obs.runtime_s = f64::NAN;

        let err = obs.validate().expect_err("nan runtime should fail");

        assert!(err.to_string().contains("runtime_s must be finite"));
    }

    #[test]
    fn observation_rejects_negative_memory() {
        let mut obs = valid_observation();
        obs.memory_mb = -1.0;

        let err = obs.validate().expect_err("negative memory should fail");

        assert!(err.to_string().contains("memory_mb must be non-negative"));
    }
}
