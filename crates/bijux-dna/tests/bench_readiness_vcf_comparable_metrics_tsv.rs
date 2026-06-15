#![allow(clippy::expect_used, clippy::too_many_lines)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_vcf_comparable_metrics_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-vcf-comparable-metrics"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/vcf-comparable-metrics.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read VCF comparable metrics TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some("stage_id\tmetric_id\tmetric_name\tunit\tdirection\trequired\ttools_covered")
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 33);
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.call_gl\tsites_with_likelihoods\tsites with likelihoods\tsites\thigher_is_better\ttrue\tangsd,bcftools"
        }),
        "TSV must retain the governed VCF GL comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.phasing\tswitch_error_proxy\tswitch error proxy\tfraction\tlower_is_better\ttrue\tbeagle,eagle,shapeit5"
        }),
        "TSV must retain the governed VCF phasing comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"vcf.impute\tmasked_truth_match_count\tmasked-truth matches\tsites\thigher_is_better\ttrue\tbeagle,glimpse,impute5,minimac4"
        }),
        "TSV must retain the governed VCF imputation comparable row"
    );
}
