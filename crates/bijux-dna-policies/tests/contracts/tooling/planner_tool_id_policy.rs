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
