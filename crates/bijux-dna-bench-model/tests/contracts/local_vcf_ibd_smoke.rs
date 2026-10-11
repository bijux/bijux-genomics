use bijux_dna_bench_model::{
    benchmark_local_vcf_ibd_smoke_schema, validate_benchmark_local_vcf_ibd_smoke_report,
    BenchmarkLocalVcfIbdInsufficientOverlapProbe, BenchmarkLocalVcfIbdSmokeReport,
    BenchmarkLocalVcfIbdSmokeRow, BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1,
};

fn report() -> BenchmarkLocalVcfIbdSmokeReport {
    BenchmarkLocalVcfIbdSmokeReport {
        schema_version: BENCHMARK_LOCAL_VCF_IBD_SMOKE_SCHEMA_V1.to_string(),
        command: "bijux-dna bench local run-vcf-ibd-smoke --tool-id germline".to_string(),
        stage_id: "vcf.ibd".to_string(),
        tool_id: "germline".to_string(),
        corpus_id: "vcf_production_regression".to_string(),
        input_fixture_id: "vcf_mini_multisample_cohort".to_string(),
        fixture_manifest_path: "benchmarks/tests/fixtures/corpora/vcf-mini/manifest.toml"
            .to_string(),
        input_vcf_path: "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/input/ibd_input.vcf"
            .to_string(),
        sample_metadata_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/input/sample_metadata.tsv"
                .to_string(),
        output_root: "artifacts/runs/bench/local-smoke/vcf.ibd/germline".to_string(),
        ibd_tsv_path: "artifacts/runs/bench/local-smoke/vcf.ibd/germline/ibd.tsv".to_string(),
        ibd_json_path: "artifacts/runs/bench/local-smoke/vcf.ibd/germline/ibd.json".to_string(),
        source_ibd_input_path: "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_input.tsv"
            .to_string(),
        source_ibd_segments_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_segments.tsv".to_string(),
        source_ibd_merged_segments_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_merged_segments.tsv".to_string(),
        source_ibd_filtered_segments_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_filtered_segments.tsv"
                .to_string(),
        source_ibd_summary_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_summary.json".to_string(),
        source_ibd_metrics_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_ibd_metrics.json".to_string(),
        source_logs_path: "artifacts/runs/bench/local-smoke/vcf.ibd/germline/source_logs.txt".to_string(),
        stage_result_manifest_path:
            "artifacts/runs/bench/local-smoke/vcf.ibd/germline/stage-result.json".to_string(),
        started_at: "unix:1".to_string(),
        finished_at: "unix:2".to_string(),
        elapsed_seconds: 0.5,
        exit_code: 0,
        execution_mode: "fallback_proxy".to_string(),
        germline_tool_ok: false,
        ibdseq_tool_ok: false,
        ibdhap_tool_ok: false,
        pair_count: 1,
        rows: vec![BenchmarkLocalVcfIbdSmokeRow {
            sample_a: "sample_a".to_string(),
            sample_b: "sample_b".to_string(),
            segment_count: 2,
            total_length: 2.08,
            overlap_marker_count: 2,
            status: "complete".to_string(),
        }],
        insufficient_overlap_probe: BenchmarkLocalVcfIbdInsufficientOverlapProbe {
            input_vcf_path:
                "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_sparse_overlap.vcf"
                    .to_string(),
            ibd_status: "insufficient_marker_overlap".to_string(),
            insufficient_data_reason: "no_pairs_met_min_marker_or_length_threshold".to_string(),
            overlap_marker_count: 1,
            filtered_segment_count: 0,
            source_ibd_summary_path:
                "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_ibd_summary.json"
                    .to_string(),
            source_ibd_filtered_segments_path:
                "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_ibd_filtered_segments.tsv"
                    .to_string(),
            unrelated_stage_id: "vcf.roh".to_string(),
            unrelated_stage_status: "complete".to_string(),
            unrelated_stage_segment_count: 2,
            source_roh_summary_path:
                "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_roh_summary.json"
                    .to_string(),
            source_roh_segments_path:
                "artifacts/runs/bench/local-smoke/vcf.ibd/germline/artifacts/probe/probe_source_roh_segments.tsv"
                    .to_string(),
        },
        status: "complete".to_string(),
    }
}

#[test]
fn local_vcf_ibd_smoke_contract_accepts_fallback_and_real_tool_evidence() -> anyhow::Result<()> {
    let fallback = report();
    validate_benchmark_local_vcf_ibd_smoke_report(&fallback)?;

    let mut real_tool = fallback;
    real_tool.execution_mode = "real_tool".to_string();
    real_tool.germline_tool_ok = true;
    validate_benchmark_local_vcf_ibd_smoke_report(&real_tool)?;
    Ok(())
}

#[test]
fn local_vcf_ibd_smoke_contract_rejects_pair_backend_and_probe_drift() {
    let mut pair_drift = report();
    pair_drift.rows[0].sample_a = "sample_b".to_string();
    assert!(validate_benchmark_local_vcf_ibd_smoke_report(&pair_drift).is_err());

    let mut backend_drift = report();
    backend_drift.execution_mode = "real_tool".to_string();
    assert!(validate_benchmark_local_vcf_ibd_smoke_report(&backend_drift).is_err());

    let mut probe_drift = report();
    probe_drift.insufficient_overlap_probe.filtered_segment_count = 1;
    assert!(validate_benchmark_local_vcf_ibd_smoke_report(&probe_drift).is_err());
}

#[test]
fn local_vcf_ibd_smoke_schema_matches_model_authority() -> anyhow::Result<()> {
    let repository_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .ok_or_else(|| anyhow::anyhow!("benchmark model crate is outside the workspace"))?;
    let path = repository_root.join("benchmarks/contracts/schemas/local-vcf-ibd-smoke.schema.json");
    let generated = serde_json::to_value(benchmark_local_vcf_ibd_smoke_schema())?;
    if std::env::var_os("BIJUX_UPDATE_GENERATED_SCHEMAS").is_some() {
        let mut bytes = serde_json::to_vec_pretty(&generated)?;
        bytes.push(b'\n');
        std::fs::write(&path, bytes)?;
    }
    let checked: serde_json::Value = serde_json::from_slice(&std::fs::read(path)?)?;
    assert_eq!(checked, generated, "local VCF IBD smoke schema drifted from its model owner");
    Ok(())
}

#[test]
fn local_vcf_ibd_smoke_record_rejects_unknown_fields() -> anyhow::Result<()> {
    let mut value = serde_json::to_value(report())?;
    value["legacy_status"] = serde_json::json!("passed");
    assert!(serde_json::from_value::<BenchmarkLocalVcfIbdSmokeReport>(value).is_err());
    Ok(())
}
