use anyhow::Result;
use bijux_dna_bench_stage::{
    execution::{
        build_local_stage_command_entry, materialize_declared_output, materialize_plan_fixtures,
        publish_stage_commands, StagePlanBundle,
    },
    BenchmarkLocalStageReadinessKind,
};
use bijux_dna_stage_contract::StagePlanV1;
use std::{collections::BTreeMap, fs, path::Path};

fn stage_plan() -> StagePlanV1 {
    StagePlanV1 {
        stage_id: bijux_dna_core::ids::StageId::new("fastq.trim_reads"),
        stage_instance_id: None,
        stage_version: bijux_dna_core::ids::StageVersion(1),
        tool_id: bijux_dna_core::ids::ToolId::new("fastp"),
        tool_version: "1.0".to_string(),
        image: bijux_dna_core::prelude::ContainerImageRefV1 {
            image: "fastp".to_string(),
            digest: None,
        },
        command: bijux_dna_core::prelude::CommandSpecV1 { template: vec!["fastp".to_string()] },
        resources: bijux_dna_core::contract::ToolConstraints::default(),
        io: bijux_dna_core::contract::StageIO {
            inputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_in"),
                "reads.fq".into(),
                bijux_dna_core::contract::ArtifactRole::Reads,
            )],
            outputs: vec![bijux_dna_core::contract::ArtifactRef::required(
                bijux_dna_core::ids::ArtifactId::new("reads_out"),
                "reads.trimmed.fq".into(),
                bijux_dna_core::contract::ArtifactRole::TrimmedReads,
            )],
        },
        out_dir: "out/fastq.trim_reads".into(),
        params: serde_json::json!({"quality": 20}),
        effective_params: serde_json::json!({"quality": 20}),
        aux_images: BTreeMap::new(),
        operating_mode: bijux_dna_core::contract::StageOperatingMode::Enforced,
        canonical_contract: None,
        provenance: None,
        reason: bijux_dna_stage_contract::PlanDecisionReason::default(),
    }
}

fn command(root: &Path) -> Result<bijux_dna_bench_stage::BenchmarkLocalStageCommandEntry> {
    build_local_stage_command_entry(
        root,
        StagePlanBundle {
            stage_id: "fastq.trim_reads".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
            plans: vec![stage_plan()],
        },
    )
}

#[test]
fn memory_overflow_is_rejected_without_panicking() -> Result<()> {
    let mut plan = stage_plan();
    plan.resources.mem_gb = u32::MAX;
    let error = build_local_stage_command_entry(
        Path::new("."),
        StagePlanBundle {
            stage_id: "fastq.trim_reads".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
            plans: vec![plan],
        },
    )
    .err()
    .ok_or_else(|| anyhow::anyhow!("memory overflow accepted"))?;
    assert!(error.to_string().contains("MiB range"));
    Ok(())
}

#[test]
fn resource_disagreement_and_missing_plans_refuse_projection() {
    let plan = stage_plan();
    let mut other = plan.clone();
    other.resources.threads = plan.resources.threads.max(1) + 1;
    for plans in [vec![], vec![plan, other]] {
        assert!(build_local_stage_command_entry(
            Path::new("."),
            StagePlanBundle {
                stage_id: "fastq.trim_reads".to_string(),
                readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
                plans
            }
        )
        .is_err());
    }
}

#[test]
fn invalid_command_and_conflicting_destinations_preserve_prior_publication() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = fs::canonicalize(dir.path())?;
    let path = root.join("commands.sh");
    fs::write(&path, "retained script")?;
    let mut bad = command(&root)?;
    bad.memory_mb = 0;
    assert!(publish_stage_commands(&root, &path, vec![bad]).is_err());
    assert_eq!(fs::read_to_string(&path)?, "retained script");
    fs::create_dir(root.join("commands.argv.jsonl"))?;
    assert!(publish_stage_commands(&root, &path, vec![command(&root)?]).is_err());
    assert_eq!(fs::read_to_string(&path)?, "retained script");
    assert!(!root.join("commands.json").exists());
    Ok(())
}

#[test]
fn synthetic_fixtures_preserve_foreign_bytes_and_conflicting_kinds() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = fs::canonicalize(dir.path())?;
    let output = root.join("native.json");
    fs::write(&output, b"native evidence")?;
    assert!(materialize_declared_output(&output).is_err());
    assert_eq!(fs::read(&output)?, b"native evidence");
    let directory = root.join("other.json");
    fs::create_dir(&directory)?;
    fs::write(directory.join("retained"), b"evidence")?;
    assert!(materialize_declared_output(&directory).is_err());
    assert_eq!(fs::read(directory.join("retained"))?, b"evidence");
    let no_extension = root.join("index");
    fs::write(&no_extension, b"retained")?;
    assert!(materialize_declared_output(&no_extension).is_err());
    assert_eq!(fs::read(&no_extension)?, b"retained");
    let fresh = root.join("fresh.json");
    materialize_declared_output(&fresh)?;
    materialize_declared_output(&fresh)?;
    assert_eq!(fs::read(&fresh)?, b"{}\n");
    Ok(())
}

#[test]
fn conflicting_plan_output_refuses_before_plan_publication() -> Result<()> {
    let dir = tempfile::tempdir()?;
    let root = fs::canonicalize(dir.path())?;
    let plan = stage_plan();
    fs::write(root.join("reads.trimmed.fq"), b"native reads")?;
    assert!(materialize_plan_fixtures(&root, "fastq.trim_reads", &[plan]).is_err());
    assert!(!root.join("out").exists());
    assert_eq!(fs::read(root.join("reads.trimmed.fq"))?, b"native reads");
    Ok(())
}

#[cfg(unix)]
#[test]
fn nested_script_runs_from_its_repository_after_relocation() -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let dir = tempfile::tempdir()?;
    let root = fs::canonicalize(dir.path())?.join("repository with spaces");
    fs::create_dir(&root)?;
    let output = Path::new("artifacts/benchmark-smoke/readiness/local ready/rendered.sh");
    let report = publish_stage_commands(&root, output, vec![command(&root)?])?;
    let moved = root.with_file_name("relocated repository");
    fs::rename(&root, &moved)?;
    let bin = moved.join("mock bin");
    fs::create_dir(&bin)?;
    let cargo = bin.join("cargo");
    fs::write(&cargo, b"#!/bin/sh\npwd > \"$CAPTURE\"\nprintf '%s\\n' \"$@\" >> \"$CAPTURE\"\n")?;
    fs::set_permissions(&cargo, fs::Permissions::from_mode(0o755))?;
    let capture = moved.join("observed");
    let path = std::env::join_paths(
        std::iter::once(bin)
            .chain(std::env::split_paths(&std::env::var_os("PATH").unwrap_or_default())),
    )?;
    let result = std::process::Command::new("bash")
        .arg(moved.join(&report.script_output_path))
        .current_dir(dir.path())
        .env("PATH", path)
        .env("CAPTURE", &capture)
        .output()?;
    assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
    let observed = fs::read_to_string(&capture)?;
    let lines = observed.lines().collect::<Vec<_>>();
    assert_eq!(lines[0], moved.to_str().ok_or_else(|| anyhow::anyhow!("non-UTF-8 test path"))?);
    assert_eq!(
        &lines[1..],
        report.commands[0].argv[1..].iter().map(String::as_str).collect::<Vec<_>>()
    );
    Ok(())
}

#[cfg(unix)]
#[test]
fn synthetic_fixture_symlinks_preserve_their_targets() -> Result<()> {
    use std::os::unix::fs::symlink;
    let dir = tempfile::tempdir()?;
    let root = fs::canonicalize(dir.path())?;
    let target = root.join("target.json");
    fs::write(&target, b"evidence")?;
    let link = root.join("link.json");
    symlink(&target, &link)?;
    assert!(materialize_declared_output(&link).is_err());
    assert_eq!(fs::read(&target)?, b"evidence");
    Ok(())
}
