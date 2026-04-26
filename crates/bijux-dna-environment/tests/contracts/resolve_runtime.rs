use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::{Mutex, MutexGuard};

use bijux_dna_environment::resolve::{
    apptainer_sif_path, cache_dir, docker_image_exists, load_image_catalog, load_platform,
    resolve_image, run_shell_capture, select_best_runner, validate_images_for_stage, EnvError,
    ImageRef, PlatformSpec, ResolvedImage, RuntimeKind, ToolImageSpec,
};
use bijux_dna_environment::runtime_spec::{is_platform_runner_compatible, RuntimeSpec};

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn env_lock() -> MutexGuard<'static, ()> {
    ENV_LOCK.lock().unwrap_or_else(|err| panic!("environment lock poisoned: {err}"))
}

struct CurrentDirGuard {
    previous: PathBuf,
}

impl CurrentDirGuard {
    fn change_to(path: &Path) -> anyhow::Result<Self> {
        let previous = std::env::current_dir()?;
        std::env::set_current_dir(path)?;
        Ok(Self { previous })
    }
}

impl Drop for CurrentDirGuard {
    fn drop(&mut self) {
        let _ = std::env::set_current_dir(&self.previous);
    }
}

struct EnvVarGuard {
    key: &'static str,
    value: Option<std::ffi::OsString>,
}

impl EnvVarGuard {
    fn capture(key: &'static str) -> Self {
        Self { key, value: std::env::var_os(key) }
    }
}

impl Drop for EnvVarGuard {
    fn drop(&mut self) {
        if let Some(value) = self.value.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

fn write_runtime_platform_fixture(root: &Path, body: &[u8]) -> anyhow::Result<()> {
    let path = root.join("configs/runtime/platforms.toml");
    bijux_dna_infra::atomic_write_bytes(&path, body)?;
    Ok(())
}

fn write_image_catalog_fixture(root: &Path, images: &[u8], registry: &[u8]) -> anyhow::Result<()> {
    bijux_dna_infra::atomic_write_bytes(&root.join("configs/ci/tools/images.toml"), images)?;
    bijux_dna_infra::atomic_write_bytes(
        &root.join("configs/ci/registry/tool_registry.toml"),
        registry,
    )?;
    Ok(())
}

#[test]
fn runtime_kind_from_str_parses_known_runners() -> Result<(), EnvError> {
    assert_eq!(RuntimeKind::from_str("docker")?, RuntimeKind::Docker);
    assert_eq!(RuntimeKind::from_str("singularity")?, RuntimeKind::Singularity);
    assert_eq!(RuntimeKind::from_str("apptainer")?, RuntimeKind::Apptainer);
    assert_eq!(RuntimeKind::from_str(" Docker ")? , RuntimeKind::Docker);
    Ok(())
}

#[test]
fn platform_spec_toml_roundtrip_is_stable() -> Result<(), EnvError> {
    let toml = r#"
name = "docker-mac-arm64"
runner = "docker"
container_dir = "containers/docker/arm64"
image_prefix = "bijuxdna"
arch = "arm64"
"#;
    let spec: PlatformSpec =
        bijux_dna_infra::formats::parse_toml(toml).map_err(|err| EnvError::Parse(err.message))?;
    let out = bijux_dna_infra::formats::to_toml_string(&spec)
        .map_err(|err| EnvError::Parse(err.message))?;
    let reparsed: PlatformSpec =
        bijux_dna_infra::formats::parse_toml(&out).map_err(|err| EnvError::Parse(err.message))?;
    assert_eq!(reparsed.name, spec.name);
    assert_eq!(reparsed.runner, spec.runner);
    Ok(())
}

#[test]
fn image_ref_formats_deterministically() {
    let image = ImageRef {
        tool: "fastp".to_string(),
        version: "0.23.4".to_string(),
        arch: "arm64".to_string(),
    };
    assert_eq!(image.to_full_name("bijuxdna"), "bijuxdna/fastp:0.23.4-arm64");
}

#[test]
fn select_best_runner_prefers_requested_runner_when_available() -> Result<(), EnvError> {
    let available = vec![RuntimeKind::Apptainer, RuntimeKind::Docker];
    assert_eq!(select_best_runner(RuntimeKind::Docker, &available)?, RuntimeKind::Docker);
    Ok(())
}

#[test]
fn select_best_runner_falls_back_to_oci_runners() -> Result<(), EnvError> {
    let available = vec![RuntimeKind::Singularity];
    assert_eq!(select_best_runner(RuntimeKind::Docker, &available)?, RuntimeKind::Singularity);
    Ok(())
}

#[test]
fn select_best_runner_errors_when_nothing_is_available() {
    assert!(select_best_runner(RuntimeKind::Docker, &[]).is_err());
}

#[test]
fn resolve_image_builds_tagged_and_digest_pinned_names() -> Result<(), EnvError> {
    let platform = PlatformSpec {
        name: "docker-mac-arm64".to_string(),
        runner: RuntimeKind::Docker,
        container_dir: PathBuf::from("containers/docker/arm64"),
        image_prefix: "bijuxdna".to_string(),
        arch: "arm64".to_string(),
    };
    let tagged = ToolImageSpec {
        tool: "fastp".to_string(),
        version: "0.23.4".to_string(),
        digest: None,
        enabled: None,
        shipping_policy: None,
    };
    let pinned = ToolImageSpec { digest: Some("sha256:abc123".to_string()), ..tagged.clone() };
    assert_eq!(resolve_image(&tagged, &platform)?.full_name, "bijuxdna/fastp:0.23.4-arm64");
    assert_eq!(resolve_image(&pinned, &platform)?.full_name, "bijuxdna/fastp@sha256:abc123");
    Ok(())
}

#[test]
fn resolved_image_and_runtime_spec_report_compatibility_consistently() {
    let docker_platform = PlatformSpec {
        name: "docker-mac-arm64".to_string(),
        runner: RuntimeKind::Docker,
        container_dir: PathBuf::from("containers/docker/arm64"),
        image_prefix: "bijuxdna".to_string(),
        arch: "arm64".to_string(),
    };
    let oci_platform = PlatformSpec {
        name: "apptainer-amd64".to_string(),
        runner: RuntimeKind::Apptainer,
        container_dir: PathBuf::from("containers/apptainer/sif"),
        image_prefix: "bijuxdna".to_string(),
        arch: "amd64".to_string(),
    };
    let docker_image = ResolvedImage {
        full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
        arch: "arm64".to_string(),
        runner: RuntimeKind::Docker,
    };
    let oci_image = ResolvedImage {
        full_name: "bijuxdna/fastp@sha256:abc123".to_string(),
        arch: "amd64".to_string(),
        runner: RuntimeKind::Apptainer,
    };
    assert!(docker_image.is_compatible(RuntimeKind::Docker));
    assert!(!docker_image.is_compatible(RuntimeKind::Apptainer));
    assert!(oci_image.is_compatible(RuntimeKind::Singularity));
    assert!(is_platform_runner_compatible(&oci_platform, RuntimeKind::Singularity));
    assert!(!is_platform_runner_compatible(&docker_platform, RuntimeKind::Apptainer));
    assert!(RuntimeSpec::new(RuntimeKind::Docker, docker_platform).is_compatible());
    assert!(RuntimeSpec::new(RuntimeKind::Singularity, oci_platform).is_compatible());
}

#[test]
fn validate_images_for_stage_reports_missing_tools() {
    let mut catalog = HashMap::new();
    catalog.insert(
        "fastp".to_string(),
        ToolImageSpec {
            tool: "fastp".to_string(),
            version: "0.23.4".to_string(),
            digest: None,
            enabled: None,
            shipping_policy: None,
        },
    );
    let err = validate_images_for_stage(&catalog, &["fastp", "bwa"])
        .err()
        .unwrap_or_else(|| panic!("expected missing tool error"));
    assert!(err.to_string().contains("bwa"));
}

#[test]
fn load_platform_prefers_cache_root_for_apptainer_platforms() -> anyhow::Result<()> {
    let _env = env_lock();
    let temp = bijux_dna_testkit::tempdir_for("environment-platform-cache-root");
    let _cwd = CurrentDirGuard::change_to(temp.path())?;
    let _cache_root = EnvVarGuard::capture("BIJUX_CACHE_ROOT");
    write_runtime_platform_fixture(
        temp.path(),
        br#"
default = "apptainer-amd64"

[platforms.apptainer-amd64]
runner = "apptainer"
container_dir = "containers/apptainer/sif"
image_prefix = "bijuxdna"
arch = "amd64"
"#,
    )?;
    std::env::set_var("BIJUX_CACHE_ROOT", "/var/tmp/bijux-cache-root");
    let platform = load_platform(Some("apptainer-amd64"))?;
    assert_eq!(platform.runner, RuntimeKind::Apptainer);
    assert_eq!(
        platform.container_dir,
        Path::new("/var/tmp/bijux-cache-root")
            .join("bijux-dna-container")
            .join("apptainer")
            .join("sif")
    );
    Ok(())
}

#[test]
fn load_platform_keeps_relative_apptainer_dir_without_cache_env() -> anyhow::Result<()> {
    let _env = env_lock();
    let temp = bijux_dna_testkit::tempdir_for("environment-platform-relative-apptainer");
    let _cwd = CurrentDirGuard::change_to(temp.path())?;
    let _cache_root = EnvVarGuard::capture("BIJUX_CACHE_ROOT");
    let _hpc_root = EnvVarGuard::capture("BIJUX_HPC_ROOT");
    let _apptainer_dir = EnvVarGuard::capture("BIJUX_APPTAINER_CONTAINER_DIR");
    write_runtime_platform_fixture(
        temp.path(),
        br#"
default = "apptainer-amd64"

[platforms.apptainer-amd64]
runner = "apptainer"
container_dir = "containers/apptainer/sif"
image_prefix = "bijuxdna"
arch = "amd64"
"#,
    )?;
    std::env::remove_var("BIJUX_CACHE_ROOT");
    std::env::remove_var("BIJUX_HPC_ROOT");
    std::env::remove_var("BIJUX_APPTAINER_CONTAINER_DIR");
    let platform = load_platform(Some("apptainer-amd64"))?;
    assert_eq!(platform.container_dir, Path::new("containers").join("apptainer").join("sif"));
    Ok(())
}

#[test]
fn load_image_catalog_hydrates_missing_digests_from_registry() -> anyhow::Result<()> {
    let _env = env_lock();
    let temp = bijux_dna_testkit::tempdir_for("environment-image-catalog");
    let _cwd = CurrentDirGuard::change_to(temp.path())?;
    write_image_catalog_fixture(
        temp.path(),
        b"[fastqc]\nversion = \"latest-pinned\"\n",
        br#"[[tools]]
id = "fastqc"
container_ref = "bijuxdna/fastqc@sha256:abc123"
"#,
    )?;
    let catalog = load_image_catalog()?;
    assert_eq!(
        catalog.get("fastqc").and_then(|spec| spec.digest.as_deref()),
        Some("sha256:abc123")
    );
    Ok(())
}

#[test]
fn cache_paths_and_docker_image_checks_are_deterministic() {
    let _env = env_lock();
    let image = ResolvedImage {
        full_name: "bijuxdna/fastp@sha256:abc123".to_string(),
        arch: "arm64".to_string(),
        runner: RuntimeKind::Apptainer,
    };
    let _cache_root = EnvVarGuard::capture("BIJUX_CACHE_ROOT");
    std::env::set_var("BIJUX_CACHE_ROOT", std::env::temp_dir().join("bijux_cache"));
    assert!(cache_dir(RuntimeKind::Docker).to_string_lossy().contains("bijux/docker/images"));
    assert!(apptainer_sif_path(&image).to_string_lossy().contains("fastp-sha256:abc123-arm64.sif"));

    let docker = ResolvedImage {
        full_name: "bijuxdna/fastp:0.23.4-arm64".to_string(),
        arch: "arm64".to_string(),
        runner: RuntimeKind::Docker,
    };
    assert!(!docker_image_exists(&docker));
}

#[test]
fn run_shell_capture_preserves_stdout_and_stderr() -> anyhow::Result<()> {
    let success = run_shell_capture("printf 'stdout\\n'; printf 'stderr\\n' >&2")?;
    assert!(success.contains("stdout"));
    assert!(success.contains("stderr"));

    let failure = run_shell_capture("printf 'stdout\\n'; printf 'stderr\\n' >&2; exit 7")
        .err()
        .unwrap_or_else(|| panic!("expected command failure"));
    let message = failure.to_string();
    assert!(message.contains("stdout"));
    assert!(message.contains("stderr"));
    Ok(())
}
