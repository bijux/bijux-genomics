#![allow(non_snake_case)]
#![allow(non_snake_case)]
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
fn policy__boundaries__purity_scans__engine_has_no_domain_strings() {
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
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-engine must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__runner_has_no_domain_strings() {
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
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-runner must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__core_execution_contract_has_no_stage_contract_imports() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    for file in collect_rs_files(&root.join("crates/bijux-core/src/contract/execution")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if content.contains("bijux_stage_contract") {
            offenders.push(file.display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-core execution contracts must not import bijux-stage-contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__engine_has_no_stage_contract_imports() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = [
        "bijux_stage_contract",
        "bijux_stages_fastq",
        "bijux_stages_bam",
        "bijux_runner",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-engine/src")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-engine must not import stage-contract or stage crates:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__execution_graph_has_no_stage_contract_symbols() {
    let root = workspace_root();
    let path = root.join("crates/bijux-core/src/contract/execution/graph.rs");
    let content = std::fs::read_to_string(&path).expect("read execution_graph.rs");
    let needles = ["StagePlanV1", "StagePlugin"];
    bijux_policies::policy_assert!(
        !needles.iter().any(|needle| content.contains(needle)),
        "execution_graph.rs must not reference stage-contract symbols"
    );
}

#[test]
fn policy__boundaries__purity_scans__stage_specs_have_no_command_building() {
    let root = workspace_root();
    let mut offenders = Vec::new();
    let needles = ["CommandSpecV1", "ContainerImageRefV1", "argv"];
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if !path.to_string_lossy().contains("stage_specs") {
            continue;
        }
        if path
            .to_string_lossy()
            .contains("/crates/bijux-policies/tests/")
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read stage_specs source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "stage_specs must not build commands/images:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__planners_only_build_execution_steps() {
    let root = workspace_root();
    let allowlist = [
        "bijux-core",
        "bijux-planner-fastq",
        "bijux-planner-bam",
        "bijux-stage-contract",
    ];
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path);
        let rel_str = rel.to_string_lossy();
        if !rel_str.contains("/src/") {
            continue;
        }
        if rel_str.ends_with("_tests.rs") {
            continue;
        }
        if allowlist
            .iter()
            .any(|crate_name| rel_str.contains(crate_name))
        {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        if content.contains("#[cfg(test)]") {
            continue;
        }
        if content.contains("ExecutionStep {")
            && (content.contains("command:") || content.contains("image:"))
        {
            offenders.push(rel_str.to_string());
        }
    }
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "ExecutionStep construction must live in planners/stage-contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__pipelines_do_not_embed_tool_names() {
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
    bijux_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-pipelines must not embed tool ids in source:\n{}",
        offenders.join("\n")
    );
}
