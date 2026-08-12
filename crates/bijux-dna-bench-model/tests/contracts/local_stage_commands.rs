use bijux_dna_bench_model::{
    benchmark_local_stage_commands_schema, validate_benchmark_local_stage_commands,
    BenchmarkLocalStageArtifactEntry, BenchmarkLocalStageCommandEntry,
    BenchmarkLocalStageCommandManifest, BenchmarkLocalStageReadinessKind,
    BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3,
};

fn artifact(id: &str, path: &str, role: &str) -> BenchmarkLocalStageArtifactEntry {
    BenchmarkLocalStageArtifactEntry {
        artifact_id: id.to_string(),
        path: path.to_string(),
        role: role.to_string(),
        optional: false,
    }
}

fn manifest() -> BenchmarkLocalStageCommandManifest {
    BenchmarkLocalStageCommandManifest {
        schema_version: BENCHMARK_LOCAL_STAGE_COMMANDS_SCHEMA_V3.to_string(),
        script_output_path: "artifacts/benchmark-smoke/readiness/rendered.sh".to_string(),
        manifest_output_path: "artifacts/benchmark-smoke/readiness/rendered.json".to_string(),
        argv_output_path: "artifacts/benchmark-smoke/readiness/rendered.argv.jsonl".to_string(),
        command_count: 1,
        commands: vec![BenchmarkLocalStageCommandEntry {
            stage_id: "fastq.validate_reads".to_string(),
            readiness_kind: BenchmarkLocalStageReadinessKind::DryOrSmoke,
            tool_id: "fastqvalidator".to_string(),
            inputs: vec![artifact("reads", "data/input.fastq.gz", "reads")],
            outputs: vec![artifact("report", "runs/report.json", "report")],
            threads: 2,
            memory_mb: 4096,
            argv: vec![
                "bijux-dna".to_string(),
                "bench".to_string(),
                "local".to_string(),
                "materialize-stage".to_string(),
                "--stage-id".to_string(),
                "fastq.validate_reads".to_string(),
            ],
            command: "bijux-dna bench local materialize-stage --stage-id fastq.validate_reads"
                .to_string(),
        }],
    }
}

#[test]
fn local_stage_command_manifest_round_trips_and_admits() -> anyhow::Result<()> {
    let manifest = manifest();
    validate_benchmark_local_stage_commands(&manifest)?;
    let round_trip: BenchmarkLocalStageCommandManifest =
        serde_json::from_slice(&serde_json::to_vec(&manifest)?)?;
    assert_eq!(round_trip, manifest);
    Ok(())
}

#[test]
fn local_stage_command_manifest_rejects_aggregate_resource_and_identity_drift() {
    let mut aggregate = manifest();
    aggregate.command_count = 0;
    assert!(validate_benchmark_local_stage_commands(&aggregate).is_err());

    let mut resource = manifest();
    resource.commands[0].threads = 0;
    assert!(validate_benchmark_local_stage_commands(&resource).is_err());

    let mut identity = manifest();
    identity.commands.push(identity.commands[0].clone());
    identity.command_count = identity.commands.len();
    assert!(validate_benchmark_local_stage_commands(&identity).is_err());
}

#[test]
fn local_stage_command_manifest_rejects_argv_and_artifact_drift() {
    let mut argv = manifest();
    if let Some(last) = argv.commands[0].argv.last_mut() {
        *last = "bam.validate".to_string();
    }
    assert!(validate_benchmark_local_stage_commands(&argv).is_err());

    let mut artifact = manifest();
    let duplicate = artifact.commands[0].outputs[0].clone();
    artifact.commands[0].outputs.push(duplicate);
    assert!(validate_benchmark_local_stage_commands(&artifact).is_err());
}

#[test]
fn local_stage_commands_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-stage-commands.schema.json");
    let generated = serde_json::to_value(benchmark_local_stage_commands_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local stage-command schema drifted");
    Ok(())
}

#[test]
fn local_stage_command_manifest_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(manifest())?;
    value["legacy_status"] = serde_json::json!("ready");
    assert!(serde_json::from_value::<BenchmarkLocalStageCommandManifest>(value).is_err());
    Ok(())
}
