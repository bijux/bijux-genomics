//! Owner: bijux-dna-bench
//! Loader for finished benchmark observation streams.

use std::path::PathBuf;

use anyhow::{Context, Result};

use bijux_dna_bench_model::contract::validate_observation;
use bijux_dna_bench_model::BenchError;

pub fn load_observations(
    path: &PathBuf,
) -> Result<Vec<bijux_dna_bench_model::BenchmarkObservation>> {
    if !path.exists() {
        return Err(BenchError::MissingMetrics(format!(
            "observations file missing: {}",
            path.display()
        ))
        .into());
    }
    let raw = std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    let mut observations = Vec::new();
    for (line_number, line) in raw.lines().enumerate() {
        if line.trim().is_empty() {
            continue;
        }
        let obs: bijux_dna_bench_model::BenchmarkObservation = serde_json::from_str(line)
            .with_context(|| format!("parse {} line {}", path.display(), line_number + 1))?;
        validate_observation(&obs)
            .with_context(|| format!("validate {} line {}", path.display(), line_number + 1))?;
        observations.push(obs);
    }
    Ok(observations)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_bench_model::{BenchmarkObservation, MetricsEnvelope};

    use super::load_observations;

    fn sample_observation() -> BenchmarkObservation {
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
            input_hash: "input".to_string(),
            runtime_s: 1.0,
            memory_mb: 100.0,
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
            seed_policy: "default".to_string(),
            runner: "docker".to_string(),
            platform: "linux".to_string(),
            cpu: "x86_64".to_string(),
            threads: 4,
            io_mode: "local".to_string(),
        }
    }

    #[test]
    fn load_observations_rejects_invalid_rows() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let path = temp.path().join("observations.jsonl");
        let mut invalid = sample_observation();
        invalid.runner.clear();
        let line = serde_json::to_string(&invalid)?;
        bijux_dna_runtime::recording::write_atomic_bytes(&path, line.as_bytes())?;

        let result = load_observations(&path);

        assert!(result.is_err());
        let message = result.err().map(|err| err.to_string()).unwrap_or_default();
        assert!(message.contains("validate"));
        Ok(())
    }
}
