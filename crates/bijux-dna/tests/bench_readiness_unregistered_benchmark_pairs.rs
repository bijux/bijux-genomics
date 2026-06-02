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
fn bench_readiness_unregistered_benchmark_pairs_reports_registry_drift() {
    let payload =
        run_cli_json(&["bench", "readiness", "render-unregistered-benchmark-pairs", "--json"]);
    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.unregistered_benchmark_pairs.v1")
    );
    assert_eq!(
        payload.get("output_path").and_then(serde_json::Value::as_str),
        Some("target/bench-readiness/unregistered-benchmark-pairs.tsv")
    );
    assert_eq!(
        payload.get("unregistered_pair_count").and_then(serde_json::Value::as_u64),
        Some(14)
    );
    assert_eq!(payload.get("ok").and_then(serde_json::Value::as_bool), Some(false));

    let domain_counts = payload
        .get("domain_counts")
        .and_then(serde_json::Value::as_object)
        .expect("domain_counts object");
    assert_eq!(domain_counts.get("fastq").and_then(serde_json::Value::as_u64), Some(7));
    assert_eq!(domain_counts.get("bam").and_then(serde_json::Value::as_u64), Some(7));

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 14, "governed registry-drift slice must retain the current fourteen rows");
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.align")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bowtie2")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_registered_pair_missing")
        }),
        "bam.align / bowtie2 must remain visible as a pair-missing registry row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bcftools")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "bam.genotyping / bcftools must remain visible as a missing-tool registry row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.genotyping")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("angsd")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_registered_pair_missing")
        }),
        "bam.genotyping / angsd must remain visible as a pair-missing registry row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("preseq")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "bam.complexity / preseq must remain visible as a missing tool row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ngsbriggs")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "bam.damage / ngsbriggs must remain visible as a planned missing-tool registry row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.haplogroups")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("yleaf")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "bam.haplogroups / yleaf must remain visible as a missing-tool registry row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.recalibration")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("gatk")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "bam.recalibration / gatk must remain visible as a missing tool row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.detect_duplicates_premerge")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "fastq.detect_duplicates_premerge / bijux_dna must remain visible as a missing tool row"
    );
    assert!(
        rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.estimate_library_complexity_prealign")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
                && row.get("registry_status").and_then(serde_json::Value::as_str)
                    == Some("tool_missing")
        }),
        "fastq.estimate_library_complexity_prealign / bijux_dna must remain visible as a missing tool row"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("bam.overlap_correction")
        }),
        "bam.overlap_correction must not drift against the registry once the admitted bamutil row is published in production"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("addeam")
        }),
        "bam.damage / addeam must not drift against the registry once it is registered in production"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str)
                    == Some("damageprofiler")
        }),
        "bam.damage / damageprofiler must not drift against the registry once it is registered in production"
    );
    for tool_id in ["fastp", "prinseq", "seqfu"] {
        assert!(
            !rows.iter().any(|row| {
                row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.profile_read_lengths")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            }),
            "fastq.profile_read_lengths / {tool_id} must no longer drift against the registry"
        );
    }
    for tool_id in [
        "adapterremoval",
        "alientrimmer",
        "atropos",
        "bbduk",
        "cutadapt",
        "fastp",
        "fastx_clipper",
        "leehom",
        "prinseq",
        "seqkit",
        "skewer",
        "trim_galore",
        "trimmomatic",
    ] {
        assert!(
            !rows.iter().any(|row| {
                row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.trim_reads")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            }),
            "fastq.trim_reads / {tool_id} must not drift against the registry"
        );
    }
    for tool_id in ["bbduk", "fastp", "prinseq", "seqkit"] {
        assert!(
            !rows.iter().any(|row| {
                row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.filter_reads")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            }),
            "fastq.filter_reads / {tool_id} must not drift against the registry"
        );
    }
    for tool_id in ["adapterremoval", "bbmerge", "flash2", "leehom", "pear", "vsearch"] {
        assert!(
            !rows.iter().any(|row| {
                row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("fastq.merge_pairs")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            }),
            "fastq.merge_pairs / {tool_id} must not drift against the registry"
        );
    }
    for tool_id in ["bamtools", "bedtools", "samtools"] {
        assert!(
            !rows.iter().any(|row| {
                row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                    && row.get("stage_id").and_then(serde_json::Value::as_str)
                        == Some("bam.validate")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            }),
            "bam.validate / {tool_id} must not drift against the registry"
        );
    }
    assert!(
        !rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("bam")
                && row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("multiqc")
        }),
        "bam.qc_pre / multiqc must not drift against the registry"
    );
    assert!(
        !rows.iter().any(|row| {
            row.get("domain").and_then(serde_json::Value::as_str) == Some("fastq")
                && row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.extract_umis")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("umi_tools")
        }),
        "fastq.extract_umis / umi_tools must not drift against the registry"
    );
}
