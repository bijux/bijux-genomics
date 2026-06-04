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
fn bench_readiness_stage_tool_resources_reports_governed_benchmark_ready_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-tool-resources", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_resources.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-tool-resources.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_ready_command_resources")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(73));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(73)
    );
    assert_eq!(
        payload.get("nonzero_resource_row_count").and_then(serde_json::Value::as_u64),
        Some(73)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("fastq"))
            .and_then(serde_json::Value::as_u64),
        Some(63)
    );
    assert_eq!(
        payload
            .get("domain_counts")
            .and_then(|value| value.get("bam"))
            .and_then(serde_json::Value::as_u64),
        Some(10)
    );
    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let fastqc = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.detect_adapters")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
        })
        .expect("detect-adapters fastqc row");
    assert_eq!(fastqc.get("threads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(fastqc.get("memory_gb").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(fastqc.get("walltime_minutes").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(fastqc.get("scratch_gb").and_then(serde_json::Value::as_u64), Some(4));
    let fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("filter-reads fastp row");
    assert_eq!(fastp.get("threads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(fastp.get("memory_gb").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(fastp.get("walltime_minutes").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(fastp.get("scratch_gb").and_then(serde_json::Value::as_u64), Some(4));
    let trim_polyg_fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("trim-polyg fastp row");
    assert_eq!(trim_polyg_fastp.get("threads").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(trim_polyg_fastp.get("memory_gb").and_then(serde_json::Value::as_u64), Some(8));
    assert_eq!(
        trim_polyg_fastp.get("walltime_minutes").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(trim_polyg_fastp.get("scratch_gb").and_then(serde_json::Value::as_u64), Some(4));
    let trim_terminal_damage_cutadapt = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
        })
        .expect("trim-terminal-damage cutadapt row");
    assert_eq!(
        trim_terminal_damage_cutadapt.get("threads").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        trim_terminal_damage_cutadapt.get("memory_gb").and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        trim_terminal_damage_cutadapt.get("walltime_minutes").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        trim_terminal_damage_cutadapt.get("scratch_gb").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    let extract_umis = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.extract_umis")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("umi_tools")
        })
        .expect("extract-umis umi_tools row");
    assert_eq!(extract_umis.get("threads").and_then(serde_json::Value::as_u64), Some(2));
    assert_eq!(extract_umis.get("memory_gb").and_then(serde_json::Value::as_u64), Some(4));
    assert_eq!(extract_umis.get("walltime_minutes").and_then(serde_json::Value::as_u64), Some(15));
    assert_eq!(extract_umis.get("scratch_gb").and_then(serde_json::Value::as_u64), Some(4));
    let detect_duplicates_bijux = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
        })
        .expect("detect-duplicates bijux_dna row");
    assert_eq!(detect_duplicates_bijux.get("threads").and_then(serde_json::Value::as_u64), Some(1));
    assert_eq!(
        detect_duplicates_bijux.get("memory_gb").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        detect_duplicates_bijux.get("walltime_minutes").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        detect_duplicates_bijux.get("scratch_gb").and_then(serde_json::Value::as_u64),
        Some(1)
    );
    let normalize_abundance_seqkit = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_abundance")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
        })
        .expect("normalize-abundance seqkit row");
    assert_eq!(
        normalize_abundance_seqkit.get("threads").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    assert_eq!(
        normalize_abundance_seqkit.get("memory_gb").and_then(serde_json::Value::as_u64),
        Some(4)
    );
    assert_eq!(
        normalize_abundance_seqkit.get("walltime_minutes").and_then(serde_json::Value::as_u64),
        Some(15)
    );
    assert_eq!(
        normalize_abundance_seqkit.get("scratch_gb").and_then(serde_json::Value::as_u64),
        Some(2)
    );
    for tool_id in ["multiqc", "samtools"] {
        let qc_pre = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("bam qc-pre {tool_id} row"));
        assert_eq!(qc_pre.get("threads").and_then(serde_json::Value::as_u64), Some(3));
        assert_eq!(qc_pre.get("memory_gb").and_then(serde_json::Value::as_u64), Some(2));
        assert_eq!(qc_pre.get("walltime_minutes").and_then(serde_json::Value::as_u64), Some(7));
        assert_eq!(qc_pre.get("scratch_gb").and_then(serde_json::Value::as_u64), Some(2));
    }
    for tool_id in ["bbduk", "prinseq"] {
        let filter_low_complexity = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.filter_low_complexity")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("filter-low-complexity {tool_id} row"));
        assert_eq!(
            filter_low_complexity.get("threads").and_then(serde_json::Value::as_u64),
            Some(4)
        );
        assert_eq!(
            filter_low_complexity.get("memory_gb").and_then(serde_json::Value::as_u64),
            Some(8)
        );
        assert_eq!(
            filter_low_complexity.get("walltime_minutes").and_then(serde_json::Value::as_u64),
            Some(15)
        );
        assert_eq!(
            filter_low_complexity.get("scratch_gb").and_then(serde_json::Value::as_u64),
            Some(4)
        );
    }
    for tool_id in ["clumpify", "fastuniq"] {
        let remove_duplicates = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.remove_duplicates")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("remove-duplicates {tool_id} row"));
        assert_eq!(remove_duplicates.get("threads").and_then(serde_json::Value::as_u64), Some(4));
        assert_eq!(remove_duplicates.get("memory_gb").and_then(serde_json::Value::as_u64), Some(8));
        assert_eq!(
            remove_duplicates.get("walltime_minutes").and_then(serde_json::Value::as_u64),
            Some(15)
        );
        assert_eq!(
            remove_duplicates.get("scratch_gb").and_then(serde_json::Value::as_u64),
            Some(4)
        );
    }
}
