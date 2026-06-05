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
fn bench_readiness_stage_tool_containers_reports_governed_runtime_rows() {
    let payload = run_cli_json(&["bench", "readiness", "render-stage-tool-containers", "--json"]);

    assert_eq!(
        payload.get("schema_version").and_then(serde_json::Value::as_str),
        Some("bijux.bench.readiness.stage_tool_containers.v1")
    );
    assert_eq!(
        payload.get("config_path").and_then(serde_json::Value::as_str),
        Some("configs/bench/local/stage-tool-containers.toml")
    );
    assert_eq!(
        payload.get("classification_scope").and_then(serde_json::Value::as_str),
        Some("benchmark_ready_runtime_declarations")
    );
    assert_eq!(payload.get("row_count").and_then(serde_json::Value::as_u64), Some(106));
    assert_eq!(
        payload.get("benchmark_ready_row_count").and_then(serde_json::Value::as_u64),
        Some(106)
    );
    assert_eq!(payload.get("external_row_count").and_then(serde_json::Value::as_u64), Some(105));
    assert_eq!(
        payload.get("container_declared_row_count").and_then(serde_json::Value::as_u64),
        Some(105)
    );
    assert_eq!(
        payload.get("command_entrypoint_row_count").and_then(serde_json::Value::as_u64),
        Some(106)
    );
    assert_eq!(payload.get("host_binary_row_count").and_then(serde_json::Value::as_u64), Some(1));
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
        Some(43)
    );

    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("containerized"))
            .and_then(serde_json::Value::as_u64),
        Some(85)
    );
    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("internal"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("java"))
            .and_then(serde_json::Value::as_u64),
        Some(10)
    );
    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("mixed"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );
    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("python"))
            .and_then(serde_json::Value::as_u64),
        Some(8)
    );
    assert_eq!(
        payload
            .get("execution_mode_counts")
            .and_then(|value| value.get("r"))
            .and_then(serde_json::Value::as_u64),
        Some(1)
    );

    let rows = payload.get("rows").and_then(serde_json::Value::as_array).expect("rows array");
    let ngsbriggs = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("ngsbriggs")
        })
        .expect("bam damage ngsbriggs row");
    assert_eq!(
        ngsbriggs.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        ngsbriggs.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("ngsbriggs")
    );
    assert_eq!(
        ngsbriggs.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/ngsbriggs:0.1.3")
    );
    let gatk_recalibration = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.recalibration")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("gatk")
        })
        .expect("bam recalibration gatk row");
    assert_eq!(
        gatk_recalibration.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("java")
    );
    assert_eq!(
        gatk_recalibration.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("gatk")
    );
    assert_eq!(
        gatk_recalibration.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/gatk:4.6.2.0")
    );
    assert!(rows.iter().all(|row| {
        row.get("container_id").is_some()
            || row.get("command_entrypoint").is_some()
            || row.get("host_binary_mode").is_some()
    }));
    let cutadapt = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_primers")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
        })
        .expect("cutadapt normalize_primers row");
    assert_eq!(cutadapt.get("execution_mode").and_then(serde_json::Value::as_str), Some("python"));
    assert_eq!(
        cutadapt.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("cutadapt")
    );
    assert!(
        cutadapt
            .get("container_id")
            .and_then(serde_json::Value::as_str)
            .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:")),
        "cutadapt row must preserve the governed container declaration"
    );
    let bam_authenticity = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.authenticity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("authenticct")
        })
        .expect("bam authenticity authenticct row");
    assert_eq!(
        bam_authenticity.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        bam_authenticity.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("authenticct")
    );
    assert_eq!(
        bam_authenticity.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/authenticct:1.0.0")
    );
    let bamutil_overlap = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.overlap_correction")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bamutil")
        })
        .expect("overlap-correction bamutil row");
    assert_eq!(
        bamutil_overlap.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        bamutil_overlap.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("bam")
    );
    assert_eq!(
        bamutil_overlap.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/bamutil:1.0.15")
    );
    let mapdamage2_bias = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.bias_mitigation")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("mapdamage2")
        })
        .expect("bam bias-mitigation mapdamage2 row");
    assert_eq!(
        mapdamage2_bias.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        mapdamage2_bias.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("mapdamage2")
    );
    assert_eq!(
        mapdamage2_bias.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/mapdamage2:2.2.2")
    );
    let bam_sex_rxy = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.sex")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("rxy")
        })
        .expect("bam sex rxy row");
    assert_eq!(
        bam_sex_rxy.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        bam_sex_rxy.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("rxy")
    );
    assert_eq!(
        bam_sex_rxy.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/rxy")
    );
    let fastqc = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.detect_adapters")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastqc")
        })
        .expect("detect-adapters fastqc row");
    assert_eq!(fastqc.get("execution_mode").and_then(serde_json::Value::as_str), Some("java"));
    assert_eq!(
        fastqc.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("fastqc")
    );
    assert_eq!(
        fastqc.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789")
    );
    let fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.filter_reads")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("filter-reads fastp row");
    assert_eq!(
        fastp.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(fastp.get("command_entrypoint").and_then(serde_json::Value::as_str), Some("fastp"));
    assert_eq!(
        fastp.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10")
    );
    let trim_polyg_fastp = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_polyg_tails")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("fastp")
        })
        .expect("trim-polyg fastp row");
    assert_eq!(
        trim_polyg_fastp.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        trim_polyg_fastp.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("fastp")
    );
    assert_eq!(
        trim_polyg_fastp
            .get("container_id")
            .and_then(serde_json::Value::as_str),
        Some("bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10")
    );
    let trim_terminal_damage_cutadapt = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.trim_terminal_damage")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("cutadapt")
        })
        .expect("trim-terminal-damage cutadapt row");
    assert_eq!(
        trim_terminal_damage_cutadapt.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("python")
    );
    assert_eq!(
        trim_terminal_damage_cutadapt.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("cutadapt")
    );
    assert_eq!(
        trim_terminal_damage_cutadapt
            .get("container_id")
            .and_then(serde_json::Value::as_str),
        Some("bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba")
    );
    let extract_umis = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("fastq.extract_umis")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("umi_tools")
        })
        .expect("extract-umis umi_tools row");
    assert_eq!(
        extract_umis.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("python")
    );
    for (tool_id, container_id) in [
        ("bedtools", "bijuxdna/bedtools:2.31.1"),
        ("mosdepth", "bijuxdna/mosdepth:0.3.10"),
        ("samtools", "bijuxdna/samtools:1.21"),
    ] {
        let bam_coverage = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.coverage")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("bam coverage {tool_id} row"));
        assert_eq!(
            bam_coverage.get("execution_mode").and_then(serde_json::Value::as_str),
            Some("containerized")
        );
        assert_eq!(
            bam_coverage.get("command_entrypoint").and_then(serde_json::Value::as_str),
            Some(tool_id)
        );
        assert_eq!(
            bam_coverage.get("container_id").and_then(serde_json::Value::as_str),
            Some(container_id)
        );
    }
    let gc_bias_picard = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.gc_bias")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
        })
        .expect("bam gc-bias picard row");
    assert_eq!(
        gc_bias_picard.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("java")
    );
    assert_eq!(
        gc_bias_picard.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("picard")
    );
    assert_eq!(
        gc_bias_picard.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/picard:3.3.0")
    );
    assert_eq!(
        extract_umis.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("umi_tools")
    );
    assert_eq!(
        extract_umis.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/umi_tools@sha256:b2913af8c02c1eeea5de7a4b5c120f65e2003b90479c8873f0ec37689d36296c")
    );
    let bam_complexity_preseq = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.complexity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("preseq")
        })
        .expect("bam complexity preseq row");
    assert_eq!(
        bam_complexity_preseq.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        bam_complexity_preseq.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("preseq")
    );
    assert_eq!(
        bam_complexity_preseq.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/preseq")
    );
    let bam_endogenous_content = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.endogenous_content")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam endogenous-content samtools row");
    assert_eq!(
        bam_endogenous_content.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        bam_endogenous_content.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        bam_endogenous_content.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/samtools:1.21")
    );
    for (tool_id, command_entrypoint, container_id) in [
        ("contammix", "contammix", "bijuxdna/contammix"),
        ("schmutzi", "schmutzi", "bijuxdna/schmutzi"),
        ("verifybamid2", "verifybamid2", "bijuxdna/verifybamid2"),
    ] {
        let contamination = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.contamination")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("bam contamination {tool_id} row"));
        assert_eq!(
            contamination.get("execution_mode").and_then(serde_json::Value::as_str),
            Some("containerized")
        );
        assert_eq!(
            contamination.get("command_entrypoint").and_then(serde_json::Value::as_str),
            Some(command_entrypoint)
        );
        assert_eq!(
            contamination.get("container_id").and_then(serde_json::Value::as_str),
            Some(container_id)
        );
    }
    let normalize_abundance_seqkit = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.normalize_abundance")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("seqkit")
        })
        .expect("normalize-abundance seqkit row");
    assert_eq!(
        normalize_abundance_seqkit.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        normalize_abundance_seqkit.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("seqkit")
    );
    assert_eq!(
        normalize_abundance_seqkit.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991")
    );
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(serde_json::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(serde_json::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(serde_json::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.insert_size")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(serde_json::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(serde_json::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(serde_json::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.markdup")
            && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(serde_json::Value::as_str)
                == Some("containerized")
            && row.get("command_entrypoint").and_then(serde_json::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(serde_json::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    let qc_pre_multiqc = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("multiqc")
        })
        .expect("bam qc-pre multiqc row");
    assert_eq!(
        qc_pre_multiqc.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("python")
    );
    assert_eq!(
        qc_pre_multiqc.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("multiqc")
    );
    assert_eq!(
        qc_pre_multiqc.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/multiqc@sha256:40af0025fcc5bc4ea15e5cd2a4fd7bcfc98ea06c9ca781e6268f3c81d12787ec")
    );
    let qc_pre_samtools = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.qc_pre")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam qc-pre samtools row");
    assert_eq!(
        qc_pre_samtools.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        qc_pre_samtools.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        qc_pre_samtools.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/samtools:1.21")
    );
    for (tool_id, command_entrypoint, container_id) in [
        ("bamtools", "bamtools", "bijuxdna/bamtools:2.5.2"),
        ("samtools", "samtools", "bijuxdna/samtools:1.21"),
    ] {
        let mapq_filter = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapq_filter")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("bam mapq-filter {tool_id} row"));
        assert_eq!(
            mapq_filter.get("execution_mode").and_then(serde_json::Value::as_str),
            Some("containerized")
        );
        assert_eq!(
            mapq_filter.get("command_entrypoint").and_then(serde_json::Value::as_str),
            Some(command_entrypoint)
        );
        assert_eq!(
            mapq_filter.get("container_id").and_then(serde_json::Value::as_str),
            Some(container_id)
        );
    }
    let length_filter_picard = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.length_filter")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
        })
        .expect("bam length-filter picard row");
    assert_eq!(
        length_filter_picard.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("java")
    );
    assert_eq!(
        length_filter_picard.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("picard")
    );
    assert_eq!(
        length_filter_picard.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/picard:3.3.0")
    );
    let length_filter_samtools = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.length_filter")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam length-filter samtools row");
    assert_eq!(
        length_filter_samtools.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        length_filter_samtools.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        length_filter_samtools.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/samtools:1.21")
    );
    let mapping_summary_picard = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapping_summary")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
        })
        .expect("bam mapping-summary picard row");
    assert_eq!(
        mapping_summary_picard.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("java")
    );
    assert_eq!(
        mapping_summary_picard.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("picard")
    );
    assert_eq!(
        mapping_summary_picard.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/picard:3.3.0")
    );
    let mapping_summary_samtools = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.mapping_summary")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam mapping-summary samtools row");
    assert_eq!(
        mapping_summary_samtools.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        mapping_summary_samtools.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        mapping_summary_samtools.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/samtools:1.21")
    );
    let duplication_metrics_picard = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("picard")
        })
        .expect("bam duplication-metrics picard row");
    assert_eq!(
        duplication_metrics_picard.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("java")
    );
    assert_eq!(
        duplication_metrics_picard.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("picard")
    );
    assert_eq!(
        duplication_metrics_picard.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/picard:3.3.0")
    );
    let duplication_metrics_samtools = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("bam.duplication_metrics")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("samtools")
        })
        .expect("bam duplication-metrics samtools row");
    assert_eq!(
        duplication_metrics_samtools.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        duplication_metrics_samtools.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("samtools")
    );
    assert_eq!(
        duplication_metrics_samtools.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/samtools:1.21")
    );
    for (tool_id, command_entrypoint, container_id) in [
        ("bamtools", "bamtools", "bijuxdna/bamtools:2.5.2"),
        ("bedtools", "bedtools", "bijuxdna/bedtools:2.31.1"),
        ("samtools", "samtools", "bijuxdna/samtools:1.21"),
    ] {
        let bam_filter = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str) == Some("bam.filter")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("bam filter {tool_id} row"));
        assert_eq!(
            bam_filter.get("execution_mode").and_then(serde_json::Value::as_str),
            Some("containerized")
        );
        assert_eq!(
            bam_filter.get("command_entrypoint").and_then(serde_json::Value::as_str),
            Some(command_entrypoint)
        );
        assert_eq!(
            bam_filter.get("container_id").and_then(serde_json::Value::as_str),
            Some(container_id)
        );
    }
    let detect_duplicates_bijux = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.detect_duplicates_premerge")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bijux_dna")
        })
        .expect("detect-duplicates bijux_dna row");
    assert_eq!(
        detect_duplicates_bijux.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("internal")
    );
    assert_eq!(
        detect_duplicates_bijux.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("bijux-dna")
    );
    assert_eq!(
        detect_duplicates_bijux.get("host_binary_mode").and_then(serde_json::Value::as_str),
        Some("workspace_binary")
    );
    assert!(
        detect_duplicates_bijux.get("container_id").is_none(),
        "workspace-binary detect-duplicates row must not declare a container id"
    );
    let filter_low_complexity_bbduk = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("bbduk")
        })
        .expect("filter-low-complexity bbduk row");
    assert_eq!(
        filter_low_complexity_bbduk.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        filter_low_complexity_bbduk.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("bbduk")
    );
    assert_eq!(
        filter_low_complexity_bbduk.get("container_id").and_then(serde_json::Value::as_str),
        Some("bijuxdna/bbduk@sha256:da5764715915a5edeb0e40e2c18a5ce7142f31dac8e4844bd2dcb463403b8bd4")
    );
    let filter_low_complexity_prinseq = rows
        .iter()
        .find(|row| {
            row.get("stage_id").and_then(serde_json::Value::as_str)
                == Some("fastq.filter_low_complexity")
                && row.get("tool_id").and_then(serde_json::Value::as_str) == Some("prinseq")
        })
        .expect("filter-low-complexity prinseq row");
    assert_eq!(
        filter_low_complexity_prinseq.get("execution_mode").and_then(serde_json::Value::as_str),
        Some("containerized")
    );
    assert_eq!(
        filter_low_complexity_prinseq.get("command_entrypoint").and_then(serde_json::Value::as_str),
        Some("prinseq++")
    );
    assert_eq!(
        filter_low_complexity_prinseq
            .get("container_id")
            .and_then(serde_json::Value::as_str),
        Some("bijuxdna/prinseq@sha256:7216ffecd7913edaea33ec76b3775ab0cb0d60064f31e96c63e043d578a3f971")
    );
    for tool_id in ["clumpify", "fastuniq"] {
        let remove_duplicates = rows
            .iter()
            .find(|row| {
                row.get("stage_id").and_then(serde_json::Value::as_str)
                    == Some("fastq.remove_duplicates")
                    && row.get("tool_id").and_then(serde_json::Value::as_str) == Some(tool_id)
            })
            .unwrap_or_else(|| panic!("remove-duplicates {tool_id} row"));
        assert_eq!(
            remove_duplicates.get("execution_mode").and_then(serde_json::Value::as_str),
            Some("containerized")
        );
        assert_eq!(
            remove_duplicates.get("command_entrypoint").and_then(serde_json::Value::as_str),
            Some("bash")
        );
        assert!(
            remove_duplicates
                .get("container_id")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.starts_with(&format!("bijuxdna/{tool_id}@sha256:"))),
            "remove-duplicates {tool_id} row must preserve the governed container declaration"
        );
    }
}
