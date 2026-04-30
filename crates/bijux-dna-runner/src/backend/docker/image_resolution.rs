mod apptainer_registry;
mod docker_policy;

use anyhow::Result;
use bijux_dna_environment::api::{
    resolve_image, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageSpec,
};

use apptainer_registry::resolve_apptainer_image_for_run;
use docker_policy::resolve_docker_image_for_run;

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
        RuntimeKind::Local => Ok(image),
        RuntimeKind::Docker => resolve_docker_image_for_run(spec, platform, image),
        RuntimeKind::Apptainer | RuntimeKind::Singularity => {
            resolve_apptainer_image_for_run(spec, platform, image)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{apptainer_registry::apptainer_registry_root, resolve_image_for_run};
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
            container_dir: PathBuf::from("/artifacts/runtime/does-not-exist-bijux"),
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
        let sif_path = registry_dir.join("16e615286a66.sif");
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
    fn resolve_image_for_run_rejects_placeholder_only_registry_sif() -> anyhow::Result<()> {
        let temp = bijux_dna_infra::temp_dir("bijux-runner-apptainer-placeholder-registry")?;
        let flat_dir = temp.path().join("apptainer").join("sif");
        let registry_dir = temp.path().join("seqkit");
        bijux_dna_infra::ensure_dir(&flat_dir)?;
        bijux_dna_infra::ensure_dir(&registry_dir)?;
        bijux_dna_infra::atomic_write_bytes(&registry_dir.join("pending.sif"), b"sif")?;
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

        let error = match resolve_image_for_run(&spec, &platform) {
            Ok(image) => panic!("expected placeholder sif rejection, got {}", image.full_name),
            Err(error) => error,
        };
        assert!(error.to_string().contains("apptainer image not found"));
        Ok(())
    }

    #[test]
    fn apptainer_registry_root_uses_parent_of_flat_sif_layout() {
        let flat_dir = Path::new("/shared/containers/apptainer/sif");
        assert_eq!(apptainer_registry_root(flat_dir), PathBuf::from("/shared/containers"));
    }
}
