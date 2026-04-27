#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

#[test]
fn policy__contracts__apptainer_purge_policy__no_purge_or_autoremove_in_defs() {
    let root = workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("def") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            let lowered = line.to_ascii_lowercase();
            if lowered.contains("apt-get purge")
                || lowered.contains("apt purge")
                || lowered.contains("apt-get autoremove")
                || lowered.contains("apt autoremove")
                || lowered.contains("apt-mark auto")
            {
                offenders.push(format!("{}:{}", path.display(), idx + 1));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Apptainer defs must not use apt purge/autoremove/apt-mark auto; remove source dirs only:\n{}",
        offenders.join("\n")
    );
}
