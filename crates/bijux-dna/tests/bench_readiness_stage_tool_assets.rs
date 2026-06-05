#![allow(clippy::expect_used)]

use std::process::Command;

#[path = "contracts/banks/bank_fixtures.rs"]
mod support;

fn run_cli(args: &[&str]) -> std::process::Output {
    let _cwd_guard = support::CWD_LOCK.lock().expect("cwd lock");
    let _env_guard = support::EnvGuard::new().expect("capture env");
    let _crate_root = support::crate_root("bijux-dna").expect("crate root");
    let repo_root = support::repo_root().expect("repo root");
    let home = tempfile::tempdir().expect("tempdir");

    Command::new(env!("CARGO_BIN_EXE_bijux-dna"))
        .current_dir(&repo_root)
        .env("HOME", home.path())
        .env("BIJUX_SKIP_QA", "1")
        .env("BIJUX_ALLOW_SILVER", "1")
        .env("BIJUX_SKIP_IMAGE_CHECK", "1")
        .args(args)
        .output()
        .expect("run cli")
}

fn run_cli_json(args: &[&str]) -> serde_json::Value {
    let output = run_cli(args);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("parse stdout as json")
}

#[test]
fn bench_readiness_stage_tool_assets_reports_governed_asset_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-tool-assets", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_assets.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-tool-assets.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_ready_command_assets")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(
        payload.get("declared_stage_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(13)
    );
    assert_eq!(payload.get("asset_id_row_count").and_then(serde_json::Value::as_u64), Some(20));
    assert_eq!(payload.get("unique_asset_id_count").and_then(serde_json::Value::as_u64), Some(10));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(7)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(13)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("taxonomy_database_root"))
            .and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("reference_fasta"))
            .and_then(serde_json::Value::as_u64),
        Some(6)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert!(rows.iter().all(|row| {
        row.get("asset_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("asset_path")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    let taxonomy = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("kraken2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("taxonomy_database_root")
        })
        .expect("fastq taxonomy kraken2 asset row");
    assert_eq!(
        taxonomy.get("asset_id").and_then(serde_json::Value::as_str),
        Some("taxonomy_reference")
    );
    assert_eq!(
        taxonomy.get("asset_path").and_then(serde_json::Value::as_str),
        Some("assets/reference/taxonomy/references/mock_community_taxonomy")
    );

    let rrna = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.deplete_rrna")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("sortmerna")
        })
        .expect("fastq deplete-rrna asset row");
    assert_eq!(rrna.get("asset_role").and_then(serde_json::Value::as_str), Some("rrna_reference"));
    assert_eq!(
        rrna.get("asset_id").and_then(serde_json::Value::as_str),
        Some("sortmerna_common_rrna_reference")
    );

    let contamination_panel = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("verifybamid2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_panel")
        })
        .expect("bam contamination panel row");
    assert_eq!(
        contamination_panel.get("asset_id").and_then(serde_json::Value::as_str),
        Some("human_like_contamination_panel")
    );

    let haplogroups = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.haplogroups")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_panel")
        })
        .expect("bam haplogroups panel row");
    assert_eq!(
        haplogroups.get("asset_id").and_then(serde_json::Value::as_str),
        Some("human-like-y-hg38-mini")
    );

    let genotyping_sites = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
                && row.get("asset_role").and_then(serde_json::Value::as_str) == Some("sites_vcf")
        })
        .expect("bam genotyping sites row");
    assert_eq!(
        genotyping_sites.get("asset_id").and_then(serde_json::Value::as_str),
        Some("human_like_genotyping_candidate_sites")
    );

    let recalibration = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("gatk")
                && row.get("asset_role").and_then(serde_json::Value::as_str) == Some("known_sites")
        })
        .expect("bam recalibration known-sites row");
    assert_eq!(
        recalibration.get("asset_id").and_then(serde_json::Value::as_str),
        Some("human_like_recalibration_known_sites")
    );
}
