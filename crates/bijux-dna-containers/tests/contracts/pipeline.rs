use std::path::Path;

use anyhow::{Context, Result};
use bijux_dna_containers::api::{
    container_pipeline_v1_schema, container_pipeline_v2_schema, load_container_pipeline,
    ContainerPipelineContract,
};

fn repository_root() -> Result<std::path::PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize()?)
}

#[test]
fn tracked_pipeline_schemas_match_rust_authority() -> Result<()> {
    let root = repository_root()?;
    for (path, expected) in [
        ("containers/contracts/config-pipeline.schema.json", container_pipeline_v1_schema()),
        ("containers/contracts/config-pipeline-v2.schema.json", container_pipeline_v2_schema()),
    ] {
        let checked: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(root.join(path))?)?;
        assert_eq!(checked, expected, "tracked schema drifted: {path}");
    }
    Ok(())
}

#[test]
fn every_tracked_pipeline_is_admitted_by_its_exact_schema() -> Result<()> {
    let runs = repository_root()?.join("containers/runs");
    let mut legacy = 0;
    let mut current = 0;
    for entry in std::fs::read_dir(runs)? {
        let path = entry?.path().join("config-pipeline.yml");
        if !path.is_file() {
            continue;
        }
        match load_container_pipeline(&path)? {
            ContainerPipelineContract::Legacy(_) => legacy += 1,
            ContainerPipelineContract::Current(_) => current += 1,
        }
    }
    assert_eq!(legacy, 61);
    assert_eq!(current, 10);
    Ok(())
}

#[test]
fn version_specific_fields_fail_closed() -> Result<()> {
    let root = tempfile::tempdir()?;
    let source = std::fs::read_to_string(
        repository_root()?.join("containers/runs/run-1004-bwa/config-pipeline.yml"),
    )?;
    let legacy = root.path().join("legacy.yml");
    std::fs::write(
        &legacy,
        source.replace(
            "  operator: bijan\n",
            "  operator: bijan\n  submitted_at_utc: '2026-08-07T00:00:00Z'\n",
        ),
    )?;
    let error = load_container_pipeline(&legacy)
        .err()
        .context("legacy pipeline accepted a current-only field")?;
    assert!(format!("{error:#}").contains("unknown field `submitted_at_utc`"));

    let source = std::fs::read_to_string(
        repository_root()?.join("containers/runs/run-1124-trim_galore/config-pipeline.yml"),
    )?;
    let current = root.path().join("current.yml");
    let without_mount = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("mount_path:"))
        .collect::<Vec<_>>()
        .join("\n");
    std::fs::write(&current, format!("{without_mount}\n"))?;
    let error = load_container_pipeline(&current)
        .err()
        .context("current pipeline accepted inputs without mount paths")?;
    assert!(format!("{error:#}").contains("missing field `mount_path`"));
    Ok(())
}
