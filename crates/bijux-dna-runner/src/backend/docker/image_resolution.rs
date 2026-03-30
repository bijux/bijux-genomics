use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Result};
use bijux_dna_environment::api::{
    docker_image_exists, resolve_image, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageSpec,
};
use tracing::warn;

/// Resolve a concrete image reference for execution and verify local availability.
///
/// # Errors
/// Returns an error if resolution fails or required local images are unavailable.
pub fn resolve_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
) -> Result<ResolvedImage> {
    let image = resolve_image(spec, platform)?;
    match platform.runner {
        RuntimeKind::Docker => resolve_docker_image_for_run(spec, platform, image),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            resolve_apptainer_image_for_run(spec, platform, image)
        }
    }
}

fn resolve_docker_image_for_run(
    spec: &ToolImageSpec,
    platform: &PlatformSpec,
    image: ResolvedImage,
) -> Result<ResolvedImage> {
    if std::env::var("BIJUX_SKIP_IMAGE_CHECK").is_ok() {
        return Ok(image);
    }
    if docker_image_exists(&image) {
        return Ok(image);
    }
    if spec.digest.is_some() {
        let fallback = ResolvedImage {
            full_name: format!(
                "{}/{}:{}-{}",
                platform.image_prefix, spec.tool, spec.version, platform.arch
            ),
            arch: platform.arch.clone(),
            runner: platform.runner,
        };
        if docker_image_exists(&fallback) {
            warn!(
                "digest image missing locally; falling back to tag {}",
                fallback.full_name
            );
            return Ok(fallback);
        }
    }
    Err(anyhow!("docker image not found: {}", image.full_name))
}

fn resolve_apptainer_image_for_run(
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
        candidates
            .iter()
            .map(|path| path.display().to_string())
            .collect::<Vec<_>>()
            .join(", ")
    ))
}

fn apptainer_image_candidates(spec: &ToolImageSpec, platform: &PlatformSpec) -> Vec<PathBuf> {
    let registry_root = apptainer_registry_root(&platform.container_dir);
    let mut candidates = vec![platform.container_dir.join(format!("{}.sif", spec.tool))];
    if let Some(digest) = spec.digest.as_deref() {
        let normalized_digest = digest.strip_prefix("sha256:").unwrap_or(digest);
        candidates.push(
            registry_root
                .join(&spec.tool)
                .join(format!("{normalized_digest}.sif")),
        );
        candidates.push(registry_root.join(&spec.tool).join(format!("{digest}.sif")));
    } else if let Some(unique_sif) = unique_registry_sif(&registry_root, &spec.tool) {
        candidates.push(unique_sif);
    }
    candidates.dedup();
    candidates
}

fn unique_registry_sif(registry_root: &Path, tool: &str) -> Option<PathBuf> {
    let tool_dir = registry_root.join(tool);
    let entries = fs::read_dir(&tool_dir).ok()?;
    let mut sifs = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "sif"))
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
    let is_flat_apptainer_sif_dir = container_dir
        .file_name()
        .and_then(|name| name.to_str())
        .is_some_and(|name| name == "sif")
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
    use super::{apptainer_registry_root, resolve_image_for_run};
    use bijux_dna_environment::api::{PlatformSpec, RuntimeKind, ToolImageSpec};
    use std::path::{Path, PathBuf};

    #[test]
    fn resolve_image_for_run_uses_platform_sif_for_apptainer() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer")?;
        let sif_path = temp.path().join("fastqc.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "cluster-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: temp.path().to_path_buf(),
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn resolve_image_for_run_rejects_missing_apptainer_sif() {
        let platform = PlatformSpec {
            name: "cluster-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: PathBuf::from("/tmp/does-not-exist-bijux"),
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let error = match resolve_image_for_run(&spec, &platform) {
            Ok(image) => panic!("expected missing sif failure, got {}", image.full_name),
            Err(error) => error,
        };
        assert!(error.to_string().contains("apptainer image not found"));
    }

    #[test]
    fn resolve_image_for_run_uses_digest_pinned_apptainer_sif_when_flat_name_is_absent(
    ) -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer-registry")?;
        let flat_dir = temp.path().join("apptainer").join("sif");
        let registry_dir = temp.path().join("fastqc");
        bijux_dna_infra::ensure_dir(&flat_dir)?;
        bijux_dna_infra::ensure_dir(&registry_dir)?;
        let sif_path = registry_dir.join("abc123.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "cluster-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: flat_dir,
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "fastqc".to_string(),
            version: "latest-pinned".to_string(),
            digest: Some("sha256:abc123".to_string()),
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn resolve_image_for_run_uses_single_registry_sif_when_digest_is_missing() -> anyhow::Result<()>
    {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer-unique-registry")?;
        let flat_dir = temp.path().join("apptainer").join("sif");
        let registry_dir = temp.path().join("seqkit");
        bijux_dna_infra::ensure_dir(&flat_dir)?;
        bijux_dna_infra::ensure_dir(&registry_dir)?;
        let sif_path = registry_dir.join("pending.sif");
        bijux_dna_infra::atomic_write_bytes(&sif_path, b"sif")?;
        let platform = PlatformSpec {
            name: "cluster-apptainer".to_string(),
            runner: RuntimeKind::Apptainer,
            container_dir: flat_dir,
            image_prefix: "bijuxdna".to_string(),
            arch: "amd64".to_string(),
        };
        let spec = ToolImageSpec {
            tool: "seqkit".to_string(),
            version: "latest-pinned".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        };

        let image = resolve_image_for_run(&spec, &platform)?;

        assert_eq!(image.full_name, sif_path.display().to_string());
        assert_eq!(image.runner, RuntimeKind::Apptainer);
        Ok(())
    }

    #[test]
    fn apptainer_registry_root_uses_parent_of_flat_sif_layout() {
        let flat_dir = Path::new("/shared/containers/apptainer/sif");
        assert_eq!(
            apptainer_registry_root(flat_dir),
            PathBuf::from("/shared/containers")
        );
    }
}
