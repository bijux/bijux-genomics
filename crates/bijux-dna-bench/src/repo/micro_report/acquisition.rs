//! Owner: lazy evidence acquisition shared by the projections of one micro result.
use anyhow::{Context, Result};
use bijux_dna_bench_model::BenchmarkStageResultManifest;
use serde_json::Value;
use std::{fs, path::Path};

/// Lifetime is one result row, bounding retained payloads to one JSON and one stage manifest.
pub(super) struct ResultEvidence<'a> {
    root: &'a Path,
    json_path: Option<&'a str>,
    stage_path: Option<&'a str>,
    json: Option<Value>,
    stage: Option<BenchmarkStageResultManifest>,
}
impl<'a> ResultEvidence<'a> {
    pub(super) fn new(
        root: &'a Path,
        json_path: Option<&'a str>,
        stage_path: Option<&'a str>,
    ) -> Self {
        Self { root, json_path, stage_path, json: None, stage: None }
    }
    pub(super) fn json(&mut self) -> Result<&Value> {
        if self.json.is_none() {
            let path =
                self.root.join(self.json_path.context("micro result has no JSON evidence path")?);
            let payload =
                fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
            self.json = Some(
                serde_json::from_str(&payload)
                    .with_context(|| format!("parse {}", path.display()))?,
            );
        }
        self.json.as_ref().context("micro JSON evidence was not acquired")
    }
    pub(super) fn stage_result(&mut self) -> Result<&BenchmarkStageResultManifest> {
        if self.stage.is_none() {
            let path =
                self.root.join(self.stage_path.context("micro result has no stage-result path")?);
            self.stage = Some(
                crate::repo::stage_result::load_benchmark_stage_result_manifest(&path)
                    .with_context(|| format!("load {}", path.display()))?,
            );
        }
        self.stage.as_ref().context("micro stage evidence was not acquired")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn result_projections_reuse_acquired_bytes_and_new_results_observe_changes() -> Result<()> {
        let temp = tempfile::tempdir()?;
        let root = temp.path();
        let path = root.join("evidence.json");
        fs::write(&path, r#"{"elapsed_seconds":1.0,"memory_mb":32.0}"#)?;
        let mut observed = ResultEvidence::new(root, Some("evidence.json"), None);
        assert_eq!(observed.json()?["elapsed_seconds"], 1.0);
        fs::write(&path, r#"{"elapsed_seconds":2.0,"memory_mb":64.0}"#)?;
        assert_eq!(observed.json()?["memory_mb"], 32.0);
        let mut fresh = ResultEvidence::new(root, Some("evidence.json"), None);
        assert_eq!(fresh.json()?["memory_mb"], 64.0);
        fs::remove_file(&path)?;
        assert_eq!(observed.json()?["elapsed_seconds"], 1.0);
        assert!(ResultEvidence::new(root, Some("evidence.json"), None).json().is_err());
        assert!(ResultEvidence::new(root, None, None).stage_result().is_err());
        Ok(())
    }
}
