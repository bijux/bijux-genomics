#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use regex::Regex;
use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
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

fn contains_tool_id_token(content: &str, tool_id: &str) -> bool {
    let pattern = format!(r"(^|[^a-z0-9_]){}([^a-z0-9_]|$)", regex::escape(tool_id));
    Regex::new(&pattern)
        .unwrap_or_else(|err| panic!("compile tool-id token matcher `{pattern}`: {err}"))
        .is_match(content)
}

fn strip_rust_test_modules(raw: &str) -> String {
    let mut lines = Vec::new();
    let mut awaiting_test_module = false;
    let mut skip_depth: usize = 0;

    for raw_line in raw.lines() {
        let trimmed = raw_line.trim();
        if skip_depth > 0 {
            skip_depth = skip_depth
                .saturating_add(raw_line.matches('{').count())
                .saturating_sub(raw_line.matches('}').count());
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            awaiting_test_module = true;
            continue;
        }
        if awaiting_test_module {
            if trimmed.is_empty() || trimmed.starts_with("#[") {
                continue;
            }
            if trimmed.starts_with("mod ") && raw_line.contains('{') {
                skip_depth =
                    raw_line.matches('{').count().saturating_sub(raw_line.matches('}').count());
                awaiting_test_module = false;
                continue;
            }
            awaiting_test_module = false;
        }
        lines.push(raw_line);
    }

    lines.join("\n")
}

#[test]
fn policy__boundaries__purity_scans__engine_has_no_domain_strings() {
    let root = repo_root();
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
    for file in collect_rs_files(&root.join("crates/bijux-dna-engine/src")) {
        let content =
            strip_rust_test_modules(&std::fs::read_to_string(&file).expect("read source"));
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__runner_has_no_domain_strings() {
    let root = repo_root();
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
    for file in collect_rs_files(&root.join("crates/bijux-dna-runner/src")) {
        let content =
            strip_rust_test_modules(&std::fs::read_to_string(&file).expect("read source"));
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-runner must not reference domain/stage strings:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__core_execution_contract_has_no_stage_contract_imports() {
    let root = repo_root();
    let mut offenders = Vec::new();
    for file in collect_rs_files(&root.join("crates/bijux-dna-core/src/contract/execution")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if content.contains("bijux_dna_stage_contract") {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-core execution contracts must not import bijux-dna-stage-contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__engine_has_no_stage_contract_imports() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let needles = [
        "bijux_dna_stage_contract",
        "bijux_dna_stages_fastq",
        "bijux_dna_stages_bam",
        "bijux_dna_runner",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-dna-engine/src")) {
        let content = std::fs::read_to_string(&file).expect("read source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine must not import stage-contract or stage crates:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__execution_graph_has_no_stage_contract_symbols() {
    let root = repo_root();
    let path = root.join("crates/bijux-dna-core/src/contract/execution/graph.rs");
    let content = std::fs::read_to_string(&path).expect("read execution_graph.rs");
    let needles = ["StagePlanV1", "StagePlugin"];
    bijux_dna_policies::policy_assert!(
        !needles.iter().any(|needle| content.contains(needle)),
        "execution_graph.rs must not reference stage-contract symbols"
    );
}

#[test]
fn policy__boundaries__purity_scans__stage_specs_have_no_command_building() {
    let root = repo_root();
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
        if path.to_string_lossy().contains("/crates/bijux-dna-policies/tests/") {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read stage_specs source");
        if needles.iter().any(|needle| content.contains(needle)) {
            offenders.push(path.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage_specs must not build commands/images:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__stage_crates_define_invocations_only_no_execution_effects() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let deny_tokens = [
        "std::process::Command",
        "Command::new(",
        "execute_step(",
        "run_docker_command(",
        "run_apptainer_command(",
        "bijux_dna_runner",
    ];
    for crate_dir in ["crates/bijux-dna-stages-fastq/src", "crates/bijux-dna-stages-bam/src"] {
        for file in collect_rs_files(&root.join(crate_dir)) {
            let content = std::fs::read_to_string(&file).expect("read source");
            if deny_tokens.iter().any(|token| content.contains(token)) {
                offenders.push(file.display().to_string());
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage crates must define invocations only (no execution effects):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__planners_only_build_execution_steps() {
    let root = repo_root();
    let allowlist = [
        "crates/bijux-dna-core",
        "crates/bijux-dna-planner-fastq",
        "crates/bijux-dna-planner-bam",
        "crates/bijux-dna-stage-contract",
    ];
    let execution_step_allowlist = [
        "crates/bijux-dna-api/src/runtime/run/reporting/local_workflows.rs",
        "crates/bijux-dna-api/src/runtime/run/reporting/failure_injection.rs",
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
        if allowlist.iter().any(|crate_name| rel_str.contains(crate_name)) {
            continue;
        }
        if execution_step_allowlist.iter().any(|allowed| rel_str == *allowed) {
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
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "ExecutionStep construction must live in planners/stage-contract:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__pipelines_do_not_embed_tool_names() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let tool_ids = [
        "adapterremoval",
        "angsd",
        "authenticct",
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
        "fastqvalidator",
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
    let allowlisted_files = [
        "crates/bijux-dna-pipelines/src/bam/profile_invariants.rs",
        "crates/bijux-dna-pipelines/src/fastq/invariants.rs",
        "crates/bijux-dna-pipelines/src/lib.rs",
        "crates/bijux-dna-pipelines/src/vcf/mod.rs",
        "crates/bijux-dna-pipelines/src/bam/workflow_registry.rs",
    ];
    for file in collect_rs_files(&root.join("crates/bijux-dna-pipelines/src")) {
        let rel =
            file.strip_prefix(&root).unwrap_or(file.as_path()).to_string_lossy().replace('\\', "/");
        if allowlisted_files.iter().any(|allowed| rel.ends_with(allowed)) {
            continue;
        }
        let content = std::fs::read_to_string(&file).expect("read source");
        if tool_ids.iter().any(|tool| contains_tool_id_token(&content, tool)) {
            offenders.push(file.display().to_string());
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-pipelines must not embed tool ids in source:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__purity_scans__tool_rosters_are_confined_to_registry_sources() {
    let root = repo_root();
    let mut offenders = Vec::new();
    let roster_tokens = [
        "\"fastp\"",
        "\"cutadapt\"",
        "\"samtools\"",
        "\"kraken2\"",
        "\"pydamage\"",
        "\"verifybamid2\"",
        "\"schmutzi\"",
    ];
    let allowed_paths = [
        "crates/bijux-dna-domain-compiler/src/lib.rs",
        "crates/bijux-dna-domain-compiler/src/compiler/support/status.rs",
        "crates/bijux-dna-environment-qa/src/image_qa/mod.rs",
        "crates/bijux-dna-environment/src/build.rs",
        "crates/bijux-dna-environment/src/build/defaults.rs",
        "crates/bijux-dna-core/src/id_catalog.rs",
        "crates/bijux-dna-planner-fastq/src/selection/tool_registry.rs",
        "crates/bijux-dna-planner-fastq/src/selection/tool_selection.rs",
        "crates/bijux-dna-planner-bam/src/selection/tool_registry.rs",
        "crates/bijux-dna-planner-bam/src/tool_adapters/tools/mod.rs",
        "crates/bijux-dna-domain-compiler/src/compiler_sections/domain_models_and_utils.rs",
        "crates/bijux-dna-domain-fastq/src/stages/contract.rs",
        "crates/bijux-dna-domain-bam/src/stage_specs/mod.rs",
        "crates/bijux-dna-api/src/internal/fastq/stages/preprocess/stage_backend_policy.rs",
        "crates/bijux-dna-environment-qa/src/image_qa/contracts.rs",
        "crates/bijux-dna-core/src/id_catalog/tool/fastq.rs",
        "crates/bijux-dna-core/src/id_catalog/tool/bam.rs",
        "crates/bijux-dna-domain-bam/src/artifacts.rs",
        "crates/bijux-dna-api/src/internal/handlers/cross/bam_exec_stage_postprocess.rs",
    ];

    for entry in WalkDir::new(root.join("crates"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path).to_string_lossy();
        if !rel.contains("/src/") {
            continue;
        }
        if allowed_paths.iter().any(|allowed| rel == *allowed) {
            continue;
        }
        let content = std::fs::read_to_string(path).expect("read source");
        let roster_hits = roster_tokens.iter().filter(|token| content.contains(*token)).count();
        if roster_hits >= 3 {
            offenders.push(rel.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "tool rosters must be declared only in registry compiler/domain loaders:\n{}",
        offenders.join("\n")
    );
}
