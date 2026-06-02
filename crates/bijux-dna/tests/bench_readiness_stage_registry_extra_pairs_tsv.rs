#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_stage_registry_extra_pairs_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-stage-registry-extra-pairs"])
        .output()
        .expect("run cli");

    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let tsv_path = repo_root.join("target/bench-readiness/stage-registry-extra-pairs.tsv");
    assert!(tsv_path.is_file(), "stage registry extra pairs TSV must exist");

    let tsv = std::fs::read_to_string(&tsv_path).expect("read stage registry extra pairs");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("domain\tstage_id\ttool_id\tcontract_status\tregistry_sources\tregistered_stage_ids\tintentional_override_status\tintentional_override_reason\treason")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 2, "TSV must retain the governed stage-registry drift row count");
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.bias_mitigation\tsamtools\tpair_missing_from_contract\tstages.primary_tools\tbam.align,bam.coverage,bam.duplication_metrics,bam.endogenous_content,bam.filter,bam.length_filter,bam.mapping_summary,bam.mapq_filter,bam.markdup,bam.qc_pre,bam.validate\tnone\t\tstage registry admits `bam.bias_mitigation` / `samtools` inside the benchmark scope but domain contracts do not; contract status: pair_missing_from_contract; registry sources: stages.primary_tools; admitted contract stages for `samtools`: bam.align, bam.coverage, bam.duplication_metrics, bam.endogenous_content, bam.filter, bam.length_filter, bam.mapping_summary, bam.mapq_filter, bam.markdup, bam.qc_pre, bam.validate; stage rationale: <none>"
        }),
        "TSV must retain the governed bam.bias_mitigation / samtools registry-only row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam\tbam.haplogroups\tsamtools\tpair_missing_from_contract\tstages.primary_tools\tbam.align,bam.coverage,bam.duplication_metrics,bam.endogenous_content,bam.filter,bam.length_filter,bam.mapping_summary,bam.mapq_filter,bam.markdup,bam.qc_pre,bam.validate\tnone\t\tstage registry admits `bam.haplogroups` / `samtools` inside the benchmark scope but domain contracts do not; contract status: pair_missing_from_contract; registry sources: stages.primary_tools; admitted contract stages for `samtools`: bam.align, bam.coverage, bam.duplication_metrics, bam.endogenous_content, bam.filter, bam.length_filter, bam.mapping_summary, bam.mapq_filter, bam.markdup, bam.qc_pre, bam.validate; stage rationale: <none>"
        }),
        "TSV must retain the governed bam.haplogroups / samtools registry-only row"
    );
    assert!(
        !rows.iter().any(|row| row.starts_with("bam\tbam.qc_pre\tmultiqc\t")),
        "TSV must no longer retain a governed bam.qc_pre / multiqc registry-only row"
    );
}
