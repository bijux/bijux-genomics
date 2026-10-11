use bijux_dna_bench_model::{
    benchmark_local_hpc_pipeline_node_array_schema,
    validate_benchmark_local_hpc_pipeline_node_array, BenchmarkLocalHpcArrayResources,
    BenchmarkLocalHpcPipelineNodeArrayDependency, BenchmarkLocalHpcPipelineNodeArrayExpectedOutput,
    BenchmarkLocalHpcPipelineNodeArrayReport, BenchmarkLocalHpcPipelineNodeArrayRow,
    BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1,
};

fn resources(factor: u32, minutes: usize) -> BenchmarkLocalHpcArrayResources {
    BenchmarkLocalHpcArrayResources {
        cpus_per_task: factor,
        memory_mb: factor * 1024,
        time_limit: format!("00:{minutes:02}:00"),
        scratch_gb: factor,
    }
}

fn output(output_id: &str, output_path: &str) -> BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
    BenchmarkLocalHpcPipelineNodeArrayExpectedOutput {
        output_id: output_id.to_string(),
        output_path: output_path.to_string(),
    }
}

fn row(index: usize, node_id: &str) -> BenchmarkLocalHpcPipelineNodeArrayRow {
    let job_id = format!("pipeline:vcf-essential:{node_id}");
    let root = format!("artifacts/runs/bench/results/{node_id}");
    let dependency = (index == 1).then(|| BenchmarkLocalHpcPipelineNodeArrayDependency {
        node_id: "map".to_string(),
        job_id_local: "pipeline:vcf-essential:map".to_string(),
        array_index: 0,
        upstream_result_ids: vec!["aligned-bam".to_string()],
        expected_outputs: vec![output(
            "aligned-bam",
            "artifacts/runs/bench/results/map/aligned.bam",
        )],
    });
    BenchmarkLocalHpcPipelineNodeArrayRow {
        array_index: index,
        job_id_local: job_id,
        pipeline_id: "vcf-essential".to_string(),
        node_id: node_id.to_string(),
        domain: "vcf".to_string(),
        stage_id: if index == 0 { "bam.mapping" } else { "vcf.calling" }.to_string(),
        tool_id: if index == 0 { "bwa-mem2" } else { "bcftools" }.to_string(),
        corpus_id: "high-coverage".to_string(),
        asset_profile_id: "grch38".to_string(),
        dependency_job_ids: dependency.iter().map(|entry| entry.job_id_local.clone()).collect(),
        upstream_result_ids: dependency
            .iter()
            .flat_map(|entry| entry.upstream_result_ids.clone())
            .collect(),
        dependencies: dependency.into_iter().collect(),
        expected_outputs: if index == 0 {
            vec![output("aligned-bam", "artifacts/runs/bench/results/map/aligned.bam")]
        } else {
            vec![output("variants-vcf", "artifacts/runs/bench/results/call/variants.vcf.gz")]
        },
        script_path: format!("artifacts/runs/bench/slurm/{node_id}.sbatch"),
        pipeline_node_argv: vec!["bijux-dna".to_string(), "bench".to_string()],
        pipeline_node_command: "bijux-dna bench".to_string(),
        scratch_root: format!("{root}/scratch"),
        input_links_root: format!("{root}/inputs"),
        output_root: format!("{root}/outputs"),
        log_root: format!("{root}/logs"),
        stdout_path: format!("{root}/stdout.log"),
        stderr_path: format!("{root}/stderr.log"),
        resources: resources(if index == 0 { 1 } else { 2 }, index + 10),
    }
}

fn report() -> BenchmarkLocalHpcPipelineNodeArrayReport {
    BenchmarkLocalHpcPipelineNodeArrayReport {
        schema_version: BENCHMARK_LOCAL_HPC_PIPELINE_NODE_ARRAY_SCHEMA_V1.to_string(),
        script_path: "artifacts/runs/bench/hpc-dry-run/slurm/pipeline-node-array.sbatch"
            .to_string(),
        manifest_path: "artifacts/runs/bench/hpc-dry-run/slurm/pipeline-node-array-manifest.json"
            .to_string(),
        wrapper_log_root: "artifacts/runs/bench/hpc-dry-run/slurm/logs/pipeline-node-array"
            .to_string(),
        array_spec: "0-1".to_string(),
        pipeline_count: 1,
        pipeline_job_count: 2,
        dependency_count: 1,
        resource_envelope: resources(2, 11),
        rows: vec![row(0, "map"), row(1, "call")],
    }
}

#[test]
fn local_hpc_pipeline_node_array_round_trips_and_matches_schema() -> anyhow::Result<()> {
    let report = report();
    validate_benchmark_local_hpc_pipeline_node_array(&report)?;
    assert_eq!(
        serde_json::from_slice::<BenchmarkLocalHpcPipelineNodeArrayReport>(&serde_json::to_vec(
            &report
        )?)?,
        report
    );
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("crate outside workspace"))?;
    let generated = serde_json::to_value(benchmark_local_hpc_pipeline_node_array_schema())?;
    let path = root.join("benchmarks/contracts/schemas/local-hpc-pipeline-node-array.schema.json");
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local HPC pipeline-node array schema drifted");
    Ok(())
}

#[test]
fn local_hpc_pipeline_node_array_rejects_lineage_count_and_envelope_drift() {
    let mut lineage = report();
    lineage.rows[1].dependencies[0].array_index = 1;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&lineage).is_err());

    let mut count = report();
    count.dependency_count = 0;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&count).is_err());

    let mut envelope = report();
    envelope.resource_envelope.memory_mb = 1024;
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&envelope).is_err());

    let mut projection = report();
    projection.rows[1].dependencies[0].upstream_result_ids = vec!["unknown-result".to_string()];
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&projection).is_err());

    let mut upstream_closure = report();
    upstream_closure.rows[1].upstream_result_ids.clear();
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&upstream_closure).is_err());
}

#[test]
fn local_hpc_pipeline_node_array_rejects_overflowing_resource_durations() {
    let mut row = report();
    row.rows[0].resources.time_limit = "18446744073709551615:00:00".to_string();
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&row).is_err());
    let mut envelope = report();
    envelope.resource_envelope.time_limit = "18446744073709551615:00:00".to_string();
    assert!(validate_benchmark_local_hpc_pipeline_node_array(&envelope).is_err());
}

#[test]
fn pipeline_array_join_rejects_ambiguous_sources_and_identity_drift() -> anyhow::Result<()> {
    use bijux_dna_bench_model::{
        build_hpc_pipeline_array_report, BenchmarkLocalAllDomainSlurmSubmitJob,
        BenchmarkLocalHpcScratchLayoutJob, BenchmarkLocalPipelineDagValidationReport,
        HpcPipelineArrayInputs, BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,
    };
    use serde_json::json;
    use std::collections::BTreeMap;
    let job: BenchmarkLocalAllDomainSlurmSubmitJob = serde_json::from_value(json!({
    "job_id_local":"pipeline:vcf-essential:call","result_id":null,"pipeline_id":"vcf-essential","node_id":"call",
    "domain":"vcf","stage_id":"vcf.calling","tool_id":"bcftools","corpus_id":"run","asset_profile_id":"profile",
    "script_path":"artifacts/runs/bench/job.sbatch","stdout":"out","stderr":"err","outputs":["artifacts/runs/bench/result.vcf"],"dependencies":[],
    "resources":{"cpus_per_task":1,"memory_mb":1024,"time_limit":"00:10:00"}
    }))?;
    let scratch: BenchmarkLocalHpcScratchLayoutJob = serde_json::from_value(json!({
    "job_id_local":job.job_id_local,"result_id":null,"job_kind":"essential_pipeline_node","pipeline_id":job.pipeline_id,"node_id":job.node_id,
    "domain":"vcf","stage_id":"vcf.calling","tool_id":"bcftools","corpus_id":"run","asset_profile_id":"profile",
    "scratch_root":"scratch","input_links_root":"inputs","output_root":"outputs","log_root":"logs","stdout_path":"out","stderr_path":"err",
    "resources":{"cpus_per_task":1,"memory_mb":1024,"time_limit":"00:10:00","scratch_gb":1},
    "cleanup_policy":{"policy_id":"preserve","remove_scratch_after_successful_sync":false,"retain_scratch_on_failure":true,"preserve_output_root":true,"preserve_log_root":true,"preserve_staged_inputs":true},"input_links":[]
    }))?;
    let pipeline: BenchmarkLocalPipelineDagValidationReport = serde_json::from_value(json!({
    "schema_version":BENCHMARK_LOCAL_PIPELINE_DAG_VALIDATION_SCHEMA_V1,"config_path":"pipeline.toml","output_path":"dag.json","pipeline_id":"vcf-essential","domain":"vcf","default_corpus_id":"run","summary":"one calling node",
    "reference_context":null,"execution_context":null,"project_source_count":0,"project_sources":[],"ancient_dna_policy":null,"reference_assets":null,"variant_assets":null,
    "node_count":1,"edge_count":0,"acyclic":true,"valid":true,"topological_order":["call"],"validation_profiles":[],
    "nodes":[{"node_id":"call","stage_id":"vcf.calling","readiness_kind":"dry_run","dependency_count":0,"external_input_count":1,"upstream_input_count":0,"output_count":1,"depends_on":[],"external_inputs":["input-bam"],"upstream_inputs":[],"outputs":["variants"]}]
    }))?;
    let jobs = [job];
    let scratch = [scratch];
    let pipelines = BTreeMap::from([("vcf-essential".to_string(), pipeline)]);
    let build = |jobs: &[BenchmarkLocalAllDomainSlurmSubmitJob],
                 scratch: &[BenchmarkLocalHpcScratchLayoutJob],
                 pipelines: &BTreeMap<String, BenchmarkLocalPipelineDagValidationReport>,
                 order: &[&str]| {
        build_hpc_pipeline_array_report(HpcPipelineArrayInputs {
            jobs,
            scratch,
            pipelines,
            pipeline_order: order,
            script_path: "artifacts/runs/bench/array.sbatch".into(),
            manifest_path: "artifacts/runs/bench/array-manifest.json".into(),
            wrapper_log_root: "artifacts/runs/bench/logs/array".into(),
        })
    };
    let order = ["vcf-essential"];
    let accepted = build(&jobs, &scratch, &pipelines, &order)?;
    assert_eq!(accepted.rows[0].expected_outputs[0].output_id, "variants");
    assert_eq!(accepted.rows[0].expected_outputs[0].output_path, jobs[0].outputs[0]);
    assert!(build(&[], &scratch, &pipelines, &order).is_err());
    assert!(build(&jobs, &[], &pipelines, &order).is_err());
    assert!(build(&[jobs[0].clone(), jobs[0].clone()], &scratch, &pipelines, &order).is_err());
    assert!(build(&jobs, &[scratch[0].clone(), scratch[0].clone()], &pipelines, &order).is_err());
    assert!(build(&jobs, &scratch, &pipelines, &[]).is_err());
    assert!(build(&jobs, &scratch, &pipelines, &["vcf-essential", "vcf-essential"]).is_err());
    let mut drift = scratch.clone();
    drift[0].tool_id = "other".into();
    assert!(build(&jobs, &drift, &pipelines, &order).is_err());
    let mut drift = pipelines.clone();
    drift
        .get_mut("vcf-essential")
        .ok_or_else(|| anyhow::anyhow!("missing fixture pipeline"))?
        .valid = false;
    assert!(build(&jobs, &scratch, &drift, &order).is_err());
    let mut drift = pipelines.clone();
    drift
        .get_mut("vcf-essential")
        .ok_or_else(|| anyhow::anyhow!("missing fixture pipeline"))?
        .pipeline_id = "other".into();
    assert!(build(&jobs, &scratch, &drift, &order).is_err());
    Ok(())
}
