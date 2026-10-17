use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result};
use bijux_dna_db_ref::{resolve_map, resolve_map_lock, resolve_panel, resolve_panel_lock};

const CONFIGS: [(&str, &str); 4] = [
    ("configs/vcf/panels/panels.toml", include_str!("../../../../configs/vcf/panels/panels.toml")),
    (
        "configs/vcf/panels/locks/panel_locks.toml",
        include_str!("../../../../configs/vcf/panels/locks/panel_locks.toml"),
    ),
    ("configs/vcf/maps/maps.toml", include_str!("../../../../configs/vcf/maps/maps.toml")),
    (
        "configs/vcf/maps/locks/map_locks.toml",
        include_str!("../../../../configs/vcf/maps/locks/map_locks.toml"),
    ),
];

fn parent_directory() -> Result<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .context("repository root")?;
    let path = root.join("artifacts/testing/reference-provider-roots");
    std::fs::create_dir_all(&path)?;
    Ok(path)
}

fn repository(parent: &Path, version: &str) -> Result<tempfile::TempDir> {
    let root = tempfile::Builder::new().prefix("reference-authority-").tempdir_in(parent)?;
    std::fs::create_dir_all(root.path().join("crates/bijux-dna"))?;
    std::fs::write(root.path().join("Cargo.toml"), include_bytes!("../../../../Cargo.toml"))?;
    std::fs::write(root.path().join("Cargo.lock"), include_bytes!("../../../../Cargo.lock"))?;
    std::fs::write(
        root.path().join("crates/bijux-dna/Cargo.toml"),
        include_bytes!("../../../bijux-dna/Cargo.toml"),
    )?;
    for (name, content) in CONFIGS {
        let mut config: toml::Value = toml::from_str(content)?;
        if let Some(rows) = config.get_mut("panel").and_then(toml::Value::as_array_mut) {
            rows.first_mut()
                .context("catalog row")?
                .as_table_mut()
                .context("catalog entry")?
                .insert("version".to_string(), toml::Value::String(version.to_string()));
        }
        if let Some(rows) = config.get_mut("map").and_then(toml::Value::as_array_mut) {
            rows.first_mut()
                .context("catalog row")?
                .as_table_mut()
                .context("catalog entry")?
                .insert("version".to_string(), toml::Value::String(version.to_string()));
        }
        if let Some(locks) = config.get_mut("locks").and_then(toml::Value::as_table_mut) {
            for (_, lock) in locks.iter_mut() {
                lock.as_table_mut()
                    .context("lock entry")?
                    .insert("version".to_string(), toml::Value::String(version.to_string()));
            }
        }
        let path = root.path().join(name);
        std::fs::create_dir_all(path.parent().context("config parent")?)?;
        std::fs::write(path, toml::to_string(&config)?)?;
    }
    Ok(root)
}

fn launch(case: &str, cwd: &Path, binding: Option<&std::ffi::OsStr>, expected: &str) -> Result<()> {
    let runtime = cwd.join("artifacts/test-runtime");
    std::fs::create_dir_all(&runtime)?;
    let mut command = Command::new(std::env::current_exe()?);
    command
        .args([case, "--exact", "--test-threads=1"])
        .current_dir(cwd)
        .env("BIJUX_REFERENCE_PROVIDER_EXPECTED", expected)
        .envs([("HOME", &runtime), ("TMPDIR", &runtime), ("TMP", &runtime), ("TEMP", &runtime)]);
    if let Some(root) = binding {
        command.env("BIJUX_REPO_ROOT", root);
    } else {
        command.env_remove("BIJUX_REPO_ROOT");
    }
    let output = command.output()?;
    assert!(
        output.status.success(),
        "provider child failed: {}\n{}\n{}",
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
fn reference_provider_uses_private_catalog_and_lock_authority() -> Result<()> {
    if let Ok(version) = std::env::var("BIJUX_REFERENCE_PROVIDER_EXPECTED") {
        let panel = resolve_panel("Homo sapiens", "GRCh38", None)?;
        let map = resolve_map("Homo sapiens", "GRCh38", None)?;
        assert_eq!(panel.version, version);
        assert_eq!(resolve_panel_lock(&panel)?.version, version);
        assert_eq!(map.version, version);
        assert_eq!(resolve_map_lock(&map)?.version, version);
        assert!(!panel.files.is_empty());
        assert!(!map.files.is_empty());
        return Ok(());
    }
    let parent = parent_directory()?;
    let alpha = repository(&parent, "authority-alpha")?;
    let beta = repository(&parent, "authority-beta")?;
    let case = "runtime_root::reference_provider_uses_private_catalog_and_lock_authority";
    launch(case, beta.path(), Some(alpha.path().as_os_str()), "authority-alpha")?;
    launch(case, alpha.path(), Some(beta.path().as_os_str()), "authority-beta")?;
    launch(case, alpha.path(), None, "authority-alpha")?;
    Ok(())
}

#[test]
fn reference_provider_refuses_missing_or_invalid_runtime_authority() -> Result<()> {
    if let Ok(provider) = std::env::var("BIJUX_REFERENCE_PROVIDER_EXPECTED") {
        match provider.as_str() {
            "panel" => assert!(resolve_panel("Homo sapiens", "GRCh38", None).is_err()),
            "map" => assert!(resolve_map("Homo sapiens", "GRCh38", None).is_err()),
            _ => {
                assert!(resolve_panel("Homo sapiens", "GRCh38", None).is_err());
                assert!(resolve_map("Homo sapiens", "GRCh38", None).is_err());
            }
        }
        return Ok(());
    }
    let parent = parent_directory()?;
    let cwd = repository(&parent, "authority-cwd")?;
    let case = "runtime_root::reference_provider_refuses_missing_or_invalid_runtime_authority";
    let absent = parent.join("absent-reference-root");
    launch(case, cwd.path(), Some(absent.as_os_str()), "invalid")?;
    launch(case, cwd.path(), Some(std::ffi::OsStr::new("")), "invalid")?;
    let foreign = repository(&parent, "authority-foreign")?;
    std::fs::write(
        foreign.path().join("crates/bijux-dna/Cargo.toml"),
        "[package]\nname = 'unrelated'\n",
    )?;
    launch(case, cwd.path(), Some(foreign.path().as_os_str()), "invalid")?;
    for (relative, provider) in [
        ("configs/vcf/panels/panels.toml", "panel"),
        ("configs/vcf/panels/locks/panel_locks.toml", "panel"),
        ("configs/vcf/maps/maps.toml", "map"),
        ("configs/vcf/maps/locks/map_locks.toml", "map"),
    ] {
        let root = repository(&parent, "authority-missing")?;
        std::fs::remove_file(root.path().join(relative))?;
        launch(case, cwd.path(), Some(root.path().as_os_str()), provider)?;
        std::fs::create_dir(root.path().join(relative))?;
        launch(case, cwd.path(), Some(root.path().as_os_str()), provider)?;
    }
    Ok(())
}
