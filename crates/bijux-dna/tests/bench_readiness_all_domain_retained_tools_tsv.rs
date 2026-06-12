#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_all_domain_retained_tools_writes_governed_tsv_file() {
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
        .args(["bench", "readiness", "render-all-domain-retained-tools"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/all-domains/retained-tools.tsv");

    let payload = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read all-domain retained tools");
    let mut lines = payload.lines();
    assert_eq!(
        lines.next(),
        Some(
            "tool_id\tdomains\tactive_stage_count\tbenchmark_ready_stage_count\tactive_binding_count\tbenchmark_ready_binding_count\tbenchmark_statuses\tactive_stage_ids\tbenchmark_ready_stage_ids"
        )
    );

    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 69);
    assert!(rows.iter().any(|row| {
        row == &"kraken2\tfastq\t1\t1\t1\t1\tbenchmark_ready\tfastq.screen_taxonomy\tfastq.screen_taxonomy"
    }));
    assert!(rows.iter().any(|row| {
        row == &"bcftools\tvcf\t11\t11\t11\t11\tbenchmark_ready\tvcf.call,vcf.call_diploid,vcf.call_gl,vcf.call_pseudohaploid,vcf.damage_filter,vcf.filter,vcf.gl_propagation,vcf.postprocess,vcf.prepare_reference_panel,vcf.qc,vcf.stats\tvcf.call,vcf.call_diploid,vcf.call_gl,vcf.call_pseudohaploid,vcf.damage_filter,vcf.filter,vcf.gl_propagation,vcf.postprocess,vcf.prepare_reference_panel,vcf.qc,vcf.stats"
    }));
    assert!(rows.iter().any(|row| {
        row == &"beagle\tvcf\t2\t2\t2\t2\tbenchmark_ready\tvcf.imputation_metrics,vcf.impute\tvcf.imputation_metrics,vcf.impute"
    }));
    assert!(rows.iter().any(|row| {
        row == &"shapeit5\tvcf\t1\t1\t1\t1\tbenchmark_ready\tvcf.phasing\tvcf.phasing"
    }));
    assert!(
        rows.iter().all(|row| !row.starts_with("star\t") && !row.starts_with("bowtie2_build\t")),
        "retained tools TSV must exclude not-benchmark-ready-only tools"
    );
}
