#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_comparable_metrics_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-comparable-metrics"])
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
    assert_eq!(rendered_path.trim(), "benchmarks/readiness/bam-comparable-metrics.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM comparable metrics TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\tcomparison_contract_status\ttool_count\ttool_ids\tdefault_tool_id\tcorpus_status\tshared_metric_field_count\tshared_metric_fields\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 15);
    assert!(
        rows.iter().any(|row| {
            row == &"bam.validate\tdeclared\t3\tbamtools,bedtools,samtools\tsamtools\tfixture:corpus-01-bam-mini\t4\tvalidation_status,validation_errors,validation_warnings,input_bam_identity\tstage `bam.validate` publishes governed shared comparable metrics `validation_status, validation_errors, validation_warnings, input_bam_identity` for same-stage tool comparison while corpus routing remains `fixture:corpus-01-bam-mini`"
        }),
        "TSV must retain the governed BAM validation comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.coverage\tdeclared\t3\tbedtools,mosdepth,samtools\tmosdepth\tfixture:corpus-01-bam-mini\t5\tmean_depth,breadth_1x,covered_bases,observed_region_count,region_ids\tstage `bam.coverage` publishes governed shared comparable metrics `mean_depth, breadth_1x, covered_bases, observed_region_count, region_ids` for same-stage tool comparison while corpus routing remains `fixture:corpus-01-bam-mini`"
        }),
        "TSV must retain the governed BAM coverage comparable row"
    );
    assert!(
        rows.iter().any(|row| {
            row == &"bam.damage\tdeclared\t6\taddeam,damageprofiler,mapdamage2,ngsbriggs,pmdtools,pydamage\tmapdamage2\tfixture:corpus-01-adna-damage-mini\t5\tterminal_c_to_t_5p,terminal_g_to_a_3p,damage_signal,runtime_s,memory_mb\tstage `bam.damage` publishes governed shared comparable metrics `terminal_c_to_t_5p, terminal_g_to_a_3p, damage_signal, runtime_s, memory_mb` for same-stage tool comparison while corpus routing remains `fixture:corpus-01-adna-damage-mini`"
        }),
        "TSV must retain the governed BAM damage comparable row"
    );
}
