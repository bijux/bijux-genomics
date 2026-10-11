//! Owner: bijux-dna-bench-workflows
//! Amplicon execution artifacts.
use super::super::{fs, Context, Deserialize, Path, PathBuf, Result, Value};

pub(crate) fn load_json<T: for<'de> Deserialize<'de>>(path: &Path) -> Result<T> {
    let raw = fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

pub(crate) fn write_json_file(path: &Path, value: &Value) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    let rendered = serde_json::to_vec_pretty(value)
        .with_context(|| format!("serialize {}", path.display()))?;
    bijux_dna_infra::write_bytes(path, rendered)
        .with_context(|| format!("write {}", path.display()))
}

pub(crate) fn write_text_file(path: &Path, content: &str) -> Result<()> {
    if let Some(parent) = path.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    bijux_dna_infra::write_bytes(path, content).with_context(|| format!("write {}", path.display()))
}

pub(crate) fn is_nonempty_file(path: &Path) -> bool {
    path.metadata().is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0)
}

pub(crate) fn copy_repo_relative_file(
    repo_root: &Path,
    stage_root: &Path,
    source_relative_path: &str,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    copy_absolute_file(
        repo_root,
        stage_root,
        &repo_root.join(source_relative_path),
        destination_relative_path,
    )
}

pub(crate) fn copy_repo_relative_file_with_absolute_fallback(
    repo_root: &Path,
    stage_root: &Path,
    source_relative_path: &str,
    fallback_source_path: &Path,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    let source_path = repo_root.join(source_relative_path);
    let resolved_source_path =
        if source_path.metadata().is_ok_and(|metadata| metadata.is_file() && metadata.len() > 0) {
            source_path
        } else {
            fallback_source_path.to_path_buf()
        };
    copy_absolute_file(repo_root, stage_root, &resolved_source_path, destination_relative_path)
}

pub(crate) fn copy_absolute_file(
    _repo_root: &Path,
    stage_root: &Path,
    source_path: &Path,
    destination_relative_path: &str,
) -> Result<PathBuf> {
    let destination = stage_root.join(destination_relative_path);
    if let Some(parent) = destination.parent() {
        bijux_dna_infra::ensure_dir(parent)
            .with_context(|| format!("create {}", parent.display()))?;
    }
    fs::copy(source_path, &destination)
        .with_context(|| format!("copy {} to {}", source_path.display(), destination.display()))?;
    Ok(destination)
}

pub(crate) fn stage_root(output_root: &Path, stage_id: &str) -> PathBuf {
    output_root.join("artifacts").join(stage_id)
}

/// Completed stage artifacts retain their typed producers through judgment and handoff assembly.
pub(crate) struct AmpliconStageArtifacts {
    pub(crate) corpus: super::super::AmpliconCorpusStageArtifacts,
    pub(crate) truth: super::super::AmpliconTruthStageArtifacts,
    pub(crate) normalize: super::super::NormalizePrimersStageArtifacts,
    pub(crate) infer: super::super::InferAsvsStageArtifacts,
    pub(crate) chimera: super::super::RemoveChimerasStageArtifacts,
    pub(crate) otu: super::super::ClusterOtusStageArtifacts,
    pub(crate) abundance: super::super::NormalizeAbundanceStageArtifacts,
}
