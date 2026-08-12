use bijux_dna_bench_model::{
    benchmark_local_slurm_dependency_check_schema, benchmark_local_slurm_shell_syntax_schema,
    validate_benchmark_local_slurm_dependency_check, validate_benchmark_local_slurm_shell_syntax,
    BenchmarkLocalSlurmDependencyCheckJob, BenchmarkLocalSlurmDependencyCheckReport,
    BenchmarkLocalSlurmShellSyntaxEntry, BenchmarkLocalSlurmShellSyntaxReport,
    BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1, BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1,
};

fn report() -> BenchmarkLocalSlurmDependencyCheckReport {
    BenchmarkLocalSlurmDependencyCheckReport {
        schema_version: BENCHMARK_LOCAL_SLURM_DEPENDENCY_CHECK_SCHEMA_V1.to_string(),
        root_path: "runs/bench/slurm-dry-run".to_string(),
        manifest_path: "runs/bench/slurm-dry-run/submit-manifest.json".to_string(),
        report_path: "runs/bench/slurm-dry-run/dependency-check.json".to_string(),
        job_count: 1,
        manifest_dependency_count: 1,
        script_header_dependency_count: 0,
        findings_count: 0,
        ok: true,
        report_findings: Vec::new(),
        jobs: vec![BenchmarkLocalSlurmDependencyCheckJob {
            job_name: "fastq-profile".to_string(),
            stage_id: Some("fastq.profile_reads".to_string()),
            script_path: "runs/bench/slurm-dry-run/fastq-profile.sbatch".to_string(),
            manifest_dependencies: vec!["fastq-validate".to_string()],
            script_header_dependencies: Vec::new(),
            dependency_source: "submit_manifest".to_string(),
            ok: true,
            findings: Vec::new(),
        }],
    }
}

fn shell_syntax_report() -> BenchmarkLocalSlurmShellSyntaxReport {
    BenchmarkLocalSlurmShellSyntaxReport {
        schema_version: BENCHMARK_LOCAL_SLURM_SHELL_SYNTAX_SCHEMA_V1.to_string(),
        root_path: "runs/bench/slurm-dry-run".to_string(),
        report_path: "runs/bench/slurm-dry-run/bash-n-report.json".to_string(),
        script_count: 1,
        findings_count: 0,
        ok: true,
        report_findings: Vec::new(),
        scripts: vec![BenchmarkLocalSlurmShellSyntaxEntry {
            script_path: "runs/bench/slurm-dry-run/fastq-profile.sbatch".to_string(),
            ok: true,
            findings: Vec::new(),
        }],
    }
}

#[test]
fn local_slurm_dependency_report_round_trips_and_admits() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_slurm_dependency_check(&report)?;
    let round_trip: BenchmarkLocalSlurmDependencyCheckReport =
        serde_json::from_slice(&serde_json::to_vec(&report)?)?;
    assert_eq!(round_trip, report);
    Ok(())
}

#[test]
fn local_slurm_dependency_report_rejects_aggregate_and_source_drift() {
    let mut aggregate = report();
    aggregate.manifest_dependency_count = 0;
    assert!(validate_benchmark_local_slurm_dependency_check(&aggregate).is_err());

    let mut source = report();
    source.jobs[0].dependency_source = "script_header".to_string();
    assert!(validate_benchmark_local_slurm_dependency_check(&source).is_err());
}

#[test]
fn local_slurm_dependency_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-slurm-dependency-check.schema.json");
    let generated = serde_json::to_value(benchmark_local_slurm_dependency_check_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local Slurm dependency-check schema drifted");
    Ok(())
}

#[test]
fn local_slurm_dependency_report_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report())?;
    value["legacy_status"] = serde_json::json!("ready");
    assert!(serde_json::from_value::<BenchmarkLocalSlurmDependencyCheckReport>(value).is_err());
    Ok(())
}

#[test]
fn local_slurm_shell_syntax_report_round_trips_and_admits() -> anyhow::Result<()> {
    let report = shell_syntax_report();
    validate_benchmark_local_slurm_shell_syntax(&report)?;
    let round_trip: BenchmarkLocalSlurmShellSyntaxReport =
        serde_json::from_slice(&serde_json::to_vec(&report)?)?;
    assert_eq!(round_trip, report);
    Ok(())
}

#[test]
fn local_slurm_shell_syntax_report_rejects_aggregate_verdict_and_identity_drift() {
    let mut aggregate = shell_syntax_report();
    aggregate.findings_count = 1;
    assert!(validate_benchmark_local_slurm_shell_syntax(&aggregate).is_err());

    let mut verdict = shell_syntax_report();
    verdict.scripts[0].ok = false;
    assert!(validate_benchmark_local_slurm_shell_syntax(&verdict).is_err());

    let mut identity = shell_syntax_report();
    identity.scripts.push(identity.scripts[0].clone());
    identity.script_count = identity.scripts.len();
    assert!(validate_benchmark_local_slurm_shell_syntax(&identity).is_err());
}

#[test]
fn local_slurm_shell_syntax_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-slurm-shell-syntax.schema.json");
    let generated = serde_json::to_value(benchmark_local_slurm_shell_syntax_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local Slurm shell-syntax schema drifted");
    Ok(())
}

#[test]
fn local_slurm_shell_syntax_report_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(shell_syntax_report())?;
    value["legacy_status"] = serde_json::json!("ready");
    assert!(serde_json::from_value::<BenchmarkLocalSlurmShellSyntaxReport>(value).is_err());
    Ok(())
}
