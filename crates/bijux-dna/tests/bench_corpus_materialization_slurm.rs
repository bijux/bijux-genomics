#![recursion_limit = "256"]
#![cfg(unix)]

use std::fs;
use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

#[test]
fn prepare_and_submit_recover_across_cli_processes_without_duplicate_effects() -> anyhow::Result<()>
{
    let root = tempfile::tempdir()?;
    let repository = root.path().join("repository");
    let data_root = root.path().join("study/data");
    fs::create_dir(&repository)?;
    fs::create_dir_all(data_root.join("evidence/acquisition-plans"))?;
    fs::create_dir_all(data_root.join("evidence/materializations"))?;
    let repository = repository.canonicalize()?;
    let data_root = data_root.canonicalize()?;
    let plan_path = data_root.join("evidence/acquisition-plans/plan.json");
    fs::write(&plan_path, serde_json::to_vec_pretty(&acquisition_plan())?)?;
    let plan_path = plan_path.canonicalize()?;
    let executable = PathBuf::from(env!("CARGO_BIN_EXE_bijux-dna")).canonicalize()?;
    let specification_path = root.path().join("materialization-spec.json");
    fs::write(
        &specification_path,
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema_version": "bijux.runner.corpus_materialization_slurm_spec.v1",
            "run_id": "run-2042",
            "attempt_id": "acquisition-attempt-001",
            "portfolio_id": "human-short-read-diversity",
            "repository_path": repository,
            "repository_commit": "a".repeat(40),
            "executable_path": executable,
            "executable_sha256": bijux_dna_infra::hash_file_sha256(&executable)?,
            "command_arguments": [
                "bench",
                "materialize-fastq-corpus",
                "--plan",
                plan_path.display().to_string(),
                "--data-root",
                data_root.display().to_string(),
                "--repository-commit",
                "a".repeat(40),
                "--jobs",
                "2",
                "--retries",
                "2",
                "--timeout-seconds",
                "120",
                "--admitted-at-utc",
                "2026-08-06T10:25:00Z",
                "--json"
            ],
            "plan_path": plan_path,
            "plan_sha256": bijux_dna_infra::hash_file_sha256(&plan_path)?,
            "data_root": data_root,
            "evidence_root": data_root.join("evidence/materializations/run-2042"),
            "resources": {
                "account": "project",
                "partition": "compute",
                "cpus": 2,
                "memory_mb": 4096,
                "walltime": "01:00:00"
            },
            "maximum_parallelism": 2,
            "retries": 2,
            "timeout_seconds": 120,
            "admitted_at_utc": "2026-08-06T10:25:00Z"
        }))?,
    )?;

    let prepared = run_cli(&[
        "bench",
        "corpus-materialization",
        "prepare",
        "--specification",
        path_text(&specification_path)?,
        "--submitted-at-utc",
        "2026-08-06T10:20:00Z",
    ])?;
    assert_success(&prepared);
    let prepared_json: serde_json::Value = serde_json::from_slice(&prepared.stdout)?;
    assert_eq!(
        prepared_json.pointer("/request/run_id").and_then(serde_json::Value::as_str),
        Some("run-2042:human-short-read-diversity:acquisition-attempt-001")
    );

    let calls = root.path().join("sbatch.calls");
    let sbatch = root.path().join("sbatch");
    write_executable(
        &sbatch,
        &format!("#!/bin/sh\nprintf x >> '{}'\nprintf '456789\\n'\n", calls.display()),
    )?;
    let submit_arguments = [
        "bench",
        "corpus-materialization",
        "submit",
        "--specification",
        path_text(&specification_path)?,
        "--sbatch",
        path_text(&sbatch)?,
    ];
    let first = run_cli(&submit_arguments)?;
    let second = run_cli(&submit_arguments)?;
    assert_success(&first);
    assert_success(&second);
    let first_handle: serde_json::Value = serde_json::from_slice(&first.stdout)?;
    let second_handle: serde_json::Value = serde_json::from_slice(&second.stdout)?;
    assert_eq!(first_handle, second_handle);
    assert_eq!(first_handle.get("job_id").and_then(serde_json::Value::as_str), Some("456789"));
    assert_eq!(fs::read_to_string(calls)?, "x");
    Ok(())
}

fn run_cli(arguments: &[&str]) -> anyhow::Result<Output> {
    Ok(Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(repository_root()?)
        .args(arguments)
        .output()?)
}

fn assert_success(output: &Output) {
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
}

fn repository_root() -> anyhow::Result<PathBuf> {
    Ok(Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("bijux-dna manifest has no workspace root"))?
        .to_path_buf())
}

fn path_text(path: &Path) -> anyhow::Result<&str> {
    path.to_str().ok_or_else(|| anyhow::anyhow!("path is not UTF-8: {}", path.display()))
}

fn write_executable(path: &Path, body: &str) -> anyhow::Result<()> {
    fs::write(path, body)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn acquisition_plan() -> serde_json::Value {
    serde_json::json!({
        "schema_version": "bijux.benchmark.corpus_acquisition.v2",
        "corpus_id": "corpus-01",
        "portfolio_id": "human-short-read-diversity",
        "species": "Homo sapiens",
        "retrieved_at_utc": "2026-08-06T10:00:00Z",
        "source_authority": "European Nucleotide Archive",
        "description": "real benchmark corpus",
        "class": "production_real",
        "access_classification": "public",
        "redistribution_classification": "public-source-copy",
        "access_terms_url": "https://www.ebi.ac.uk/ena/browser/about/policies",
        "attribution": "cite source studies and accessions",
        "projected_storage_bytes": 1024,
        "projected_cpu_hours": 1,
        "projection_basis": "bounded scheduler contract projection",
        "truth_roles": ["archive_context"],
        "truth_strategy": "observational_only",
        "known_limitations": ["not population representative"],
        "source_spec": "benchmarks/corpora/human-short-read-diversity.toml",
        "source_spec_sha256": "b".repeat(64),
        "sample_count": 1,
        "file_count": 1,
        "total_size_bytes": 1024,
        "samples": [{
            "accession": "ERR000001",
            "study_accession": "PRJEB1",
            "sample_accession": "SAMEA1",
            "experiment_accession": "ERX1",
            "era": "modern",
            "layout": "se",
            "size_band": "under-100mb",
            "selection_reason": "real single-end coverage",
            "scientific_name": "Homo sapiens",
            "library_source": "GENOMIC",
            "library_strategy": "WGS",
            "library_selection": "RANDOM",
            "instrument_platform": "ILLUMINA",
            "instrument_model": "Illumina HiSeq",
            "archive_base_count": 10,
            "archive_read_count": 2,
            "first_public": "2020-01-01",
            "last_updated": "2020-01-02",
            "source_report_url": "https://www.ebi.ac.uk/ena/portal/api/filereport",
            "files": [{
                "mate": null,
                "name": "ERR000001.fastq.gz",
                "source_url": "https://archive.example/ERR000001.fastq.gz",
                "source_md5": "c".repeat(32),
                "size_bytes": 1024
            }]
        }]
    })
}
