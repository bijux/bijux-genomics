use bijux_dna_bench_model::{
    benchmark_local_all_domain_slurm_submit_manifest_schema,
    benchmark_local_slurm_submit_manifest_schema,
    validate_benchmark_local_all_domain_slurm_submit_manifest,
    validate_benchmark_local_slurm_submit_manifest, BenchmarkLocalAllDomainSlurmSubmitJob,
    BenchmarkLocalAllDomainSlurmSubmitManifest, BenchmarkLocalSlurmSubmitJob,
    BenchmarkLocalSlurmSubmitLogs, BenchmarkLocalSlurmSubmitManifest,
    BenchmarkLocalSlurmSubmitResources, BenchmarkLocalStageArtifactEntry,
    BenchmarkLocalStageReadinessKind, BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
    BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1,
};

fn all_domain_job(
    job_id_local: &str,
    pipeline_id: Option<&str>,
    node_id: Option<&str>,
    result_id: Option<&str>,
) -> BenchmarkLocalAllDomainSlurmSubmitJob {
    BenchmarkLocalAllDomainSlurmSubmitJob {
        job_id_local: job_id_local.to_string(),
        domain: "vcf".to_string(),
        stage_id: "vcf.stats".to_string(),
        pipeline_id: pipeline_id.map(ToString::to_string),
        node_id: node_id.map(ToString::to_string),
        tool_id: "bcftools".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        asset_profile_id: "vcf_cohort".to_string(),
        result_id: result_id.map(ToString::to_string),
        script_path: format!("runs/bench/slurm-dry-run/{job_id_local}/job.sbatch"),
        stdout: format!("runs/bench/slurm-dry-run/{job_id_local}/stdout.log"),
        stderr: format!("runs/bench/slurm-dry-run/{job_id_local}/stderr.log"),
        outputs: vec![format!("runs/bench/slurm-dry-run/{job_id_local}/stage-result.json")],
        dependencies: Vec::new(),
        resources: BenchmarkLocalSlurmSubmitResources {
            cpus_per_task: 2,
            memory_mb: 4096,
            time_limit: "01:00:00".to_string(),
        },
    }
}

fn all_domain_manifest() -> BenchmarkLocalAllDomainSlurmSubmitManifest {
    BenchmarkLocalAllDomainSlurmSubmitManifest {
        schema_version: BENCHMARK_LOCAL_ALL_DOMAIN_SLURM_SUBMIT_MANIFEST_SCHEMA_V1.to_string(),
        root_path: "runs/bench/slurm-dry-run/all-domains".to_string(),
        manifest_path: "runs/bench/slurm-dry-run/all-domains/submit-manifest.json".to_string(),
        run_id: "all-domain-benchmark-dry-run".to_string(),
        job_count: 2,
        dependency_count: 0,
        benchmark_job_count: 1,
        essential_pipeline_job_count: 1,
        jobs: vec![
            all_domain_job(
                "benchmark:vcf:result-1:vcf.stats:vcf_cohort:bcftools",
                None,
                None,
                Some("result-1"),
            ),
            all_domain_job(
                "pipeline:variant-qc-vcf:vcf.stats",
                Some("variant-qc-vcf"),
                Some("vcf.stats"),
                None,
            ),
        ],
    }
}

fn manifest() -> BenchmarkLocalSlurmSubmitManifest {
    BenchmarkLocalSlurmSubmitManifest {
        schema_version: BENCHMARK_LOCAL_SLURM_SUBMIT_MANIFEST_SCHEMA_V1.to_string(),
        root_path: "runs/bench/slurm-dry-run".to_string(),
        manifest_path: "runs/bench/slurm-dry-run/submit-manifest.json".to_string(),
        run_id: "local-benchmark-dry-run".to_string(),
        job_count: 1,
        dependency_count: 0,
        jobs: vec![BenchmarkLocalSlurmSubmitJob {
            job_name: "fastq-validate_reads".to_string(),
            domain: "fastq".to_string(),
            stage_id: Some("fastq.validate_reads".to_string()),
            pipeline_id: None,
            corpus_id: Some("corpus-02-edna-mini".to_string()),
            sample_id: None,
            sample_ids: vec!["edna-a".to_string(), "edna-b".to_string()],
            tool_id: "fastqvalidator".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::Smoke,
            resources: BenchmarkLocalSlurmSubmitResources {
                cpus_per_task: 2,
                memory_mb: 4096,
                time_limit: "01:00:00".to_string(),
            },
            script_path: "runs/bench/slurm-dry-run/fastq/validate-reads.sbatch".to_string(),
            logs: BenchmarkLocalSlurmSubmitLogs {
                stdout_path: "runs/bench/slurm-dry-run/result/stdout.log".to_string(),
                stderr_path: "runs/bench/slurm-dry-run/result/stderr.log".to_string(),
            },
            result_root: "runs/bench/slurm-dry-run/result".to_string(),
            outputs: vec![BenchmarkLocalStageArtifactEntry {
                artifact_id: "validation-report".to_string(),
                path: "runs/bench/slurm-dry-run/result/report.json".to_string(),
                role: "report".to_string(),
                optional: false,
            }],
            dependencies: Vec::new(),
            compatibility_kind: "fixture".to_string(),
            compatibility_note: "fixture-backed validation".to_string(),
        }],
    }
}

#[test]
fn local_slurm_submit_manifest_round_trips_and_admits() -> anyhow::Result<()> {
    let manifest = manifest();
    validate_benchmark_local_slurm_submit_manifest(&manifest)?;
    let round_trip: BenchmarkLocalSlurmSubmitManifest =
        serde_json::from_slice(&serde_json::to_vec(&manifest)?)?;
    assert_eq!(round_trip, manifest);
    Ok(())
}

#[test]
fn local_slurm_submit_manifest_rejects_aggregate_resource_and_path_drift() {
    let mut aggregate = manifest();
    aggregate.job_count = 0;
    assert!(validate_benchmark_local_slurm_submit_manifest(&aggregate).is_err());

    let mut resource = manifest();
    resource.jobs[0].resources.cpus_per_task = 0;
    assert!(validate_benchmark_local_slurm_submit_manifest(&resource).is_err());

    let mut path = manifest();
    path.jobs[0].logs.stdout_path = "runs/wrong/stdout.log".to_string();
    assert!(validate_benchmark_local_slurm_submit_manifest(&path).is_err());
}

#[test]
fn local_slurm_submit_manifest_rejects_identity_and_dependency_drift() {
    let mut identity = manifest();
    identity.jobs[0].job_name = "wrong-name".to_string();
    assert!(validate_benchmark_local_slurm_submit_manifest(&identity).is_err());

    let mut dependency = manifest();
    dependency.jobs[0].dependencies.push("missing-job".to_string());
    dependency.dependency_count = 1;
    assert!(validate_benchmark_local_slurm_submit_manifest(&dependency).is_err());
}

#[test]
fn local_slurm_submit_manifest_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-slurm-submit-manifest.schema.json");
    let generated = serde_json::to_value(benchmark_local_slurm_submit_manifest_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local Slurm submit-manifest schema drifted");
    Ok(())
}

#[test]
fn local_slurm_submit_manifest_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(manifest())?;
    value["legacy_status"] = serde_json::json!("ready");
    assert!(serde_json::from_value::<BenchmarkLocalSlurmSubmitManifest>(value).is_err());
    Ok(())
}

#[test]
fn all_domain_slurm_submit_manifest_round_trips_and_admits() -> anyhow::Result<()> {
    let manifest = all_domain_manifest();
    validate_benchmark_local_all_domain_slurm_submit_manifest(&manifest)?;
    let round_trip: BenchmarkLocalAllDomainSlurmSubmitManifest =
        serde_json::from_slice(&serde_json::to_vec(&manifest)?)?;
    assert_eq!(round_trip, manifest);
    Ok(())
}

#[test]
fn all_domain_slurm_submit_manifest_rejects_kind_and_dependency_drift() {
    let mut kind = all_domain_manifest();
    kind.jobs[0].pipeline_id = Some("wrong-pipeline".to_string());
    assert!(validate_benchmark_local_all_domain_slurm_submit_manifest(&kind).is_err());

    let mut dependency = all_domain_manifest();
    dependency.jobs[1].dependencies.push("missing-job".to_string());
    dependency.dependency_count = 1;
    assert!(validate_benchmark_local_all_domain_slurm_submit_manifest(&dependency).is_err());
}

#[test]
fn all_domain_slurm_submit_manifest_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-all-domain-slurm-submit-manifest.schema.json");
    let generated =
        serde_json::to_value(benchmark_local_all_domain_slurm_submit_manifest_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "all-domain Slurm submit-manifest schema drifted");
    Ok(())
}
