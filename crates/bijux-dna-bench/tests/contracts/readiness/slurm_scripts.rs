use super::validate_benchmark_local_slurm_script_bodies;
use anyhow::Result;
use bijux_dna_bench_model::BenchmarkLocalSlurmScriptBodyReport;
use std::path::PathBuf;

#[test]
fn script_inspection_publishes_findings_and_preserves_evidence_on_invalid_acquisition() -> Result<()>
{
    let repository = bijux_dna_infra::temp_dir("slurm-script-inspection-")?;
    let root = PathBuf::from("artifacts/runs/bench/inspection");
    let output = root.join("inspection.json");
    let absolute_output = repository.path().join(&output);
    let script = repository.path().join(root.join("stage.sbatch"));
    let inspect = || {
        validate_benchmark_local_slurm_script_bodies(
            repository.path(),
            root.clone(),
            output.clone(),
        )
    };
    assert!(inspect().is_err());
    let empty: BenchmarkLocalSlurmScriptBodyReport =
        serde_json::from_slice(&std::fs::read(&absolute_output)?)?;
    assert!(!empty.ok && empty.script_count == 0 && empty.findings_count == 1);
    bijux_dna_infra::write_bytes(
        &script,
        b"#!/bin/bash\nset -euo pipefail\ncargo run -p bijux-dna -- bench local status\n",
    )?;
    assert!(inspect()?.ok);
    bijux_dna_infra::write_bytes(&script, b"# TODO: real command\necho execute placeholder\n")?;
    assert!(inspect().is_err());
    let retained = std::fs::read(&absolute_output)?;
    let failed: BenchmarkLocalSlurmScriptBodyReport = serde_json::from_slice(&retained)?;
    assert!(!failed.ok && failed.script_count == 1 && failed.findings_count > 0);
    bijux_dna_infra::write_bytes(&script, [255])?;
    assert!(inspect().is_err());
    assert_eq!(std::fs::read(&absolute_output)?, retained);
    let large = std::fs::OpenOptions::new().write(true).truncate(true).open(&script)?;
    large.set_len(16 * 1024 * 1024 + 1)?;
    drop(large);
    assert!(inspect().is_err());
    assert_eq!(std::fs::read(&absolute_output)?, retained);
    assert!(validate_benchmark_local_slurm_script_bodies(
        repository.path(),
        root.clone(),
        root.join("created/../bad.json")
    )
    .is_err());
    assert!(!repository.path().join(root.join("created")).exists());
    #[cfg(unix)]
    {
        std::fs::remove_file(&script)?;
        let external = repository.path().join("external.sbatch");
        bijux_dna_infra::write_bytes(&external, b"bijux-dna --help\n")?;
        std::os::unix::fs::symlink(&external, &script)?;
        assert!(inspect().is_err());
        assert_eq!(std::fs::read(&absolute_output)?, retained);
    }
    Ok(())
}
