#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_active_stage_tool_matrix_writes_governed_tsv_file() {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    let output = Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(["bench", "readiness", "render-vcf-active-stage-tool-matrix"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let rendered_path = String::from_utf8(output.stdout).expect("stdout utf8");
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf/vcf-active-stage-tool-matrix.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF active-stage-tool matrix TSV");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\ttool_id\ttool_status\tstage_support_status\tcorpus_id\tasset_profile_id\tadapter_id\tparser_id\tschema_id\tscope_state\tscope_detail\tscope_proof_path\treason"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 42);
    assert!(rows.iter().any(|row| {
        row == &"vcf.call\tbcftools\tproduction\tsupported\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbijux.schemas.bench.vcf-normalized-metrics.call.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.call` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.call_gl\tangsd\tplanned\tsupported\tvcf_production_regression\tbam_bundle\tvcf.adapter.calling\tvcf.parser.call_summary\tbijux.schemas.bench.vcf-normalized-metrics.call-gl.v1\tremoved_from_scope\tbenchmark_not_ready\tbenchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json\tbinding `vcf.call_gl` / `angsd` is retained for a supported stage but remains outside active scope because it is not benchmark ready"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.impute\tbeagle\tproduction\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.impute.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.impute` / `beagle` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.admixture\tplink2\texperimental,production\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.admixture.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.admixture` / `plink2` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.pca\teigensoft\texperimental,production\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.pca` / `eigensoft` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.pca\tplink2\texperimental,production\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.population_structure\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.pca.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.pca` / `plink2` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.imputation_metrics\tbeagle\tproduction\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.report_json\tbijux.schemas.bench.vcf-normalized-metrics.imputation-metrics.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.imputation_metrics` / `beagle` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.prepare_reference_panel\tbcftools\tproduction\tsupported\tvcf_production_regression\tvcf_reference_panel\tvcf.adapter.reference_panel\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.prepare-reference-panel.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.prepare_reference_panel` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.qc\tbcftools\tproduction\tsupported\tvcf_production_regression\tvcf_cohort\tvcf.adapter.quality_control\tvcf.parser.qc_report\tbijux.schemas.bench.vcf-normalized-metrics.qc.v1\tactive\tactive\tbenchmarks/readiness/all-domains/active-stage-tool-matrix.tsv\tbinding `vcf.qc` / `bcftools` is part of the governed all-domain active benchmark matrix"
    }));
    assert!(rows.iter().any(|row| {
        row == &"vcf.phasing\teagle\texperimental,planned\tsupported\tvcf_production_regression\tvcf_cohort_with_panel\tvcf.adapter.panel_workflow\tvcf.parser.vcf_output\tbijux.schemas.bench.vcf-normalized-metrics.phasing.v1\tremoved_from_scope\tbenchmark_not_ready\tbenchmarks/readiness/all-domains/no-not-benchmark-ready-rows.json\tbinding `vcf.phasing` / `eagle` is retained for a supported stage but remains outside active scope because it is not benchmark ready"
    }));
}
