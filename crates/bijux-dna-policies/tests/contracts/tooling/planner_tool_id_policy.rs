#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;
use regex::Regex;

#[test]
fn policy__contracts__planner_tool_id_policy__selection_registries_use_toolid_not_string_literals()
{
    let root = support::workspace_root();
    let files = [
        "crates/bijux-dna-planner-fastq/src/selection/tool_selection.rs",
        "crates/bijux-dna-planner-bam/src/selection/tool_selection.rs",
    ];
    let mut offenders = Vec::new();
    let tuple_literal = Regex::new(r#"\(\s*"[a-z0-9_\-]+"\s*,"#).expect("compile tuple regex");
    let raw_tool_literal =
        Regex::new(r#""[a-z0-9_\-]+"\.to_string\(\)"#).expect("compile literal regex");

    for rel in files {
        let path = root.join(rel);
        let content = std::fs::read_to_string(&path).expect("read selection registry");
        if !content.contains("ToolId::from_static(") && !content.contains("ToolId::new(") {
            offenders.push(format!(
                "{}: missing ToolId construction (ToolId::new/from_static)",
                path.display()
            ));
        }
        if tuple_literal.is_match(&content) {
            offenders.push(format!(
                "{}: raw string tuple tool ids are banned; use ToolId::from_static",
                path.display()
            ));
        }
        if raw_tool_literal.is_match(&content) && !content.contains("ToolId::new(tool.to_string())")
        {
            offenders.push(format!(
                "{}: raw string tool ids are banned in selection modules; wrap in ToolId",
                path.display()
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "planner tool-id policy violations:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__planner_tool_id_policy__planners_ban_silent_contract_fallbacks() {
    let root = support::workspace_root();
    let planner_files = [
        "crates/bijux-dna-planner-fastq/src/lib.rs",
        "crates/bijux-dna-planner-bam/src/lib.rs",
        "crates/bijux-dna-planner-bam/src/stages/stage_catalog.rs",
        "crates/bijux-dna-planner-bam/src/selection/tool_selection.rs",
        "crates/bijux-dna-planner-fastq/src/selection/tool_selection.rs",
        "crates/bijux-dna-planner-vcf/src/lib.rs",
    ];
    let mut offenders = Vec::new();
    let fallback_on_contract = Regex::new(
        r"contract_for_stage\s*\([^)]*\)\s*\.\s*(unwrap_or|unwrap_or_else|map_or|map_or_else)\s*\(",
    )
    .expect("compile contract-fallback regex");

    for rel in planner_files {
        let path = root.join(rel);
        let content = std::fs::read_to_string(&path).expect("read planner source");
        if fallback_on_contract.is_match(&content) {
            offenders.push(format!(
                "{}: planner contract lookup must hard-fail; fallback helper chained on contract_for_stage is banned",
                path.display()
            ));
        }
        if content.contains("fallback_tool_for_stage(") {
            offenders.push(format!(
                "{}: fallback_tool_for_stage is banned; planner selection must be registry/domain-driven and explicit",
                path.display()
            ));
        }
        if content.contains("defaults_ledger") {
            offenders.push(format!(
                "{}: planners must not embed defaults ledger concerns; keep defaults in pipelines layer only",
                path.display()
            ));
        }
    }

    assert!(
        offenders.is_empty(),
        "planner contract-fallback policy violations:\n{}",
        offenders.join("\n")
    );
}
