#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

const OWNER_ALLOWLIST: &[(&str, &str)] = &[
    ("StageId", "/crates/bijux-dna-core/"),
    ("ToolId", "/crates/bijux-dna-core/"),
    ("PipelineId", "/crates/bijux-dna-core/"),
    ("MetricId", "/crates/bijux-dna-core/"),
];

#[test]
fn policy__boundaries__ssot_catalog_authority__ssot_catalog_authority() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();
    for entry in WalkDir::new(root.join("crates")).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        let path_str = entry.path().to_string_lossy();
        if path_str.contains("/tests/") {
            continue;
        }
        if path_str.contains("/crates/bijux-dna-api/src/runtime/execution_kernel.rs")
            || path_str
                .contains("/crates/bijux-dna-api/src/internal/handlers/cross/bam_exec_contracts.rs")
        {
            continue;
        }
        let content = support::read_to_string(entry.path());
        for (id_type, owner) in OWNER_ALLOWLIST {
            if content.contains(id_type) && !path_str.contains(owner) {
                // Allow references (imports), but ban newtype definitions or const literals.
                if content.contains(&format!("struct {id_type}"))
                    || content.contains(&format!("enum {id_type}"))
                    || content.contains("StageId::new(\"")
                    || content.contains("ToolId::new(\"")
                    || content.contains("PipelineId::new(\"")
                    || content.contains("MetricId::new(\"")
                {
                    offenders.push(entry.path().display().to_string());
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "SSOT catalog authority violated.\n\
Fix by moving ID ownership to the canonical crate, and importing IDs elsewhere.\n\
See docs/40-policies/STYLE.md for SSOT rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
