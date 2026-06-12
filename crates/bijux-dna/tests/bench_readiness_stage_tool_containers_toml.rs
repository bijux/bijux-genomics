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
fn bench_readiness_stage_tool_containers_writes_governed_toml_file() {
    let output = run_cli(&["bench", "readiness", "render-stage-tool-containers"]);
    assert!(
        output.status.success(),
        "command failed: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let repo_root = support::repo_root().expect("repo root");
    let config_path = repo_root.join("benchmarks/configs/local/stage-tool-containers.toml");
    let raw = std::fs::read_to_string(&config_path).expect("read config");
    let parsed: toml::Value = toml::from_str(&raw).expect("parse config");

    assert_eq!(
        parsed.get("schema_version").and_then(toml::Value::as_str),
        Some("bijux.bench.local_stage_tool_containers.v1")
    );
    assert_eq!(
        parsed.get("classification_scope").and_then(toml::Value::as_str),
        Some("benchmark_ready_runtime_declarations")
    );
    let rows = parsed.get("rows").and_then(toml::Value::as_array).expect("rows array");
    assert_eq!(rows.len(), 118);
    assert!(rows.iter().all(|row| {
        row.get("container_id").is_some()
            || row.get("command_entrypoint").is_some()
            || row.get("host_binary_mode").is_some()
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.align")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bwa")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bwa")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/bwa")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.align")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bowtie2")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/bowtie2")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.authenticity")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("authenticct")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("authenticct")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/authenticct:1.0.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.sex")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("rxy")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("rxy")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/rxy")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.haplogroups")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("yleaf")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("yleaf")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/yleaf")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.damage")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("ngsbriggs")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("ngsbriggs")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/ngsbriggs:0.1.3")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.genotyping")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("angsd")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("angsd")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/angsd")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.kinship")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("king")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("king")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/king:2.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.normalize_primers")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("cutadapt")
            && row
                .get("container_id")
                .and_then(toml::Value::as_str)
                .is_some_and(|value| value.starts_with("bijuxdna/cutadapt@sha256:"))
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("cutadapt")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.index_reference")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bowtie2_build")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bowtie2-build")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/bowtie2_build")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.detect_adapters")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastqc")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("fastqc")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/fastqc@sha256:e0b83c56262486cab51020e2bb809b391ad9b38ba7a898588ab15b73586ee789"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.filter_reads")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.trim_polyg_tails")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("fastp")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/fastp@sha256:603656aa361eee1cbd1370db9412e588da91708da5542173e5ae74aab71cbc10"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.trim_terminal_damage")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("cutadapt")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("python")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("cutadapt")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/cutadapt@sha256:4405f2effc1a195c93098408aa36268357c25b758348bfe6da8790bbe7e842ba"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.extract_umis")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("umi_tools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("python")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("umi_tools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/umi_tools@sha256:b2913af8c02c1eeea5de7a4b5c120f65e2003b90479c8873f0ec37689d36296c"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.normalize_abundance")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("seqkit")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("seqkit")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/seqkit@sha256:ca3dc13e3fef5d34927c44b2d8cd2bc6708c2c256f42e51369d7b1203b0d2991"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.complexity")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("preseq")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("preseq")
            && row.get("container_id").and_then(toml::Value::as_str) == Some("bijuxdna/preseq")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.bias_mitigation")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("mapdamage2")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("mapdamage2")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/mapdamage2:2.2.2")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.recalibration")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("gatk")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("gatk")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/gatk:4.6.2.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.endogenous_content")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    for (tool_id, command_entrypoint, container_id) in [
        ("contammix", "contammix", "bijuxdna/contammix"),
        ("schmutzi", "schmutzi", "bijuxdna/schmutzi"),
        ("verifybamid2", "verifybamid2", "bijuxdna/verifybamid2"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.contamination")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
                && row.get("command_entrypoint").and_then(toml::Value::as_str)
                    == Some(command_entrypoint)
                && row.get("container_id").and_then(toml::Value::as_str) == Some(container_id)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.overlap_correction")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bamutil")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bam")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/bamutil:1.0.15")
    }));
    for (tool_id, command_entrypoint, container_id) in [
        ("bedtools", "bedtools", "bijuxdna/bedtools:2.31.1"),
        ("mosdepth", "mosdepth", "bijuxdna/mosdepth:0.3.10"),
        ("samtools", "samtools", "bijuxdna/samtools:1.21"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.coverage")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
                && row.get("command_entrypoint").and_then(toml::Value::as_str)
                    == Some(command_entrypoint)
                && row.get("container_id").and_then(toml::Value::as_str) == Some(container_id)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.gc_bias")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.qc_pre")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("multiqc")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("python")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("multiqc")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/multiqc@sha256:40af0025fcc5bc4ea15e5cd2a4fd7bcfc98ea06c9ca781e6268f3c81d12787ec"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.qc_pre")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.markdup")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.insert_size")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.markdup")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.length_filter")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.length_filter")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    for (tool_id, command_entrypoint, container_id) in [
        ("bamtools", "bamtools", "bijuxdna/bamtools:2.5.2"),
        ("samtools", "samtools", "bijuxdna/samtools:1.21"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.mapq_filter")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
                && row.get("command_entrypoint").and_then(toml::Value::as_str)
                    == Some(command_entrypoint)
                && row.get("container_id").and_then(toml::Value::as_str) == Some(container_id)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.mapping_summary")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.mapping_summary")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.duplication_metrics")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("picard")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("java")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("picard")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/picard:3.3.0")
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.duplication_metrics")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("samtools")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some("bijuxdna/samtools:1.21")
    }));
    for (tool_id, command_entrypoint, container_id) in [
        ("bamtools", "bamtools", "bijuxdna/bamtools:2.5.2"),
        ("bedtools", "bedtools", "bijuxdna/bedtools:2.31.1"),
        ("samtools", "samtools", "bijuxdna/samtools:1.21"),
    ] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("bam.filter")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
                && row.get("command_entrypoint").and_then(toml::Value::as_str)
                    == Some(command_entrypoint)
                && row.get("container_id").and_then(toml::Value::as_str) == Some(container_id)
        }));
    }
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.detect_duplicates_premerge")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bijux_dna")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("internal")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bijux-dna")
            && row.get("host_binary_mode").and_then(toml::Value::as_str) == Some("workspace_binary")
            && row.get("container_id").is_none()
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.filter_low_complexity")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("bbduk")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bbduk")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/bbduk@sha256:da5764715915a5edeb0e40e2c18a5ce7142f31dac8e4844bd2dcb463403b8bd4"
                )
    }));
    assert!(rows.iter().any(|row| {
        row.get("stage_id").and_then(toml::Value::as_str)
            == Some("fastq.filter_low_complexity")
            && row.get("tool_id").and_then(toml::Value::as_str) == Some("prinseq")
            && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
            && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("prinseq++")
            && row.get("container_id").and_then(toml::Value::as_str)
                == Some(
                    "bijuxdna/prinseq@sha256:7216ffecd7913edaea33ec76b3775ab0cb0d60064f31e96c63e043d578a3f971"
                )
    }));
    for tool_id in ["clumpify", "fastuniq"] {
        assert!(rows.iter().any(|row| {
            row.get("stage_id").and_then(toml::Value::as_str) == Some("fastq.remove_duplicates")
                && row.get("tool_id").and_then(toml::Value::as_str) == Some(tool_id)
                && row.get("execution_mode").and_then(toml::Value::as_str) == Some("containerized")
                && row.get("command_entrypoint").and_then(toml::Value::as_str) == Some("bash")
                && row
                    .get("container_id")
                    .and_then(toml::Value::as_str)
                    .is_some_and(|value| value.starts_with(&format!("bijuxdna/{tool_id}@sha256:")))
        }));
    }
}
