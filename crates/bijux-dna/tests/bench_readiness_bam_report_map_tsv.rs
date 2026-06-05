#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

#[test]
fn bench_readiness_bam_report_map_writes_governed_tsv_columns() {
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
        .args(["bench", "readiness", "render-bam-report-map"])
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
    assert_eq!(rendered_path.trim(), "target/bench-readiness/bam-report-map.tsv");

    let tsv = std::fs::read_to_string(repo_root.join(rendered_path.trim()))
        .expect("read BAM report map TSV");
    let mut lines = tsv.lines();
    assert_eq!(
        lines.next(),
        Some(
            "stage_id\tcanonical_stage_rank\treadiness_kind\tcriticality\tanchor_tool_id\tanchor_support_status\treport_section_id\treport_section_title\tsummary_table_id\tsummary_table_title\tworkflow_branch_id\tscientific_context_required\treport_focus\treason"
        )
    );
    let rows = lines.collect::<Vec<_>>();
    assert_eq!(rows.len(), 24, "TSV must retain every BAM benchmark-ready stage");
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam.align\t1\tdry_or_smoke\toptional\tbwa\tsupported\talignment_intake\tAlignment Intake\talignment_baseline\tAlignment Baseline\t\t\talignment provenance, validation status, and intake QC baselines\t"
            )
        }),
        "TSV must retain the governed alignment-intake row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam.contamination\t18\tdry_or_smoke\tessential\tschmutzi\tsupported\tancient_signal\tAncient Signal\tdamage_authenticity\tDamage and Authenticity\tancient_dna_authenticity\t\tdamage evidence, authenticity advisories, and contamination guardrails\t"
            )
        }),
        "TSV must retain the governed ancient-signal contamination row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam.sex\t19\tsmoke\tessential\trxy\tsupported\tsample_identity\tSample Identity\tidentity_inference\tIdentity and Relatedness\tsample_identity\tchromosome_system,minimum_y_sites\tsex inference, haplogroup context, and relatedness interpretation\t"
            )
        }),
        "TSV must retain the governed sample-identity sex row"
    );
    assert!(
        rows.iter().any(|row| {
            row.contains(
                "bam.recalibration\t21\tsmoke\tessential\tgatk\tsupported\tdownstream_readiness\tDownstream Readiness\tvariant_readiness\tVariant and Bias Readiness\tvariant_readiness\t\tbias control, recalibration, and genotyping readiness before downstream inference\t"
            )
        }),
        "TSV must retain the governed downstream-readiness recalibration row"
    );
}
