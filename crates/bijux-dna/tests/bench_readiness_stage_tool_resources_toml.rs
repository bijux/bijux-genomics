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
fn bench_readiness_stage_tool_resources_writes_governed_toml_file() {
    let output = run_cli(&["bench", "readiness", "render-stage-tool-resources"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("configs/bench/local/stage-tool-resources.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.local_stage_tool_resources.v1")
    );
    assert_eq!(
        parsed.get("classification_scope").and_then(toml::Value::as_str),
        Some("benchmark_ready_command_resources")
    );
    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 92);
    assert!(rows.iter().all(|row| {
        row.get("threads").and_then(toml::Value::as_integer).unwrap_or_default() > 0
            && row.get("memory_gb").and_then(toml::Value::as_integer).unwrap_or_default() > 0
            && row.get("walltime_minutes").and_then(toml::Value::as_integer).unwrap_or_default() > 0
            && row.get("scratch_gb").and_then(toml::Value::as_integer).unwrap_or_default() > 0
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.profile_read_lengths")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("seqfu")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.detect_adapters")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastqc")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.filter_reads")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.trim_polyg_tails")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.trim_terminal_damage")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("cutadapt")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.extract_umis")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("umi_tools")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(2)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(4)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.normalize_abundance")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("seqkit")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(2)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(4)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.complexity")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("preseq")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
    }));
    for tool_id in ["bedtools", "mosdepth", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.coverage")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(1)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(1)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(6)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(1)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.gc_bias")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
    }));
    for tool_id in ["multiqc", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    for tool_id in ["picard", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.markdup")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(9)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(3)
        }));
    }
    for tool_id in ["picard", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.length_filter")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    for tool_id in ["picard", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.mapping_summary")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.insert_size")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
    }));
    for tool_id in ["picard", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.duplication_metrics")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    for tool_id in ["bamtools", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.mapq_filter")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    for tool_id in ["bamtools", "bedtools", "samtools"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.filter")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(3)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(2)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(7)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(2)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.detect_duplicates_premerge")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bijux_dna")
            && row.get("threads").and_then(toml::Value::as_integer) == Some(1)
            && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(1)
            && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
            && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(1)
    }));
    for tool_id in ["bbduk", "prinseq"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.filter_low_complexity")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
        }));
    }
    for tool_id in ["clumpify", "fastuniq"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.remove_duplicates")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("threads").and_then(toml::Value::as_integer) == Some(4)
                && row.get("memory_gb").and_then(toml::Value::as_integer) == Some(8)
                && row.get("walltime_minutes").and_then(toml::Value::as_integer) == Some(15)
                && row.get("scratch_gb").and_then(toml::Value::as_integer) == Some(4)
        }));
    }
}
