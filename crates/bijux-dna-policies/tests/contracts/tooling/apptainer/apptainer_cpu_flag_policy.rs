#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

use support::workspace_root;

#[test]
fn policy__contracts__apptainer_cpu_flag_policy__no_arch_flag_injection_without_justification() {
    let root = workspace_root().join("containers").join("apptainer");
    let mut offenders = Vec::new();
    let whitelist = ["star"];

    for entry in WalkDir::new(&root) {
        let Ok(entry) = entry else { continue };
        let path = entry.path();
        if !entry.file_type().is_file() {
            continue;
        }
        if path.extension().and_then(|ext| ext.to_str()) != Some("def") {
            continue;
        }
        let tool_id =
            path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default().to_string();
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        let justified = content.contains("APPTAINER_CPU_FLAG_JUSTIFIED:")
            || whitelist.contains(&tool_id.as_str());
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
        "Apptainer defs must not inject CPU arch flags without whitelist or APPTAINER_CPU_FLAG_JUSTIFIED note:\n{}",
        offenders.join("\n")
    );
}
