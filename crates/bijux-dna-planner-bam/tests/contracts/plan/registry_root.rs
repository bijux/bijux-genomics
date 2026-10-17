use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use bijux_dna_domain_bam::BamStage;
use bijux_dna_planner_bam::stage_api::{
    allowed_tools_for_stage, default_tool_for_stage, BamToolRegistryObservation,
};

fn parent_directory() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repository root")?;
    let path = root.join("artifacts/testing/bam-registry-roots");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn set_primary(path: &Path, tool: &str) -> Result<()> {
    let mut config: toml::Value = toml::from_str(&std::fs::read_to_string(path)?)?;
    let table = config.as_table_mut().context("registry table")?;
    let stages = table
        .entry("stages")
        .or_insert_with(|| toml::Value::Array(Vec::new()))
        .as_array_mut()
        .context("registry stages")?;
    stages.retain(|stage| stage.get("id").and_then(toml::Value::as_str) != Some("bam.align"));
    let row: toml::Value =
        toml::from_str(&format!("id = 'bam.align'\nprimary_tools = ['{tool}']\n"))?;
    stages.push(row);
    std::fs::write(path, toml::to_string(&config)?)?;
    Ok(())
}

fn repository(parent: &Path, primary: &str) -> Result<tempfile::TempDir> {
    let root = tempfile::Builder::new().prefix("bam-registry-authority-").tempdir_in(parent)?;
    std::fs::create_dir_all(root.path().join("crates/bijux-dna"))?;
    std::fs::create_dir_all(root.path().join("configs/ci/registry"))?;
    std::fs::write(root.path().join("Cargo.toml"), include_bytes!("../../../../../Cargo.toml"))?;
    std::fs::write(root.path().join("Cargo.lock"), include_bytes!("../../../../../Cargo.lock"))?;
    std::fs::write(
        root.path().join("crates/bijux-dna/Cargo.toml"),
        include_bytes!("../../../../bijux-dna/Cargo.toml"),
    )?;
    let registry = root.path().join("configs/ci/registry/tool_registry.toml");
    std::fs::write(
        &registry,
        include_bytes!("../../../../../configs/ci/registry/tool_registry.toml"),
    )?;
    set_primary(&registry, primary)?;
    Ok(root)
}

fn launch(case: &str, cwd: &Path, binding: Option<&std::ffi::OsStr>, expected: &str) -> Result<()> {
    let runtime = cwd.join("artifacts/test-runtime");
    std::fs::create_dir_all(&runtime)?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([case, "--exact", "--test-threads=1"])
        .current_dir(cwd)
        .env("BIJUX_BAM_REGISTRY_EXPECTED", expected)
        .envs([("HOME", &runtime), ("TMPDIR", &runtime), ("TMP", &runtime), ("TEMP", &runtime)]);
    if let Some(root) = binding {
        command.env("BIJUX_REPO_ROOT", root);
    } else {
        command.env_remove("BIJUX_REPO_ROOT");
    }
    let output = command.output()?;
    assert!(
        output.status.success(),
        "registry child failed: {}\n{}\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("1 passed; 0 failed"),
        "exact provider child must execute its selected case: {}",
        String::from_utf8_lossy(&output.stdout)
    );
    Ok(())
}

#[test]
fn bam_registry_uses_bound_primary_and_refreshes_owned_observations() -> Result<()> {
    if let Ok(primary) = std::env::var("BIJUX_BAM_REGISTRY_EXPECTED") {
        let choices = allowed_tools_for_stage(BamStage::Align);
        assert!(choices.iter().any(|tool| tool.as_str() == "bwa"));
        assert!(choices.iter().any(|tool| tool.as_str() == "bowtie2"));
        assert_eq!(default_tool_for_stage(BamStage::Align).as_str(), primary);
        let root = match std::env::var_os("BIJUX_REPO_ROOT") {
            Some(root) => PathBuf::from(root),
            None => std::env::current_dir()?,
        };
        let alternate = if primary == "bwa" { "bowtie2" } else { "bwa" };
        set_primary(&root.join("configs/ci/registry/tool_registry.toml"), alternate)?;
        assert_eq!(default_tool_for_stage(BamStage::Align).as_str(), alternate);
        return Ok(());
    }
    let parent = parent_directory()?;
    let alpha = repository(&parent, "bwa")?;
    let beta = repository(&parent, "bowtie2")?;
    let case = "mod_contracts_plan_rs::registry_root::bam_registry_uses_bound_primary_and_refreshes_owned_observations";
    launch(case, beta.path(), Some(alpha.path().as_os_str()), "bwa")?;
    set_primary(&alpha.path().join("configs/ci/registry/tool_registry.toml"), "bwa")?;
    launch(case, alpha.path(), Some(beta.path().as_os_str()), "bowtie2")?;
    set_primary(&alpha.path().join("configs/ci/registry/tool_registry.toml"), "bwa")?;
    launch(case, alpha.path(), None, "bwa")?;
    Ok(())
}

#[test]
fn bam_registry_refuses_invalid_bound_provider_without_domain_fallback() -> Result<()> {
    if std::env::var_os("BIJUX_BAM_REGISTRY_EXPECTED").is_some() {
        assert!(allowed_tools_for_stage(BamStage::Align).is_empty());
        assert!(std::panic::catch_unwind(|| default_tool_for_stage(BamStage::Align)).is_err());
        return Ok(());
    }
    let parent = parent_directory()?;
    let cwd = repository(&parent, "bwa")?;
    let case = "mod_contracts_plan_rs::registry_root::bam_registry_refuses_invalid_bound_provider_without_domain_fallback";
    let absent = parent.join("absent-bam-registry-root");
    launch(case, cwd.path(), Some(absent.as_os_str()), "invalid")?;
    launch(case, cwd.path(), Some(std::ffi::OsStr::new("")), "invalid")?;
    let foreign = repository(&parent, "bwa")?;
    std::fs::write(
        foreign.path().join("crates/bijux-dna/Cargo.toml"),
        "[package]\nname = 'unrelated'\n",
    )?;
    launch(case, cwd.path(), Some(foreign.path().as_os_str()), "invalid")?;
    for payload in [None, Some("not valid toml = ["), Some("tools = 'invalid schema'")] {
        let root = repository(&parent, "bwa")?;
        let registry = root.path().join("configs/ci/registry/tool_registry.toml");
        if let Some(content) = payload {
            std::fs::write(&registry, content)?;
        } else {
            std::fs::remove_file(&registry)?;
        }
        launch(case, cwd.path(), Some(root.path().as_os_str()), "invalid")?;
    }
    let root = repository(&parent, "bwa")?;
    let registry = root.path().join("configs/ci/registry/tool_registry.toml");
    std::fs::remove_file(&registry)?;
    std::fs::create_dir(&registry)?;
    launch(case, cwd.path(), Some(root.path().as_os_str()), "invalid")?;
    Ok(())
}

#[test]
fn bam_unbound_optional_registry_retains_domain_fallback() -> Result<()> {
    if std::env::var_os("BIJUX_BAM_REGISTRY_EXPECTED").is_some() {
        assert!(std::env::var_os("BIJUX_REPO_ROOT").is_none());
        let registry = std::env::current_dir()?.join("configs/ci/registry/tool_registry.toml");
        std::fs::write(&registry, "tools = []\n")?;
        let domain = allowed_tools_for_stage(BamStage::Align);
        let primary = domain.first().context("canonical domain candidate")?.as_str().to_string();
        for payload in [None, Some("not valid toml = [")] {
            if let Some(content) = payload {
                std::fs::write(&registry, content)?;
            } else {
                std::fs::remove_file(&registry)?;
            }
            assert_eq!(allowed_tools_for_stage(BamStage::Align), domain);
            assert_eq!(default_tool_for_stage(BamStage::Align).as_str(), primary);
        }
        std::fs::remove_file(&registry)?;
        std::fs::create_dir(&registry)?;
        assert_eq!(allowed_tools_for_stage(BamStage::Align), domain);
        assert_eq!(default_tool_for_stage(BamStage::Align).as_str(), primary);
        return Ok(());
    }
    let parent = parent_directory()?;
    let root = repository(&parent, "bwa")?;
    launch(
        "mod_contracts_plan_rs::registry_root::bam_unbound_optional_registry_retains_domain_fallback",
        root.path(), None, "unbound-optional-registry",
    )?;
    Ok(())
}

#[test]
fn owned_registry_observations_keep_opposite_roots_and_refresh_new_requests() -> Result<()> {
    let parent = parent_directory()?;
    let alpha = repository(&parent, "request_alpha_tool")?;
    let beta = repository(&parent, "request_beta_tool")?;
    let observed_alpha = BamToolRegistryObservation::new(alpha.path());
    let observed_beta = BamToolRegistryObservation::new(beta.path());
    let alpha_tools = observed_alpha.allowed_tools_for_stage(BamStage::Align)?;
    let beta_tools = observed_beta.allowed_tools_for_stage(BamStage::Align)?;
    assert!(alpha_tools.iter().any(|tool| tool.as_str() == "request_alpha_tool"));
    assert!(!alpha_tools.iter().any(|tool| tool.as_str() == "request_beta_tool"));
    assert!(beta_tools.iter().any(|tool| tool.as_str() == "request_beta_tool"));
    assert!(!beta_tools.iter().any(|tool| tool.as_str() == "request_alpha_tool"));
    assert!(alpha_tools.windows(2).all(|pair| pair[0].as_str() < pair[1].as_str()));
    assert!(alpha_tools.iter().any(|tool| tool.as_str() == "bwa"));
    assert!(alpha_tools.iter().any(|tool| tool.as_str() == "bowtie2"));

    set_primary(
        &alpha.path().join("configs/ci/registry/tool_registry.toml"),
        "request_changed_tool",
    )?;
    assert_eq!(observed_alpha.allowed_tools_for_stage(BamStage::Align)?, alpha_tools);
    let fresh =
        BamToolRegistryObservation::new(alpha.path()).allowed_tools_for_stage(BamStage::Align)?;
    assert!(fresh.iter().any(|tool| tool.as_str() == "request_changed_tool"));
    assert!(!fresh.iter().any(|tool| tool.as_str() == "request_alpha_tool"));
    assert_eq!(observed_beta.allowed_tools_for_stage(BamStage::Align)?, beta_tools);
    Ok(())
}

#[test]
fn admitted_registry_snapshot_survives_source_removal_while_fresh_requests_refuse() -> Result<()> {
    let parent = parent_directory()?;
    let root = repository(&parent, "bwa")?;
    let observed = BamToolRegistryObservation::new(root.path());
    let admitted = observed.allowed_tools_for_stage(BamStage::Align)?;
    for path in
        ["Cargo.toml", "crates/bijux-dna/Cargo.toml", "configs/ci/registry/tool_registry.toml"]
    {
        std::fs::remove_file(root.path().join(path))?;
    }
    assert_eq!(observed.allowed_tools_for_stage(BamStage::Align)?, admitted);
    assert!(!observed.allowed_tools_for_stage(BamStage::QcPre)?.is_empty());
    let error = BamToolRegistryObservation::new(root.path())
        .allowed_tools_for_stage(BamStage::Align)
        .err()
        .context("new request accepted removed complete source authority")?;
    assert!(format!("{error:#}").contains("not a Bijux Genomics repository root"));
    Ok(())
}

#[test]
fn failed_registry_demand_remains_lazy_and_retryable_after_source_repair() -> Result<()> {
    let parent = parent_directory()?;
    let root = repository(&parent, "bwa")?;
    let registry = root.path().join("configs/ci/registry/tool_registry.toml");
    let original = std::fs::read(&registry)?;
    let observed = BamToolRegistryObservation::new(root.path());
    std::fs::write(&registry, "tools = 'invalid schema'\n")?;
    let error = observed
        .allowed_tools_for_stage(BamStage::Align)
        .err()
        .context("constructor acquired a previous registry observation")?;
    assert!(error.to_string().contains("must declare tools"));
    std::fs::write(&registry, "not valid toml = [")?;
    let error = observed
        .allowed_tools_for_stage(BamStage::Align)
        .err()
        .context("failed registry parse incorrectly used a successful cached observation")?;
    assert!(format!("{error:#}").contains("parse BAM tool registry"));
    std::fs::write(&registry, original)?;
    let tools = observed.allowed_tools_for_stage(BamStage::Align)?;
    assert!(tools.iter().any(|tool| tool.as_str() == "bwa"));
    Ok(())
}

#[test]
fn explicit_registry_observation_requires_complete_valid_root_and_registry() -> Result<()> {
    let parent = parent_directory()?;
    let absent = parent.join("absent-owned-registry-root");
    for path in [Path::new(""), absent.as_path()] {
        assert!(BamToolRegistryObservation::new(path)
            .allowed_tools_for_stage(BamStage::Align)
            .is_err());
    }
    for (relative, replacement, rejection) in [
        (
            "Cargo.toml",
            "[package]\nname = 'unrelated'\n",
            "repository root must declare a Cargo workspace",
        ),
        ("Cargo.toml", "not valid toml = [", "parse"),
        (
            "crates/bijux-dna/Cargo.toml",
            "[package]\nname = 'unrelated'\n",
            "repository root must contain the bijux-dna package",
        ),
        ("crates/bijux-dna/Cargo.toml", "not valid toml = [", "parse"),
        (
            "configs/ci/registry/tool_registry.toml",
            "tools = 'invalid schema'\n",
            "must declare tools",
        ),
        ("configs/ci/registry/tool_registry.toml", "not valid toml = [", "parse BAM tool registry"),
    ] {
        let root = repository(&parent, "bwa")?;
        std::fs::write(root.path().join(relative), replacement)?;
        let error = BamToolRegistryObservation::new(root.path())
            .allowed_tools_for_stage(BamStage::Align)
            .err()
            .context("invalid declared source used ambient domain fallback")?;
        assert!(format!("{error:#}").contains(rejection), "{relative}: {error:#}");
    }
    for directory in [false, true] {
        let root = repository(&parent, "bwa")?;
        let registry = root.path().join("configs/ci/registry/tool_registry.toml");
        std::fs::remove_file(&registry)?;
        if directory {
            std::fs::create_dir(&registry)?;
        }
        let error = BamToolRegistryObservation::new(root.path())
            .allowed_tools_for_stage(BamStage::Align)
            .err()
            .context("explicit missing or directory registry used domain fallback")?;
        assert!(format!("{error:#}").contains("read BAM tool registry"));
    }
    Ok(())
}
