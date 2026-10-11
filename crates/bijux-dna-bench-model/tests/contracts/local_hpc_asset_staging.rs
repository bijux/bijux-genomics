use bijux_dna_bench_model::{
    benchmark_local_hpc_asset_staging_schema, validate_benchmark_local_hpc_asset_staging_manifest,
    BenchmarkLocalHpcAssetStagingInput, BenchmarkLocalHpcAssetStagingJob,
    BenchmarkLocalHpcAssetStagingManifest, BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1,
};

fn input(source: &str, consumer: &str) -> BenchmarkLocalHpcAssetStagingInput {
    BenchmarkLocalHpcAssetStagingInput {
        artifact_id: Some("reference_fasta".to_string()),
        artifact_role: Some("reference".to_string()),
        source_kind: "file".to_string(),
        source_path: source.to_string(),
        staged_path: format!("artifacts/runs/bench/hpc-dry-run/staged/{source}"),
        checksum_sha256: "a".repeat(64),
        size_bytes: 1024,
        member_count: 1,
        consumer_result_id: consumer.to_string(),
    }
}

fn job(
    result_id: &str,
    domain: &str,
    stage_id: &str,
    tool_id: &str,
    sources: &[&str],
) -> BenchmarkLocalHpcAssetStagingJob {
    BenchmarkLocalHpcAssetStagingJob {
        result_id: result_id.to_string(),
        domain: domain.to_string(),
        stage_id: stage_id.to_string(),
        tool_id: tool_id.to_string(),
        corpus_id: "corpus-01".to_string(),
        asset_profile_id: "sample-set".to_string(),
        staged_inputs: sources.iter().map(|source| input(source, result_id)).collect(),
    }
}

fn manifest() -> BenchmarkLocalHpcAssetStagingManifest {
    let shared = "assets/reference/genome.fasta";
    BenchmarkLocalHpcAssetStagingManifest {
        schema_version: BENCHMARK_LOCAL_HPC_ASSET_STAGING_SCHEMA_V1.to_string(),
        output_path: "artifacts/runs/bench/hpc-dry-run/asset-staging-manifest.json".to_string(),
        staging_root: "artifacts/runs/bench/hpc-dry-run/staged".to_string(),
        selected_job_count: 2,
        staged_input_count: 3,
        unique_source_path_count: 2,
        jobs: vec![
            job("bam-result", "bam", "bam.align", "bwa-mem2", &[shared]),
            job(
                "vcf-result",
                "vcf",
                "vcf.normalize",
                "bcftools",
                &[shared, "benchmarks/corpora/input.vcf.gz"],
            ),
        ],
    }
}

#[test]
fn local_hpc_asset_staging_round_trips_and_matches_generated_schema() -> anyhow::Result<()> {
    let manifest = manifest();
    validate_benchmark_local_hpc_asset_staging_manifest(&manifest)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcAssetStagingManifest>(&serde_json::to_vec(
            &manifest
        )?)?,
        manifest
    );

    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_asset_staging_schema())?;
    let path =
        repository_root.join("benchmarks/contracts/schemas/local-hpc-asset-staging.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC asset-staging schema drifted");
    Ok(())
}

#[test]
fn local_hpc_asset_staging_rejects_counter_identity_and_order_drift() {
    let mut counter = manifest();
    counter.staged_input_count = 2;
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&counter).is_err());

    let mut duplicate_result = manifest();
    let duplicate_id = duplicate_result.jobs[0].result_id.clone();
    duplicate_result.jobs[1].result_id.clone_from(&duplicate_id);
    for input in &mut duplicate_result.jobs[1].staged_inputs {
        input.consumer_result_id.clone_from(&duplicate_id);
    }
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&duplicate_result).is_err());

    let mut job_order = manifest();
    job_order.jobs.reverse();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&job_order).is_err());

    let mut input_order = manifest();
    input_order.jobs[1].staged_inputs.reverse();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&input_order).is_err());

    let mut shared_identity = manifest();
    shared_identity.jobs[1].staged_inputs[0].checksum_sha256 = "b".repeat(64);
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&shared_identity).is_err());

    let mut unique_count = manifest();
    unique_count.unique_source_path_count = 3;
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&unique_count).is_err());
}

#[test]
fn local_hpc_asset_staging_rejects_unsafe_or_incomplete_inputs() {
    let mut overlapping_roots = manifest();
    overlapping_roots.staging_root =
        "artifacts/runs/bench/hpc-dry-run/asset-staging-manifest.json/staged".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&overlapping_roots).is_err());

    let mut consumer = manifest();
    consumer.jobs[0].staged_inputs[0].consumer_result_id = "other".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&consumer).is_err());

    let mut staged_lineage = manifest();
    staged_lineage.jobs[0].staged_inputs[0].staged_path =
        "artifacts/runs/bench/hpc-dry-run/staged/other.fasta".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&staged_lineage).is_err());

    let mut traversal = manifest();
    traversal.jobs[0].staged_inputs[0].source_path = "assets/../secret".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&traversal).is_err());

    let mut already_staged = manifest();
    already_staged.jobs[0].staged_inputs[0].source_path =
        "artifacts/runs/bench/hpc-dry-run/staged/already-present.fasta".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&already_staged).is_err());

    let mut source_kind = manifest();
    source_kind.jobs[0].staged_inputs[0].source_kind = "remote".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&source_kind).is_err());

    let mut checksum = manifest();
    checksum.jobs[0].staged_inputs[0].checksum_sha256 = "invalid".to_string();
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&checksum).is_err());

    let mut empty = manifest();
    empty.jobs[0].staged_inputs[0].member_count = 0;
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&empty).is_err());

    let mut artifact_pairing = manifest();
    artifact_pairing.jobs[0].staged_inputs[0].artifact_role = None;
    assert!(validate_benchmark_local_hpc_asset_staging_manifest(&artifact_pairing).is_err());
}
