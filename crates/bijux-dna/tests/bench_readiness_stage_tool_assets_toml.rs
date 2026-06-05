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

#[test]
fn bench_readiness_stage_tool_assets_writes_governed_toml_file() {
    let output = run_cli(&["bench", "readiness", "render-stage-tool-assets"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("configs/bench/local/stage-tool-assets.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.local_stage_tool_assets.v1")
    );
    assert_eq!(
        parsed.get("classification_scope").and_then(toml::Value::as_str),
        Some("governed_benchmark_command_assets")
    );

    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 36);
    assert!(rows.iter().all(|row| {
        row.get("asset_id")
            .and_then(toml::Value::as_str)
            .is_some_and(|value| !value.trim().is_empty())
            && row
                .get("asset_path")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| !value.trim().is_empty())
    }));

    for tool_id in ["centrifuge", "kaiju", "kraken2", "krakenuniq"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("asset_role").and_then(toml::Value::as_str)
                    == Some("taxonomy_database_root")
                && row.get("asset_id").and_then(toml::Value::as_str) == Some("taxonomy_reference")
        }));
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.screen_taxonomy")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("asset_role").and_then(toml::Value::as_str)
                    == Some("database_artifact_id")
                && row.get("asset_id").and_then(toml::Value::as_str) == Some("taxonomy_db")
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.deplete_host")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("reference_catalog_id")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("host_reference")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.deplete_host")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("asset_role").and_then(toml::Value::as_str)
                == Some("reference_index_artifact_id")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("reference_index")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.deplete_reference_contaminants")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("asset_role").and_then(toml::Value::as_str)
                == Some("reference_catalog_id")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("contaminant_reference")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.deplete_reference_contaminants")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("asset_role").and_then(toml::Value::as_str)
                == Some("reference_index_artifact_id")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("reference_index")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.deplete_rrna")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("sortmerna")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("rrna_reference")
            && row.get("asset_id").and_then(toml::Value::as_str)
                == Some("sortmerna_common_rrna_reference")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.deplete_rrna")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("sortmerna")
            && row.get("asset_role").and_then(toml::Value::as_str)
                == Some("database_artifact_id")
            && row.get("asset_id").and_then(toml::Value::as_str)
                == Some("sortmerna_common_rrna_reference")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.index_reference")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2_build")
            && row.get("asset_role").and_then(toml::Value::as_str)
                == Some("reference_index_output")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("reference_index")
    }));

    for tool_id in ["contammix", "schmutzi", "verifybamid2"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.contamination")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("asset_role").and_then(toml::Value::as_str) == Some("reference_panel")
                && row.get("asset_id").and_then(toml::Value::as_str)
                    == Some("adna_contamination_panel")
        }));
    }

    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.sex")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("rxy")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("reference_fasta")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("adna_bam_reference")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.haplogroups")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("yleaf")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("reference_panel")
            && row.get("asset_id").and_then(toml::Value::as_str) == Some("adna-y-hg38-mini")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.genotyping")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("angsd")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("regions")
            && row.get("asset_id").and_then(toml::Value::as_str)
                == Some("human_like_genotyping_target_regions")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.recalibration")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("gatk")
            && row.get("asset_role").and_then(toml::Value::as_str) == Some("known_sites")
            && row.get("asset_id").and_then(toml::Value::as_str)
                == Some("human_like_recalibration_known_sites")
    }));
    for tool_id in ["angsd", "king"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("asset_role").and_then(toml::Value::as_str)
                    == Some("reference_fasta")
                && row.get("asset_id").and_then(toml::Value::as_str)
                    == Some("corpus_01_bam_reference")
        }));
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.kinship")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("asset_role").and_then(toml::Value::as_str)
                    == Some("reference_panel")
                && row.get("asset_id").and_then(toml::Value::as_str)
                    == Some("human_like_relatedness_panel")
        }));
    }
}
