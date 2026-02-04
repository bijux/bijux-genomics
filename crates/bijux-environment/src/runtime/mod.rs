#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::path::PathBuf;
    use std::str::FromStr;

    use crate::resolve::{
        apptainer_sif_path, available_runners_with, cache_dir, docker_image_exists_with,
        load_image_catalog_from_file, resolve_image, select_best_runner, validate_images_for_stage,
        EnvError, ImageRef, PlatformSpec, ResolvedImage, RunnerKind, ToolImageSpec,
    };
    use bijux_infra::atomic_write_bytes;

    #[test]
    fn runner_kind_from_str() -> Result<(), EnvError> {
        assert_eq!(RunnerKind::from_str("docker")?, RunnerKind::Docker);
        assert_eq!(
            RunnerKind::from_str("singularity")?,
            RunnerKind::Singularity
        );
        assert_eq!(RunnerKind::from_str("apptainer")?, RunnerKind::Apptainer);
        Ok(())
    }

    #[test]
    fn platform_spec_toml_roundtrip() -> Result<(), EnvError> {
        let toml = r#"
name = "docker-mac-arm64"
runner = "docker"
container_dir = "containers/docker/arm64"
image_prefix = "bijuxdna"
arch = "arm64"
"#;
        let spec: PlatformSpec =
            bijux_infra::formats::parse_toml(toml).map_err(|err| EnvError::Parse(err.message))?;
        assert_eq!(spec.name, "docker-mac-arm64");
        assert_eq!(spec.runner, RunnerKind::Docker);
        let out = bijux_infra::formats::to_toml_string(&spec)
            .map_err(|err| EnvError::Parse(err.message))?;
        let spec2: PlatformSpec =
            bijux_infra::formats::parse_toml(&out).map_err(|err| EnvError::Parse(err.message))?;
        assert_eq!(spec2.name, spec.name);
        Ok(())
    }

    #[test]
    fn image_ref_formats() {
        let image = ImageRef {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            arch: "arm64".to_string(),
        };
        assert_eq!(
            image.to_full_name("bijuxdna"),
            "bijuxdna/fastp:0.23.4-arm64"
        );
    }

    #[test]
    fn available_runners_mocked() {
        let runners = available_runners_with(|cmd| cmd == "docker");
        assert_eq!(runners, vec![RunnerKind::Docker]);
    }

    #[test]
    fn select_best_runner_prefers_available() -> Result<(), EnvError> {
        let available = vec![RunnerKind::Apptainer, RunnerKind::Docker];
        let selected = select_best_runner(RunnerKind::Docker, &available)?;
        assert_eq!(selected, RunnerKind::Docker);
        Ok(())
    }

    #[test]
    fn select_best_runner_fallbacks() -> Result<(), EnvError> {
        let available = vec![RunnerKind::Singularity];
        let selected = select_best_runner(RunnerKind::Docker, &available)?;
        assert_eq!(selected, RunnerKind::Singularity);
        Ok(())
    }

    #[test]
    fn select_best_runner_errors() {
        let available = Vec::new();
        assert!(select_best_runner(RunnerKind::Docker, &available).is_err());
    }

    #[test]
    fn tool_image_spec_constructs() {
        let spec = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: None,
        };
        assert_eq!(spec.tool, "fastp");
    }

    #[test]
    fn resolve_image_builds_full_name() -> Result<(), EnvError> {
        let platform = PlatformSpec {
            name: "docker-mac-arm64".to_string(),
            runner: RunnerKind::Docker,
            container_dir: PathBuf::from("containers/docker/arm64"),
            image_prefix: "bijuxdna".to_string(),
            arch: "arm64".to_string(),
        };
        let tool = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: None,
        };
        let resolved = resolve_image(&tool, &platform)?;
        assert_eq!(resolved.full_name, "bijuxdna/fastp:0.23.4-arm64");
        assert_eq!(resolved.arch, "arm64");
        assert_eq!(resolved.runner, RunnerKind::Docker);
        Ok(())
    }

    #[test]
    fn resolved_image_compatibility() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Docker,
        };
        assert!(image.is_compatible(RunnerKind::Docker));
        assert!(!image.is_compatible(RunnerKind::Apptainer));
        assert!(!image.is_compatible(RunnerKind::Singularity));

        let oci = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Apptainer,
        };
        assert!(oci.is_compatible(RunnerKind::Apptainer));
        assert!(oci.is_compatible(RunnerKind::Singularity));
        assert!(!oci.is_compatible(RunnerKind::Docker));
    }

    #[test]
    fn load_image_catalog_parses() -> Result<(), EnvError> {
        let temp_dir = std::env::temp_dir();
        let path = temp_dir.join("bijux_images.toml");
        atomic_write_bytes(
            &path,
            b"[fastp]\nversion = \"0.23.4\"\ndigest = \"sha256:abc123\"\n\n[bwa]\nversion = \"0.7.17\"\n",
        )
        .map_err(std::io::Error::other)?;
        let catalog = load_image_catalog_from_file(&path)?;
        assert!(catalog.contains_key("fastp"));
        let _ = bijux_infra::remove_file(&path);
        Ok(())
    }

    #[test]
    fn validate_images_for_stage_errors() {
        let mut catalog = HashMap::new();
        catalog.insert(
            "fastp".to_string(),
            ToolImageSpec {
                tool: "fastp".to_string(),
                version: "0.23.4".to_string(),
                digest: None,
            },
        );
        match validate_images_for_stage(&catalog, &["fastp", "bwa"]) {
            Ok(()) => panic!("expected error for missing bwa"),
            Err(err) => {
                assert!(format!("{err}").contains("bwa"));
            }
        }
    }

    #[test]
    fn resolve_image_with_digest() -> Result<(), EnvError> {
        let platform = PlatformSpec {
            name: "docker-mac-arm64".to_string(),
            runner: RunnerKind::Docker,
            container_dir: PathBuf::from("containers/docker/arm64"),
            image_prefix: "bijuxdna".to_string(),
            arch: "arm64".to_string(),
        };
        let tool = ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: Some("sha256:abc123".to_string()),
        };
        let resolved = resolve_image(&tool, &platform)?;
        assert_eq!(resolved.full_name, "bijuxdna/fastp@sha256:abc123");
        Ok(())
    }

    #[test]
    fn cache_dir_is_deterministic() -> Result<(), EnvError> {
        let home = std::env::temp_dir().join("bijux_home");
        bijux_infra::ensure_dir(&home)?;
        let original = std::env::var_os("HOME");
        std::env::set_var("HOME", &home);
        let docker = cache_dir(RunnerKind::Docker);
        let apptainer = cache_dir(RunnerKind::Apptainer);
        if let Some(value) = original {
            std::env::set_var("HOME", value);
        }
        assert!(docker
            .to_string_lossy()
            .contains(".cache/bijux/docker/images"));
        assert!(apptainer
            .to_string_lossy()
            .contains(".cache/bijux/apptainer/sif"));
        Ok(())
    }

    #[test]
    fn docker_image_exists_mocked() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Docker,
        };
        let exists = docker_image_exists_with(&image, |args| {
            args == ["image", "inspect", "bijuxdna/fastp:0.23.4-arm64"]
        });
        assert!(exists);
    }

    #[test]
    fn apptainer_sif_path_is_deterministic() {
        let image = ResolvedImage {
            full_name: "bijuxdna/fastp@sha256:abc123".to_string(),
            arch: "arm64".to_string(),
            runner: RunnerKind::Apptainer,
        };
        let path = apptainer_sif_path(&image);
        assert!(path
            .to_string_lossy()
            .contains("fastp-sha256:abc123-arm64.sif"));
    }
}
