#![cfg(test)]
use crate::execution::output::prepare_output_root;
use anyhow::Result;
use serde_json::json;
use std::path::{Path, PathBuf};

fn report(repository: &Path, root: &Path, outputs: &serde_json::Value) -> Result<PathBuf> {
    let path = root.join("summary.json");
    bijux_dna_infra::atomic_write_json(
        &path,
        &json!({
            "schema_version": "workflow.v1", "pipeline_id": "test-workflow",
            "output_path": path.strip_prefix(repository)?.display().to_string(),
            "rows": [{"outputs": outputs, "evidence_path": null}]
        }),
    )?;
    Ok(path)
}

#[test]
fn unrelated_files_survive_first_admission_and_declared_output_retirement() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    bijux_dna_infra::ensure_dir(&root)?;
    let notes = root.join("notes.txt");
    std::fs::write(&notes, "user notes")?;
    let summary = root.join("summary.json");
    prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1")?;
    let generated = root.join("artifacts/stage/result.json");
    let extra = root.join("artifacts/stage/research.txt");
    bijux_dna_infra::atomic_write_bytes(&generated, b"generated")?;
    std::fs::write(&extra, "research")?;
    report(&repository, &root, &json!({"result": "output/artifacts/stage/result.json"}))?;
    prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1")?;
    assert!(!generated.exists());
    assert!(!summary.exists());
    assert_eq!(std::fs::read_to_string(notes)?, "user notes");
    assert_eq!(std::fs::read_to_string(extra)?, "research");
    Ok(())
}

#[test]
fn every_declaration_is_admitted_before_any_retirement() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    let generated = root.join("artifacts/result.json");
    bijux_dna_infra::atomic_write_bytes(&generated, b"previous")?;
    let summary = report(
        &repository,
        &root,
        &json!({"a": "output/artifacts/result.json", "z": "output/../source.txt"}),
    )?;
    assert!(
        prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1").is_err()
    );
    assert_eq!(std::fs::read(&generated)?, b"previous");
    assert!(summary.is_file());
    assert!(prepare_output_root(&repository, &root, &summary, "different", "workflow.v1").is_err());
    assert_eq!(std::fs::read(&generated)?, b"previous");
    Ok(())
}

#[test]
fn unclaimed_artifacts_and_repository_root_are_refused() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    let unclaimed = root.join("artifacts/unclaimed.txt");
    bijux_dna_infra::atomic_write_bytes(&unclaimed, b"keep")?;
    assert!(prepare_output_root(
        &repository,
        &root,
        &root.join("summary.json"),
        "test-workflow",
        "workflow.v1"
    )
    .is_err());
    assert!(prepare_output_root(
        &repository,
        &repository,
        &repository.join("summary.json"),
        "test-workflow",
        "workflow.v1"
    )
    .is_err());
    assert_eq!(std::fs::read(unclaimed)?, b"keep");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symbolic_output_declarations_do_not_retire_external_files() -> Result<()> {
    let temp = tempfile::tempdir()?;
    let repository = temp.path().canonicalize()?;
    let root = repository.join("output");
    bijux_dna_infra::ensure_dir(root.join("artifacts"))?;
    let external = repository.join("source.txt");
    std::fs::write(&external, "source")?;
    std::os::unix::fs::symlink(&external, root.join("artifacts/alias.txt"))?;
    let summary = report(&repository, &root, &json!({"alias": "output/artifacts/alias.txt"}))?;
    assert!(
        prepare_output_root(&repository, &root, &summary, "test-workflow", "workflow.v1").is_err()
    );
    assert_eq!(std::fs::read_to_string(external)?, "source");
    assert!(summary.is_file());
    Ok(())
}

#[test]
fn selected_amplicon_output_parent_preserves_predecessor_handoffs() -> Result<()> {
    use anyhow::Context;
    fn copy_fixture(source: &Path, destination: &Path) -> Result<()> {
        let metadata = source.symlink_metadata()?;
        anyhow::ensure!(!metadata.file_type().is_symlink(), "fixture source must not be an alias");
        if metadata.is_dir() {
            std::fs::create_dir_all(destination)?;
            for entry in std::fs::read_dir(source)? {
                let entry = entry?;
                copy_fixture(&entry.path(), &destination.join(entry.file_name()))?;
            }
        } else {
            anyhow::ensure!(metadata.is_file(), "fixture source must be regular");
            if let Some(parent) = destination.parent() {
                std::fs::create_dir_all(parent)?;
            }
            std::fs::copy(source, destination)?;
        }
        Ok(())
    }
    let source = std::env::var_os("BIJUX_REPO_ROOT")
        .map_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."), PathBuf::from);
    anyhow::ensure!(source.join("Cargo.lock").is_file(), "declare runtime repository fixture root");
    let mut observations = Vec::new();
    for (selected, relative) in [
        (false, crate::amplicon::DEFAULT_AMPLICON_MICRO_PIPELINE_PATH),
        (true, "selected-amplicon/summary.json"),
    ] {
        let root = tempfile::tempdir()?;
        for fixture in [
            "benchmarks/tests/fixtures/corpora/corpus-03-amplicon-mini",
            "benchmarks/tests/fixtures/science/amplicon-truth",
            "assets/reference/amplicon_governance.toml",
            "assets/reference/primers/16S_universal_v1.fasta",
            "assets/toy/core-v1/tables/otu_abundance_small.tsv",
            "assets/toy/core-v1/fastq/reads_with_primers.fastq",
            "assets/toy/corpus-03-amplicon-mini/fastq/merged_amplicon_reads.fastq",
        ] {
            copy_fixture(&source.join(fixture), &root.path().join(fixture))?;
        }
        let report = crate::amplicon::render_amplicon_micro_pipeline(root.path(), relative.into())?;
        let judgment = report
            .rows
            .iter()
            .find(|row| row.stage_id == "benchmark.amplicon_output_judgment")
            .context("actual amplicon output judgment")?;
        let default_exists =
            root.path().join("artifacts/benchmark-smoke/micro/pipelines/amplicon").exists();
        let truth_valid = judgment.metrics["valid"].clone();
        observations.push((selected, report, truth_valid, default_exists));
    }
    for (selected, report, truth_valid, default_exists) in observations {
        assert_eq!(truth_valid, serde_json::json!(true), "actual truth control must be valid");
        assert!(
            report.handoffs.iter().all(|handoff| handoff.accepted),
            "caller-selected parent must preserve declared predecessor identities: {selected}"
        );
        assert!(report.passes_behavior_test);
        if selected {
            assert!(!default_exists, "selected output must not depend on prior default artifacts");
        }
    }
    Ok(())
}
