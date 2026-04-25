use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_environment::api::{PlatformSpec, ResolvedImage, ToolImageSpec};

pub(super) fn resolve_apptainer_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: ResolvedImage,
) -> Result<ResolvedImage> {
    let candidates = apptainer_image_candidates(spec, platform);
    let image_path = candidates
        .iter()
        .find(|path| path.is_file())
        .cloned()
        .unwrap_or_else(|| candidates[0].clone());
    let resolved = ResolvedImage {
        full_name: image_path.display().to_string(),
        arch: image.arch,
        runner: image.runner,
    };
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() || image_path.is_file() {
        return Ok(resolved);
    }
    Err(anyhow!(
        "apptainer image not found for tool {}. checked: {}",
        spec.tool,
        candidates.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ")
    ))
}

fn apptainer_image_candidates(spec: &ToolImageSpec, platform: &PlatformSpec) -> Vec<PathBuf> {
    let registry_root = apptainer_registry_root(&platform.container_dir);
    let mut candidates = vec![platform.container_dir.join(format!("{}.sif", spec.tool))];
    if let Some(digest) = spec.digest.as_deref().filter(|digest| !is_placeholder_digest(digest)) {
        let normalized_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
        candidates.push(registry_root.join(&spec.tool).join(format!("{normalized_digest}.sif")));
        candidates.push(registry_root.join(&spec.tool).join(format!("{digest}.sif")));
    } else if let Some(unique_sif) = unique_registry_sif(&registry_root, &spec.tool) {
        candidates.push(unique_sif);
    }
    candidates.dedup();
    candidates
}

fn is_placeholder_digest(digest: &str) -> bool {
    let normalized = digest.strip_prefix("sha256:").unwrap_or(digest).trim();
    normalized.eq_ignore_ascii_case("pending")
        || (!normalized.is_empty() && normalized.chars().all(|char| char == '0'))
}

fn unique_registry_sif(registry_root: &Path, tool: &str) -> Option<PathBuf> {
    let tool_dir = registry_root.join(tool);
    let entries = fs::read_dir(&tool_dir).ok()?;
    let mut sifs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sif"))
        .filter(|path| {
            path.file_stem()
                .and_then(|stem| stem.to_str())
                .is_some_and(|stem| !is_placeholder_digest(stem))
        })
        .collect::<Vec<_>>();
    sifs.sort();
    if sifs.len() == 1 {
        return sifs.into_iter().next();
    }
    None
}

pub(crate) fn apptainer_registry_root(container_dir: &Path) -> PathBuf {
    let parent = container_dir.parent();
    let grandparent = parent.and_then(Path::parent);
    let is_flat_apptainer_sif_dir =
        container_dir.file_name().and_then(|name| name.to_str()).is_some_and(|name| name == "sif")
            && parent
                .and_then(Path::file_name)
                .and_then(|name| name.to_str())
                .is_some_and(|name| name == "apptainer");
    if is_flat_apptainer_sif_dir {
        return grandparent.unwrap_or(container_dir).to_path_buf();
    }
    container_dir.to_path_buf()
}

#[cfg(test)]
mod tests {
    use super::*;
    use bijux_dna_environment::api::RuntimeKind;

    fn apptainer_platform(container_dir: PathBuf) -> PlatformSpec {
        PlatformSpec {
            name: "apptainer-amd64".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir,
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        }
    }

    #[test]
    fn placeholder_digest_uses_unique_registry_sif() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let container_dir = temp.path().join("containers").join("apptainer").join("sif");
        let registry_dir = temp.path().join("containers").join("seqtk");
        fs::create_dir_all(&container_dir)?;
        fs::create_dir_all(&registry_dir)?;
        let verified_sif = registry_dir.join("16e615286a66.sif");
        fs::write(&verified_sif, b"sif")?;
        let spec = ToolImageSpec {
            tool: "seqtk".to_string(),
            version: "1.5-r133".to_string(),
            digest: Some(
                "sha256:0000000000000000000000000000000000000000000000000000000000000000"
                    .to_string(),
            ),
            enabled: None,
            shipping_policy: None,
        };
        let image = ResolvedImage {
            full_name: "seqtk".to_string(),
            arch: "amd64".to_string(),
            runner: RuntimeKind::Apptainer,
        };

        let resolved =
            resolve_apptainer_image_for_run(&spec, &apptainer_platform(container_dir), image)?;

        assert_eq!(resolved.full_name, verified_sif.display().to_string());
        Ok(())
    }
}
