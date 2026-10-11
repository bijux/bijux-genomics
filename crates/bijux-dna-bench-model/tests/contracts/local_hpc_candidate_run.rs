use std::collections::BTreeMap;

use bijux_dna_bench_model::{
    benchmark_local_hpc_candidate_run_manifest_schema,
    validate_benchmark_local_hpc_candidate_run_manifest, BenchmarkLocalHpcAssetStagingInput,
    BenchmarkLocalHpcCandidateBehavior, BenchmarkLocalHpcCandidateJob,
    BenchmarkLocalHpcCandidateResourceCeiling, BenchmarkLocalHpcCandidateResourceHint,
    BenchmarkLocalHpcCandidateRunManifest, BenchmarkLocalHpcCandidateStopCondition,
    BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1,
};

fn row(domain: &str, result_id: &str) -> BenchmarkLocalHpcCandidateJob {
    let execution_mode = "container";
    let source_path = format!("benchmarks/fixtures/{domain}/input.dat");
    BenchmarkLocalHpcCandidateJob {
        representative_group_id: format!("{domain}:{execution_mode}"),
        selection_reason: "smallest eligible dependency-free benchmark row".to_string(),
        job_id_local: format!("benchmark:{result_id}"),
        result_id: result_id.to_string(),
        domain: domain.to_string(),
        stage_id: format!("{domain}.stage"),
        tool_id: "tool-a".to_string(),
        corpus_id: "corpus-a".to_string(),
        asset_profile_id: "asset-a".to_string(),
        execution_mode: execution_mode.to_string(),
        execution_mode_summary: "admitted container execution".to_string(),
        resolution_kind: "admitted_artifact".to_string(),
        resolution_target: "containers/tool-a.sif".to_string(),
        command_entrypoint: Some("tool-a".to_string()),
        expected_stage_result_manifest_path: format!("runs/{result_id}/stage-result.json"),
        expected_outputs: vec![format!("runs/{result_id}/output.dat")],
        resource_hint: BenchmarkLocalHpcCandidateResourceHint {
            cpus_per_task: 2,
            memory_mb: 2048,
            time_limit: "00:10:00".to_string(),
            walltime_minutes: 10,
            scratch_gb: 1,
        },
        staged_input_count: 1,
        staged_input_total_bytes: 1024,
        staged_inputs: vec![BenchmarkLocalHpcAssetStagingInput {
            artifact_id: Some("input".to_string()),
            artifact_role: Some("reads".to_string()),
            source_kind: "file".to_string(),
            source_path: source_path.clone(),
            staged_path: format!("artifacts/runs/bench/hpc-dry-run/staged/{source_path}"),
            checksum_sha256: "a".repeat(64),
            size_bytes: 1024,
            member_count: 1,
            consumer_result_id: result_id.to_string(),
        }],
        stop_conditions: vec![BenchmarkLocalHpcCandidateStopCondition {
            condition_id: "missing_stage_result_manifest".to_string(),
            source_surface: "local_hpc_job_completion".to_string(),
            detail: "stop when the stage result manifest is absent".to_string(),
        }],
    }
}

fn count(
    rows: &[BenchmarkLocalHpcCandidateJob],
    key: impl Fn(&BenchmarkLocalHpcCandidateJob) -> &str,
) -> BTreeMap<String, usize> {
    let mut counts = BTreeMap::new();
    for row in rows {
        *counts.entry(key(row).to_string()).or_default() += 1;
    }
    counts
}

fn manifest() -> BenchmarkLocalHpcCandidateRunManifest {
    let rows = vec![row("bam", "bam-result"), row("fastq", "fastq-result")];
    BenchmarkLocalHpcCandidateRunManifest {
        schema_version: BENCHMARK_LOCAL_HPC_CANDIDATE_RUN_MANIFEST_SCHEMA_V1.to_string(),
        output_path: "artifacts/benchmark-smoke/readiness/hpc/FIRST_HPC_CANDIDATE_RUN.json"
            .to_string(),
        selection_profile_id: "small_runtime_surface_representatives".to_string(),
        evaluated_job_count: 10,
        benchmark_job_count: 8,
        root_benchmark_job_count: 6,
        eligible_small_job_count: 4,
        representative_group_count: rows.len(),
        selected_job_count: rows.len(),
        selected_staged_input_count: 2,
        selected_staged_input_bytes: 2048,
        selected_domain_counts: count(&rows, |row| &row.domain),
        selected_execution_mode_counts: count(&rows, |row| &row.execution_mode),
        selected_resolution_kind_counts: count(&rows, |row| &row.resolution_kind),
        exclusion_reason_counts: BTreeMap::from([
            ("non_benchmark_result".to_string(), 2),
            ("has_dependencies".to_string(), 2),
            ("superseded_by_smaller_representative".to_string(), 2),
            ("resource_envelope_exceeded".to_string(), 2),
        ]),
        resource_ceiling: BenchmarkLocalHpcCandidateResourceCeiling {
            cpus_per_task_max: 4,
            memory_mb_max: 4096,
            walltime_minutes_max: 15,
            scratch_gb_max: 2,
            staged_input_bytes_max: 10 * 1024 * 1024,
        },
        behavior: BenchmarkLocalHpcCandidateBehavior {
            only_benchmark_result_jobs: true,
            only_dependency_free_jobs: true,
            only_known_assets: true,
            only_known_execution_modes: true,
            only_available_execution_resolution: true,
            only_small_resource_envelopes: true,
            only_small_input_footprints: true,
            selected_rows_keep_expected_outputs: true,
            selected_rows_keep_stop_conditions: true,
            vcf_rows_excluded_for_unknown_execution_modes: true,
            proven: true,
        },
        rows,
    }
}

#[test]
fn local_hpc_candidate_run_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let manifest = manifest();
    validate_benchmark_local_hpc_candidate_run_manifest(&manifest)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcCandidateRunManifest>(&serde_json::to_vec(
            &manifest
        )?)?,
        manifest
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_candidate_run_manifest_schema())?;
    let path = repository_root
        .join("benchmarks/contracts/schemas/local-hpc-candidate-run-manifest.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC candidate-run schema drifted");
    Ok(())
}

#[test]
fn local_hpc_candidate_run_rejects_identity_counter_and_resource_drift() {
    let mut duplicate = manifest();
    duplicate.rows[1].representative_group_id = duplicate.rows[0].representative_group_id.clone();
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&duplicate).is_err());

    let mut counters = manifest();
    counters.selected_staged_input_bytes -= 1;
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&counters).is_err());

    let mut exclusions = manifest();
    exclusions.exclusion_reason_counts.remove("non_benchmark_result");
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&exclusions).is_err());

    let mut resource = manifest();
    resource.rows[0].resource_hint.memory_mb = resource.resource_ceiling.memory_mb_max + 1;
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&resource).is_err());

    let mut lineage = manifest();
    lineage.rows[0].staged_inputs[0].consumer_result_id = "other-result".to_string();
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&lineage).is_err());

    let mut behavior = manifest();
    behavior.behavior.proven = false;
    assert!(validate_benchmark_local_hpc_candidate_run_manifest(&behavior).is_err());
}
