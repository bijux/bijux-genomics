#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

#[test]
fn policy__contracts__apptainer_def_location_policy__defs_only_exist_in_bijux_or_non_bijux_dirs() {
    let root = support::workspace_root();
    let mut offenders = Vec::new();

    for entry in WalkDir::new(root.join("containers"))
        .into_iter()
        .filter_map(Result::ok)
    {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("def") {
            continue;
        }
        let rel = path.strip_prefix(&root).unwrap_or(path);
        let rel_s = rel.to_string_lossy();
        if !rel_s.starts_with("containers/apptainer/bijux/")
            && !rel_s.starts_with("containers/apptainer/non-bijux/")
        {
            offenders.push(rel_s.to_string());
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        ".def files are only allowed in containers/apptainer/bijux/ or containers/apptainer/non-bijux/:\n{}",
        offenders.join("\n")
    );
}
