use std::path::Path;

use bijux_dna_bench_model::{
    benchmark_local_essential_pipeline_fake_run_node_metrics_schema,
    benchmark_local_essential_pipeline_fake_run_pipeline_schema,
    benchmark_local_essential_pipeline_fake_runs_schema,
    validate_benchmark_local_essential_pipeline_fake_run_node_metrics,
    validate_benchmark_local_essential_pipeline_fake_run_node_report,
    validate_benchmark_local_essential_pipeline_fake_run_pipeline_report,
    validate_benchmark_local_essential_pipeline_fake_runs_report,
    BenchmarkLocalEssentialPipelineFakeRunInputBinding,
    BenchmarkLocalEssentialPipelineFakeRunNodeMetrics,
    BenchmarkLocalEssentialPipelineFakeRunNodeReport,
    BenchmarkLocalEssentialPipelineFakeRunOutputEntry,
    BenchmarkLocalEssentialPipelineFakeRunPipelineReport,
    BenchmarkLocalEssentialPipelineFakeRunsReport,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1,
    BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1,
};

const ROOT: &str = "artifacts/benchmark-smoke/simulated-runs/pipelines/essential";
const PIPELINE_ID: &str = "core-germline-fastq-bam-vcf";

fn node() -> BenchmarkLocalEssentialPipelineFakeRunNodeReport {
    let node_root = format!("{ROOT}/{PIPELINE_ID}/bam.align");
    BenchmarkLocalEssentialPipelineFakeRunNodeReport {
        pipeline_id: PIPELINE_ID.to_string(),
        node_id: "bam.align".to_string(),
        stage_id: "bam.align".to_string(),
        tool_id: "bowtie2".to_string(),
        domain: "bam".to_string(),
        readiness_kind: "smoke".to_string(),
        command_source: "bam_governed_stage_command".to_string(),
        dependency_count: 1,
        command_step_count: 2,
        declared_output_count: 2,
        created_output_count: 2,
        command_script_path: format!("{node_root}/command.sh"),
        stdout_path: format!("{node_root}/stdout.txt"),
        stderr_path: format!("{node_root}/stderr.txt"),
        metrics_path: format!("{node_root}/metrics.json"),
        stage_result_path: format!("{node_root}/stage-result.json"),
        outputs: vec![
            BenchmarkLocalEssentialPipelineFakeRunOutputEntry {
                artifact_id: "aligned_bam".to_string(),
                declared_output: "aligned_bam".to_string(),
                fake_run_path: format!("{node_root}/declared-outputs/aligned.bam"),
                role: "bam".to_string(),
                exists: true,
            },
            BenchmarkLocalEssentialPipelineFakeRunOutputEntry {
                artifact_id: "align_metrics".to_string(),
                declared_output: "align_metrics".to_string(),
                fake_run_path: format!("{node_root}/declared-outputs/align_metrics.json"),
                role: "metrics".to_string(),
                exists: true,
            },
        ],
    }
}

fn pipeline() -> BenchmarkLocalEssentialPipelineFakeRunPipelineReport {
    BenchmarkLocalEssentialPipelineFakeRunPipelineReport {
        schema_version: BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_PIPELINE_SCHEMA_V1.to_string(),
        pipeline_id: PIPELINE_ID.to_string(),
        pipeline_manifest_path: format!("{ROOT}/{PIPELINE_ID}/manifest.json"),
        node_count: 1,
        created_output_count: 2,
        topological_order: vec!["bam.align".to_string()],
        nodes: vec![node()],
    }
}

fn report() -> BenchmarkLocalEssentialPipelineFakeRunsReport {
    BenchmarkLocalEssentialPipelineFakeRunsReport {
        schema_version: BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUNS_SCHEMA_V1.to_string(),
        fake_run_root: ROOT.to_string(),
        root_manifest_path: format!("{ROOT}/manifest.json"),
        pipeline_count: 1,
        node_count: 1,
        created_output_count: 2,
        pipelines: vec![pipeline()],
    }
}

fn metrics() -> BenchmarkLocalEssentialPipelineFakeRunNodeMetrics {
    BenchmarkLocalEssentialPipelineFakeRunNodeMetrics {
        schema_version: BENCHMARK_LOCAL_ESSENTIAL_PIPELINE_FAKE_RUN_NODE_METRICS_SCHEMA_V1
            .to_string(),
        pipeline_id: PIPELINE_ID.to_string(),
        node_id: "bam.align".to_string(),
        stage_id: "bam.align".to_string(),
        tool_id: "bowtie2".to_string(),
        dependency_count: 1,
        external_input_count: 1,
        upstream_input_count: 1,
        command_step_count: 2,
        output_count: 2,
        materialized_byte_count: 128,
        simulated_elapsed_seconds: 1.5,
        corpus_id: "local-bam-ci-small-v1".to_string(),
        asset_profile_id: "bam-ci-small".to_string(),
        upstream_bindings: vec![BenchmarkLocalEssentialPipelineFakeRunInputBinding {
            input_id: "validated_reads".to_string(),
            source: "upstream".to_string(),
            producer_node_id: Some("fastq.validate_reads".to_string()),
            fake_run_path: Some(format!(
                "{ROOT}/{PIPELINE_ID}/fastq.validate_reads/declared-outputs/validated.fastq"
            )),
        }],
        external_bindings: vec![BenchmarkLocalEssentialPipelineFakeRunInputBinding {
            input_id: "reference_fasta".to_string(),
            source: "external".to_string(),
            producer_node_id: None,
            fake_run_path: None,
        }],
    }
}

#[test]
fn essential_pipeline_fake_run_records_round_trip_and_match_generated_schemas() -> anyhow::Result<()>
{
    let report = report();
    let metrics = metrics();
    validate_benchmark_local_essential_pipeline_fake_runs_report(&report, &[PIPELINE_ID])?;
    validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&metrics)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalEssentialPipelineFakeRunsReport>(
            &serde_json::to_vec(&report)?
        )?,
        report
    );
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalEssentialPipelineFakeRunNodeMetrics>(
            &serde_json::to_vec(&metrics)?
        )?,
        metrics
    );

    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    for (relative_path, generated) in [
        (
            "benchmarks/contracts/schemas/local-essential-pipeline-fake-runs.schema.json",
            benchmark_local_essential_pipeline_fake_runs_schema(),
        ),
        (
            "benchmarks/contracts/schemas/local-essential-pipeline-fake-run-pipeline.schema.json",
            benchmark_local_essential_pipeline_fake_run_pipeline_schema(),
        ),
        (
            "benchmarks/contracts/schemas/local-essential-pipeline-fake-run-node-metrics.schema.json",
            benchmark_local_essential_pipeline_fake_run_node_metrics_schema(),
        ),
    ] {
        let generated = serde_json::to_value(generated)?;
        let path = root.join(relative_path);
        if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
            let mut bytes = serde_json::to_vec_pretty(&generated)?;
            bytes.push(b'\n');
            std::fs::write(&path, bytes)?;
        }
        let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
        assert_eq!(checked, generated, "{relative_path} drifted");
    }
    Ok(())
}

#[test]
fn essential_pipeline_fake_run_admission_rejects_topology_output_and_path_drift() {
    let mut wrong_portfolio = report();
    assert!(validate_benchmark_local_essential_pipeline_fake_runs_report(
        &wrong_portfolio,
        &[PIPELINE_ID, "missing-pipeline"]
    )
    .is_err());

    wrong_portfolio.pipelines[0].topological_order[0] = "bam.other".to_string();
    assert!(validate_benchmark_local_essential_pipeline_fake_runs_report(
        &wrong_portfolio,
        &[PIPELINE_ID]
    )
    .is_err());

    let mut wrong_domain = node();
    wrong_domain.domain = "vcf".to_string();
    assert!(validate_benchmark_local_essential_pipeline_fake_run_node_report(
        Path::new(&format!("{ROOT}/{PIPELINE_ID}")),
        &wrong_domain
    )
    .is_err());

    let mut missing_output = node();
    missing_output.outputs[0].exists = false;
    missing_output.created_output_count = 1;
    assert!(validate_benchmark_local_essential_pipeline_fake_run_node_report(
        Path::new(&format!("{ROOT}/{PIPELINE_ID}")),
        &missing_output
    )
    .is_err());

    let mut escaping = pipeline();
    escaping.nodes[0].metrics_path = "artifacts/outside/metrics.json".to_string();
    assert!(validate_benchmark_local_essential_pipeline_fake_run_pipeline_report(
        Path::new(ROOT),
        &escaping
    )
    .is_err());
}

#[test]
fn essential_pipeline_node_metrics_admission_rejects_binding_and_timing_drift() {
    let mut wrong_count = metrics();
    wrong_count.upstream_input_count = 2;
    assert!(
        validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&wrong_count).is_err()
    );

    let mut false_external = metrics();
    false_external.external_bindings[0].producer_node_id = Some("producer".to_string());
    assert!(
        validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&false_external).is_err()
    );

    let mut missing_producer = metrics();
    missing_producer.upstream_bindings[0].producer_node_id = None;
    assert!(validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&missing_producer)
        .is_err());

    let mut duplicate_input = metrics();
    duplicate_input.external_bindings[0].input_id = "validated_reads".to_string();
    assert!(validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&duplicate_input)
        .is_err());

    let mut invalid_time = metrics();
    invalid_time.simulated_elapsed_seconds = f64::NAN;
    assert!(
        validate_benchmark_local_essential_pipeline_fake_run_node_metrics(&invalid_time).is_err()
    );
}
