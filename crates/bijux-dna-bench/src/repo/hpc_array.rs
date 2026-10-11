//! Owner: common HPC array destination admission, atomic publication and current-byte comparison.
use anyhow::{anyhow, ensure, Context, Result};
use serde::Serialize;
use std::fs;
use std::path::{Component, Path, PathBuf};
#[derive(Clone, Copy)]
pub(crate) enum ArrayKind {
    Stage,
    Pipeline,
}
impl ArrayKind {
    fn label(self) -> &'static str {
        match self {
            Self::Stage => "HPC stage benchmark array",
            Self::Pipeline => "HPC pipeline node array",
        }
    }
    fn command(self) -> &'static str {
        match self {
            Self::Stage => "render-hpc-stage-benchmark-array",
            Self::Pipeline => "render-hpc-pipeline-node-array",
        }
    }
}
pub(crate) struct Destinations {
    kind: ArrayKind,
    pub(crate) script: PathBuf,
    pub(crate) manifest: PathBuf,
    pub(crate) logs: PathBuf,
}
impl Destinations {
    pub(crate) fn prepare(root: &Path, script: &Path, kind: ArrayKind) -> Result<Self> {
        let script = if script.is_absolute() { script.to_path_buf() } else { root.join(script) };
        let relative =
            script.strip_prefix(root.join("artifacts/runs/bench")).with_context(|| {
                format!("{} output must resolve inside `artifacts/runs/bench`", kind.label())
            })?;
        ensure!(
            !relative.as_os_str().is_empty()
                && relative.components().all(|c| matches!(c, Component::Normal(_))),
            "{} output must be a normalized file path inside `artifacts/runs/bench`",
            kind.label()
        );
        let parent =
            script.parent().ok_or_else(|| anyhow!("{} output has no parent", kind.label()))?;
        let stem = script
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(|| anyhow!("{} output has no valid file stem", kind.label()))?;
        let manifest = parent.join(format!("{stem}-manifest.json"));
        let logs = parent.join("logs").join(stem);
        Ok(Self { kind, script, manifest, logs })
    }
    pub(crate) fn publish<T: Serialize>(&self, script: &str, report: &T) -> Result<()> {
        if let Some(parent) = self.script.parent() {
            bijux_dna_infra::ensure_dir(parent)
                .with_context(|| format!("create {}", parent.display()))?;
        }
        bijux_dna_infra::write_string(&self.script, script)
            .with_context(|| format!("write {}", self.script.display()))?;
        bijux_dna_infra::atomic_write_json(&self.manifest, report)?;
        Ok(())
    }
    pub(crate) fn validate<T: Serialize>(
        &self,
        script: &str,
        report: &T,
        script_path: &str,
    ) -> Result<()> {
        let observed = fs::read_to_string(&self.script)
            .with_context(|| format!("read {}", self.script.display()))?;
        self.ensure_match(
            observed.as_bytes(),
            script.as_bytes(),
            &self.script,
            "script",
            script_path,
        )?;
        let observed = fs::read(&self.manifest)
            .with_context(|| format!("read {}", self.manifest.display()))?;
        let expected = serde_json::to_vec_pretty(report)
            .with_context(|| format!("serialize governed {} manifest", self.kind.label()))?;
        self.ensure_match(&observed, &expected, &self.manifest, "manifest", script_path)
    }
    fn ensure_match(
        &self,
        observed: &[u8],
        expected: &[u8],
        path: &Path,
        kind: &str,
        script_path: &str,
    ) -> Result<()> {
        ensure!(observed==expected,"{} {kind} `{}` drifted from governed dry-run inputs; rerun `bijux-dna bench local {} --output {script_path}`",self.kind.label(),path.display(),self.kind.command());
        Ok(())
    }
}
pub(crate) fn relative(root: &Path, path: &Path) -> String {
    path.strip_prefix(root).unwrap_or(path).to_string_lossy().replace('\\', "/")
}
