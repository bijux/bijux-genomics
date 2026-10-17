//! One genuine CI-profile archive producer with source and physical input custody.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::{json, Value};

use crate::application::ci::ExpectedTestInventory;
use crate::runtime::compilation_cache::CompilationCache;
use crate::runtime::compiled_products::discover_native_inventory;
use crate::runtime::workspace::Workspace;

use super::super::archive::{
    archive_reuse_inputs, capture_archive_executable_seal, full_archive_args, full_archive_env,
    full_archive_extraction_args, prepare_full_archive_roots, seal_full_archive_extraction,
    set_full_archive_readonly,
};
use super::super::storage::FullArchiveRoots;
use super::protocol::{extracted_inputs, Producer, SCHEMA};
use super::{command, protect, write_json_pretty, Session};

pub(super) fn produce(
    workspace: &Workspace,
    root: &Path,
    reports: &Path,
    session: &mut Session,
    receipt: &mut Value,
) -> Result<()> {
    ensure!(!reports.join("producer.json").try_exists()?, "producer output already exists");
    ensure!(!session.scratch.join("run").try_exists()?, "producer scratch is not fresh");
    // Cargo may replace the running bootstrap executable; seal it before any build.
    publish_runner(reports)?;
    let lease = CompilationCache::acquire(&session.scratch)?;
    let roots = FullArchiveRoots {
        artifact_root: session.scratch.join("run/archive"),
        archive_extract: session.scratch.join("run/extracted"),
        cargo_target: lease.target().to_path_buf(),
        archive_file: reports.join("archive.tar.zst"),
        build_log: reports.join("archive-build.log"),
    };
    prepare_full_archive_roots(&roots)?;
    let admission = session.admission.as_ref().context("producer admission absent")?;
    let detected = std::thread::available_parallelism()?.get();
    ensure!(detected >= 2, "distributed producer requires two native listing workers");
    let control = std::env::var("CARGO_BUILD_JOBS").ok();
    let jobs = crate::commands::ops::tooling::ci::full_execution::admit_compiler_jobs(
        admission.contract.archive_build_jobs.min(detected),
        detected,
        true,
        control.as_deref(),
    )?
    .min(detected);
    let envs = full_archive_env(&roots, jobs)?;
    super::command_recorded(
        workspace,
        reports,
        "doctests",
        vec![
            "test".to_owned(),
            "--workspace".to_owned(),
            "--all-features".to_owned(),
            "--locked".to_owned(),
            "--doc".to_owned(),
            "--profile".to_owned(),
            "ci".to_owned(),
            "--target-dir".to_owned(),
            lease.target().display().to_string(),
            "--".to_owned(),
            "--test-threads=2".to_owned(),
        ],
        envs.clone(),
        receipt,
    )?;
    let mut args = full_archive_args(&roots.archive_file);
    args.extend(["--cargo-profile".to_owned(), "ci".to_owned()]);
    command(workspace, reports, "archive-build", args, envs.clone())?;
    set_full_archive_readonly(&roots.archive_file)?;
    super::super::archive::prepare_full_archive_extraction(&roots.archive_extract)?;
    command(
        workspace,
        reports,
        "archive-extract",
        full_archive_extraction_args(&roots),
        envs.clone(),
    )?;
    seal_full_archive_extraction(&roots.archive_extract)?;
    let mut seals = capture_archive_executable_seal(&roots, &workspace.root)?;
    seals.extracted.require_readonly()?;
    receipt["producer_input_manifest"] =
        seals.producer.publish_manifest(&reports.join("producer-inputs.json"))?;
    receipt["extracted_input_manifest"] =
        seals.extracted.publish_manifest(&reports.join("extracted-inputs.json"))?;
    let reuse = archive_reuse_inputs(&roots.archive_extract)?;
    let (inventory, discovery) = discover_native_inventory(
        workspace,
        &workspace.root,
        &admission.condition,
        &reuse,
        &envs,
        reports,
        detected,
    )?;
    ExpectedTestInventory::parse(&inventory)?;
    std::fs::write(reports.join("inventory.json"), &inventory)?;
    write_json_pretty(&reports.join("matrix.json"), &serde_json::to_value(&admission.matrix)?)?;
    let source = admission.condition.descriptor();
    let producer = Producer {
        schema_version: SCHEMA.to_owned(),
        source_commit: admission.snapshot.commit().to_owned(),
        source,
        compatibility: admission.compatibility.clone(),
        archive_sha256: bijux_dna_infra::hash_file_sha256(&roots.archive_file)?,
        inventory_sha256: bijux_dna_infra::hash_bytes_sha256(&inventory),
        matrix_sha256: bijux_dna_infra::hash_file_sha256(&reports.join("matrix.json"))?,
        runner_sha256: bijux_dna_infra::hash_file_sha256(&reports.join("bijux-dna-dev"))?,
        source_root: workspace.root.clone(),
        target_root: roots.archive_extract.join("target"),
        inputs: extracted_inputs(&roots.archive_extract.join("target"))?,
        matrix: admission.matrix.clone(),
    };
    let observed_producer = seals.producer.verify();
    let observed_extracted = seals.extracted.verify();
    receipt["producer_input_postcondition"] = observed_producer.receipt;
    receipt["extracted_input_postcondition"] = observed_extracted.receipt;
    ensure!(
        observed_producer.unchanged && observed_extracted.unchanged,
        "producer or extracted inputs changed during preparation"
    );
    write_json_pretty(&reports.join("producer.json"), &serde_json::to_value(&producer)?)?;
    let sha = bijux_dna_infra::hash_file_sha256(&reports.join("producer.json"))?;
    receipt["producer_sha256"] = json!(sha);
    receipt["inventory_discovery_milliseconds"] = json!(discovery.as_millis());
    receipt["archive_sha256"] = json!(producer.archive_sha256);
    receipt["compiler_jobs"] = json!(jobs);
    protect(
        session,
        ["producer.json", "archive.tar.zst", "inventory.json", "matrix.json", "bijux-dna-dev"]
            .map(|name| root.join("build").join(name)),
    )?;
    receipt["all_passed"] = json!(true);
    Ok(())
}

fn publish_runner(reports: &Path) -> Result<()> {
    let current = std::env::current_exe()?.canonicalize()?;
    let copied = reports.join("bijux-dna-dev");
    ensure!(!copied.try_exists()?, "runner output is not fresh");
    std::fs::copy(&current, &copied)?;
    ensure!(
        bijux_dna_infra::hash_file_sha256(&current)? == bijux_dna_infra::hash_file_sha256(&copied)?,
        "runner copy differs from producer"
    );
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&copied, std::fs::Permissions::from_mode(0o555))?;
    }
    std::fs::write(
        reports.join("runner.sha256"),
        format!("{}  bijux-dna-dev\n", bijux_dna_infra::hash_file_sha256(&copied)?),
    )?;
    Ok(())
}
