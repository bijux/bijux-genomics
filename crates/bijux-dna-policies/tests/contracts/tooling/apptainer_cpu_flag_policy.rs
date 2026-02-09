#![allow(non_snake_case)]
#[path = "../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

#[test]
fn policy__contracts__apptainer_cpu_flag_policy__no_arch_flag_injection_without_justification() {
    let root = workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();

    for entry in WalkDir::new(&root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("def") {
            continue;
        }
        let content = match std::fs::read_to_string(path) {
            Ok(content) => content,
            Err(_) => continue,
        };
        let justified = content.contains("APPTAINER_CPU_FLAG_JUSTIFIED:");
        for (idx, line) in content.lines().enumerate() {
            let lowered = line.to_ascii_lowercase();
            let has_arch_flag = lowered.contains("-march=")
                || lowered.contains("-mcpu=")
                || lowered.contains("-mtune=")
                || lowered.contains(" -msse")
                || lowered.contains(" -mavx");
            if has_arch_flag && !justified {
                offenders.push(format!("{}:{}", path.display(), idx + 1));
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Apptainer defs must not inject CPU arch flags without APPTAINER_CPU_FLAG_JUSTIFIED note:\n{}",
        offenders.join("\n")
    );
}
