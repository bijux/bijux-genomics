use bijux_dna_bench_vcf::adapters::observations::VcfAdapterObservations;
use bijux_dna_bench_vcf_evidence::model::outputs::{
    VcfAdapterOutputCoverageReport, VcfAdapterOutputCoverageStatus,
};

use std::path::PathBuf;

use bijux_dna_bench_vcf::evidence::serving::collect_vcf_tool_serving_map_rows;

use bijux_dna_bench_vcf::evidence::output_coverage::{
    render_vcf_adapter_output_coverage, DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH,
    VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION,
};

fn repo_root() -> PathBuf {
    crate::support::repo_root()
}

#[test]
fn vcf_adapter_output_coverage_tracks_governed_rows() {
    let root = repo_root();
    let inputs = crate::support::inputs(&root);
    let observations = VcfAdapterObservations::new(&inputs, &root);
    let report = render_vcf_adapter_output_coverage(
        &observations,
        "local-benchmark-dry-run",
        &PathBuf::from(DEFAULT_VCF_ADAPTER_OUTPUT_COVERAGE_PATH),
    )
    .expect("render VCF adapter output coverage");

    assert_eq!(report.schema_version, VCF_ADAPTER_OUTPUT_COVERAGE_SCHEMA_VERSION);
    assert_eq!(report.row_count, 39);
    let benchmark_ready_row_count = collect_vcf_tool_serving_map_rows(&inputs)
        .expect("collect VCF tool-serving rows")
        .into_iter()
        .filter(|row| row.benchmark_status == "benchmark_ready")
        .count();
    assert_eq!(report.benchmark_ready_row_count, benchmark_ready_row_count);
    assert_eq!(report.benchmark_ready_complete_row_count, benchmark_ready_row_count);
    assert_eq!(report.benchmark_ready_incomplete_row_count, 0);
    assert_eq!(report.complete_row_count + report.incomplete_row_count, report.row_count);

    assert_variants_coverage(&report);
    assert_panels_coverage(&report);
    assert_population_coverage(&report);
    assert_likelihoods_coverage(&report);
}

fn assert_variants_coverage(report: &VcfAdapterOutputCoverageReport) {
    let call = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.call" && row.tool_id == "bcftools")
        .expect("bcftools call row");
    assert_eq!(call.status, VcfAdapterOutputCoverageStatus::Complete);
    assert_eq!(call.benchmark_status, "benchmark_ready");
    assert!(call.index_outputs.iter().any(|entry| entry.contains(".tbi")));

    let stats = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.stats" && row.tool_id == "bcftools")
        .expect("bcftools stats row");
    assert_eq!(stats.status, VcfAdapterOutputCoverageStatus::Complete);
    assert!(stats.index_outputs.is_empty());
    assert!(stats.normalized_metrics.iter().any(|entry| entry.starts_with("stats_json=")));

    let qc = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.qc" && row.tool_id == "bcftools")
        .expect("bcftools qc row");
    assert_eq!(qc.status, VcfAdapterOutputCoverageStatus::Complete);
    assert_eq!(qc.benchmark_status, "benchmark_ready");
    assert_eq!(qc.raw_outputs.len(), 6);
    assert!(qc.normalized_metrics.iter().any(|entry| entry.starts_with("qc_report=")));
}

fn assert_panels_coverage(report: &VcfAdapterOutputCoverageReport) {
    let reference_panel = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.prepare_reference_panel" && row.tool_id == "bcftools")
        .expect("bcftools reference-panel row");
    assert_eq!(reference_panel.status, VcfAdapterOutputCoverageStatus::Complete);
    assert_eq!(reference_panel.benchmark_status, "benchmark_ready");
    assert!(reference_panel.index_outputs.iter().any(|entry| entry.contains(".tbi")));
    assert!(reference_panel
        .normalized_metrics
        .iter()
        .any(|entry| entry.starts_with("chunks_json=")));

    let shapeit5 = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.phasing" && row.tool_id == "shapeit5")
        .expect("shapeit5 row");
    assert_eq!(shapeit5.benchmark_status, "benchmark_ready");
    assert_eq!(shapeit5.status, VcfAdapterOutputCoverageStatus::Complete);
    assert!(shapeit5.index_outputs.iter().any(|entry| entry.contains(".tbi")));

    let impute = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.impute" && row.tool_id == "beagle")
        .expect("beagle impute row");
    assert_eq!(impute.benchmark_status, "benchmark_ready");
    assert_eq!(impute.status, VcfAdapterOutputCoverageStatus::Complete);
    assert!(impute.index_outputs.iter().any(|entry| entry.contains(".tbi")));
}

fn assert_population_coverage(report: &VcfAdapterOutputCoverageReport) {
    let demography = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.demography" && row.tool_id == "ibdne")
        .expect("ibdne row");
    assert_eq!(demography.status, VcfAdapterOutputCoverageStatus::Complete);
    assert!(demography.index_outputs.is_empty());
    assert!(demography.manifest.ends_with("/stage-result.json"));

    let roh_row_count = report
        .rows
        .iter()
        .filter(|row| row.stage_id == "vcf.roh" && row.tool_id == "plink2")
        .count();
    assert_eq!(roh_row_count, 1);
}

fn assert_likelihoods_coverage(report: &VcfAdapterOutputCoverageReport) {
    let angsd_gl = report
        .rows
        .iter()
        .find(|row| row.stage_id == "vcf.call_gl" && row.tool_id == "angsd")
        .expect("angsd call_gl row");
    assert_eq!(angsd_gl.status, VcfAdapterOutputCoverageStatus::Complete);
    assert!(angsd_gl.missing_declarations.is_empty());
    assert!(angsd_gl.index_outputs.iter().any(|entry| entry.contains(".tbi")));
}

#[test]
fn invalid_run_identity_refuses_before_coverage_effects() -> anyhow::Result<()> {
    let inputs = crate::support::inputs(&repo_root());
    let root = tempfile::tempdir()?;
    let observations = VcfAdapterObservations::new(&inputs, root.path());
    let output = root.path().join("reports/coverage.tsv");
    for run_id in ["", "../escape", ".", "..", " padded", "nested/run", "nested\\run", "nul\0run"] {
        let error = render_vcf_adapter_output_coverage(&observations, run_id, &output)
            .err()
            .map(|e| e.to_string());
        assert!(error.is_some_and(|message| message.contains("run identity")));
        assert_eq!(std::fs::read_dir(root.path())?.count(), 0);
    }
    std::fs::write(root.path().join("existing.tsv"), b"retained evidence")?;
    assert!(render_vcf_adapter_output_coverage(
        &observations,
        "../escape",
        &root.path().join("existing.tsv")
    )
    .is_err());
    assert_eq!(std::fs::read(root.path().join("existing.tsv"))?, b"retained evidence");
    Ok(())
}
