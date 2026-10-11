use super::inspect_benchmark_local_slurm_dependencies;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalSlurmDependencyCheckReport;
use std::path::PathBuf;

#[test]
fn dependency_inspection_preserves_negative_evidence_and_rejects_unsafe_acquisition() -> Result<()>
{
    let repository = bijux_dna_infra::temp_dir("slurm-dependency-inspection-")?;
    let root = PathBuf::from("artifacts/runs/bench/dependency-inspection");
    let manifest_path = root.join("submit.json");
    let report_path = root.join("dependency.json");
    let script = repository.path().join(root.join("stage.sbatch"));
    let mut wire = serde_json::json!({
        "schema_version":"bijux.bench.local_slurm_submit_manifest.v1",
        "root_path":root,"manifest_path":manifest_path,"run_id":"local-benchmark-dry-run",
        "job_count":1,"dependency_count":1,
        "jobs":[{
            "job_name":"fastq-validate_reads","domain":"fastq","stage_id":"fastq.validate_reads",
            "pipeline_id":null,"corpus_id":"mini","sample_id":null,"sample_ids":["sample"],
            "tool_id":"fastqvalidator","readiness_kind":"smoke",
            "resources":{"cpus_per_task":1,"memory_mb":128,"time_limit":"00:01:00"},
            "script_path":root.join("stage.sbatch"),"logs":{"stdout_path":"stdout.log","stderr_path":"stderr.log"},
            "result_root":"result","outputs":[],"dependencies":["upstream"],
            "compatibility_kind":"fixture","compatibility_note":"local diagnostic contract"
        }]
    });
    let inspect = || {
        inspect_benchmark_local_slurm_dependencies(
            repository.path(),
            root.clone(),
            manifest_path.clone(),
            report_path.clone(),
        )
    };
    bijux_dna_infra::atomic_write_json(&repository.path().join(&manifest_path), &wire)?;
    bijux_dna_infra::write_bytes(
        &script,
        b"#!/bin/sh\n#SBATCH --dependency=afterok:upstream\nbijux-dna --help\n",
    )?;
    assert!(inspect().is_err());
    let retained = std::fs::read(repository.path().join(&report_path))?;
    let report: BenchmarkLocalSlurmDependencyCheckReport = serde_json::from_slice(&retained)?;
    assert_eq!(report.findings_count, 2);
    assert_eq!(report.jobs[0].dependency_source, "mixed");
    wire["job_count"] = serde_json::json!(2);
    bijux_dna_infra::atomic_write_json(&repository.path().join(&manifest_path), &wire)?;
    assert!(inspect().is_err());
    assert_eq!(std::fs::read(repository.path().join(&report_path))?, retained);
    wire["job_count"] = serde_json::json!(1);
    wire["jobs"][0]["script_path"] = serde_json::json!("outside.sbatch");
    let outside = repository.path().join("outside.sbatch");
    bijux_dna_infra::write_bytes(&outside, b"bijux-dna --help\n")?;
    bijux_dna_infra::atomic_write_json(&repository.path().join(&manifest_path), &wire)?;
    assert!(inspect().is_err());
    assert_eq!(std::fs::read(repository.path().join(&report_path))?, retained);
    wire["jobs"][0]["script_path"] = serde_json::json!(root.join("stage.sbatch"));
    bijux_dna_infra::atomic_write_json(&repository.path().join(&manifest_path), &wire)?;
    bijux_dna_infra::write_bytes(&script, [255])?;
    assert!(inspect().is_err());
    assert_eq!(std::fs::read(repository.path().join(&report_path))?, retained);
    #[cfg(unix)]
    {
        std::fs::remove_file(&script)?;
        std::os::unix::fs::symlink(&outside, &script)?;
        assert!(inspect().is_err());
        assert_eq!(std::fs::read(repository.path().join(&report_path))?, retained);
        std::fs::remove_file(&script)?;
    }
    bijux_dna_infra::write_bytes(&script, b"bijux-dna --help\n")?;
    assert!(inspect()?.ok);
    Ok(())
}
