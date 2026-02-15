#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__apptainer_header_policy__bijux_defs_start_with_exact_bijux_header() {
    let root = support::workspace_root();
    let bijux_root = root.join("containers").join("apptainer").join("bijux");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&bijux_root).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("def") {
            continue;
        }
        let content = std::fs::read_to_string(path).unwrap_or_default();
        if !(content.starts_with("# Container definition license:")
            && content.contains("part of bijux-dna")
            && content.contains("Apache-2.0"))
        {
            offenders.push(path.display().to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-owned apptainer .def headers must match exact required header:\n{}",
        offenders.join("\n")
    );
}
