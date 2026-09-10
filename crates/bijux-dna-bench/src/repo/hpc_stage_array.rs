//! Owner: stage array destination admission, publication and current-byte comparison.
use anyhow::{anyhow, ensure, Context, Result};
use bijux_dna_bench_model::BenchmarkLocalHpcStageBenchmarkArrayReport;
use std::fs;
use std::path::{Component, Path, PathBuf};
pub(crate) struct Destinations {
    pub(crate) script: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) logs: PathBuf,
}
impl Destinations {
    pub(crate) fn prepare(root: &Path, script: &Path) -> Result<Self> {
        let script = if script.is_absolute() { script.to_path_buf() } else { root.join(script) };
        let relative = script.strip_prefix(root.join("artifacts/runs/bench")).context(
            "HPC stage benchmark array output must resolve inside `artifacts/runs/bench`",
        )?;
        ensure!(!relative.as_os_str().is_empty() && relative.components().all(|c|matches!(c,Component::Normal(_))),"HPC stage benchmark array output must be a normalized file path inside `artifacts/runs/bench`");
        let parent = script
            .parent()
            .ok_or_else(|| anyhow!("HPC stage benchmark array output has no parent"))?;
        let stem = script
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("HPC stage benchmark array output has no valid file stem"))?;
        let manifest = parent.join(format!("{stem}-manifest.json"));
        let logs = parent.join("logs").join(stem);
        Ok(Self { script, manifest, logs })
    }
    pub(crate) fn publish(
        &self,
        script: &str,
        report: &BenchmarkLocalHpcStageBenchmarkArrayReport,
    ) -> Result<()> {
        if let Some(parent) = self.script.parent() {
            fs::create_dir_all(parent).with_context(|| format!("create {}", parent.display()))?;
        }
        fs::write(&self.script, script)
            .with_context(|| format!("write {}", self.script.display()))?;
        bijux_dna_infra::atomic_write_json(&self.manifest, report)?;
        Ok(())
    }
    pub(crate) fn validate(
        &self,
        script: &str,
        report: &BenchmarkLocalHpcStageBenchmarkArrayReport,
    ) -> Result<()> {
        let observed = fs::read_to_string(&self.script)
            .with_context(|| format!("read {}", self.script.display()))?;
        Self::ensure_match(observed.as_bytes(), script.as_bytes(), &self.script, "script", report)?;
        let observed = fs::read(&self.manifest)
            .with_context(|| format!("read {}", self.manifest.display()))?;
        let expected = serde_json::to_vec_pretty(report)
            .context("serialize governed HPC stage benchmark array manifest")?;
        Self::ensure_match(&observed, &expected, &self.manifest, "manifest", report)
    }
    fn ensure_match(
        observed: &[u8],
        expected: &[u8],
        path: &Path,
        kind: &str,
        report: &BenchmarkLocalHpcStageBenchmarkArrayReport,
    ) -> Result<()> {
        ensure!(observed==expected,"HPC stage benchmark array {kind} `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local render-hpc-stage-benchmark-array --output {}`",path.display(),report.script_path);
        Ok(())
    }
}
pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
