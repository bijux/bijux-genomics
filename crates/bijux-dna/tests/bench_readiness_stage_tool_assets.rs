#![allow(clippy::expect_used, clippy::too_many_lines)]

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
        Some("benchmarks/configs/local/stage-tool-assets.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("governed_benchmark_command_assets")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(36));
    assert_eq!(
        payload.get("declared_stage_tool_row_count").and_then(serde_json::Value::as_u64),
        Some(19)
    );
    assert_eq!(payload.get("asset_id_row_count").and_then(serde_json::Value::as_u64), Some(36));
    assert_eq!(payload.get("unique_asset_id_count").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(16)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(20)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("database_artifact_id"))
            .and_then(serde_json::Value::as_u64),
        Some(5)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("reference_catalog_id"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("reference_index_artifact_id"))
            .and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("reference_index_output"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
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
        Some(12)
    );
    assert_eq!(
        payload
            .get("asset_role_counts")
            .and_then(|value| value.get("reference_panel"))
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

    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        let taxonomy = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.screen_taxonomy")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("asset_role").and_then(serde_json::Value::as_str)
                        == Some("taxonomy_database_root")
            })
            .expect("fastq taxonomy asset row");
        assert_eq!(
            taxonomy.get("asset_id").and_then(serde_json::Value::as_str),
            Some("taxonomy_reference")
        );
        assert_eq!(
            taxonomy.get("asset_path").and_then(serde_json::Value::as_str),
            Some("assets/reference/taxonomy/references/mock_community_taxonomy")
        );
    }

    let rrna_reference = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.deplete_rrna")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("sortmerna")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("rrna_reference")
        })
        .expect("fastq deplete-rrna asset row");
    assert_eq!(
        rrna_reference.get("asset_id").and_then(serde_json::Value::as_str),
        Some("sortmerna_common_rrna_reference")
    );
    let rrna_database_artifact = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.deplete_rrna")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("sortmerna")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("database_artifact_id")
        })
        .expect("fastq deplete-rrna database artifact row");
    assert_eq!(
        rrna_database_artifact.get("asset_id").and_then(serde_json::Value::as_str),
        Some("sortmerna_common_rrna_reference")
    );
    let host_reference = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.deplete_host")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_catalog_id")
        })
        .expect("fastq deplete-host reference catalog row");
    assert_eq!(
        host_reference.get("asset_id").and_then(serde_json::Value::as_str),
        Some("host_reference")
    );
    let host_index_artifact = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.deplete_host")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_index_artifact_id")
        })
        .expect("fastq deplete-host index artifact row");
    assert_eq!(
        host_index_artifact.get("asset_id").and_then(serde_json::Value::as_str),
        Some("reference_index")
    );
    let contaminant_reference = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.deplete_reference_contaminants")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_catalog_id")
        })
        .expect("fastq contaminant reference catalog row");
    assert_eq!(
        contaminant_reference.get("asset_id").and_then(serde_json::Value::as_str),
        Some("contaminant_reference")
    );
    let contaminant_index_artifact = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.deplete_reference_contaminants")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_index_artifact_id")
        })
        .expect("fastq contaminant index artifact row");
    assert_eq!(
        contaminant_index_artifact.get("asset_id").and_then(serde_json::Value::as_str),
        Some("reference_index")
    );
    let index_reference_output = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.index_reference")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2_build")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_index_output")
        })
        .expect("fastq index-reference output row");
    assert_eq!(
        index_reference_output.get("asset_id").and_then(serde_json::Value::as_str),
        Some("reference_index")
    );
    assert_eq!(
        index_reference_output.get("asset_path").and_then(serde_json::Value::as_str),
        Some("benchmarks/readiness/local-ready/fastq.index_reference/reference_index/bowtie2/reference")
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
        Some("adna_contamination_panel")
    );
    let sex_reference = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("rxy")
                && row.get("asset_role").and_then(serde_json::Value::as_str)
                    == Some("reference_fasta")
        })
        .expect("bam sex reference row");
    assert_eq!(
        sex_reference.get("asset_id").and_then(serde_json::Value::as_str),
        Some("adna_bam_reference")
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
        Some("adna-y-hg38-mini")
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
    for tool_id in ["angsd", "king"] {
        let kinship_panel = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("asset_role").and_then(serde_json::Value::as_str)
                        == Some("reference_panel")
            })
            .unwrap_or_else(|| panic!("bam kinship panel row for {tool_id}"));
        assert_eq!(
            kinship_panel.get("asset_id").and_then(serde_json::Value::as_str),
            Some("human_like_relatedness_panel")
        );
        assert_eq!(
            kinship_panel.get("asset_path").and_then(serde_json::Value::as_str),
            Some("benchmarks/tests/fixtures/corpora/corpus-01-bam-mini/reference/human_like_relatedness_panel.tsv")
        );
        let kinship_reference = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.kinship")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
                    && row.get("asset_role").and_then(serde_json::Value::as_str)
                        == Some("reference_fasta")
            })
            .unwrap_or_else(|| panic!("bam kinship reference row for {tool_id}"));
        assert_eq!(
            kinship_reference.get("asset_id").and_then(serde_json::Value::as_str),
            Some("corpus_01_bam_reference")
        );
    }
}
