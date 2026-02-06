use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn collect_rs_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
        .filter(|entry| entry.path().extension().and_then(|ext| ext.to_str()) == Some("rs"))
        .map(|entry| entry.path().to_path_buf())
        .collect()
}

#[test]
fn engine_has_no_domain_strings() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        "fastq",
        "bam",
        "stage_specs",
        "stage_registry",
        "stage_plan",
        "stage_plugin",
        "tool_registry",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-engine/src")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-engine must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn runner_has_no_domain_strings() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        "fastq",
        "bam",
        "stage_specs",
        "stage_registry",
        "stage_plan",
        "stage_plugin",
        "tool_registry",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-runner/src")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-runner must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn pipelines_do_not_embed_tool_names() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let tool_ids = [
        "adapterremoval",
        "angsd",
        "authenticct",
        "authenticity",
        "bbduk",
        "bbmerge",
        "bayeshammer",
        "bwa",
        "cutadapt",
        "centrifuge",
        "fastp",
        "fastqc",
        "fastq_screen",
        "fastqvalidator",
        "fastqvalidator_official",
        "flash2",
        "fqtools",
        "gatk",
        "kaiju",
        "king",
        "kraken2",
        "lighter",
        "metaphlan",
        "mosdepth",
        "musket",
        "multiqc",
        "pear",
        "preseq",
        "prinseq",
        "pydamage",
        "rcorrector",
        "rxy",
        "samtools",
        "seqkit",
        "seqkit_stats",
        "seqpurge",
        "seqtk",
        "spades",
        "trim_galore",
        "trimmomatic",
        "umi_tools",
        "vsearch",
        "yleaf",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-pipelines/src")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if tool_ids.iter().any(|tool| content.contains(tool)) {
            offenders.push(file.display().to_string());
        }
    }
    assert!(
        offenders.is_empty(),
        "bijux-pipelines must not embed tool ids in source:\n{}",
        offenders.join("\n")
    );
}
