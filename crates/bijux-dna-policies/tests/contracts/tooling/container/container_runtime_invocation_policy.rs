#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use regex::Regex;
use walkdir::WalkDir;

#[test]
fn policy__contracts__container_runtime_invocation_policy__only_scripts_containers_may_invoke_runtimes_directly(
) {
    let root = support::workspace_root();
    let scripts_root = root.join("scripts");
    let allowed_root = root.join("bijux-dna-dev").join("containers");
    let mut roots = vec![scripts_root];
    roots.push(allowed_root.clone());
    let invoke_re =
        Regex::new(r"(^|[;&|()[:space:]])(sudo[[:space:]]+)?(docker|apptainer)[[:space:]]")
            .expect("compile runtime invocation regex");

    let mut offenders = Vec::new();
    for root in roots {
        if !root.is_dir() {
            continue;
        }
        for entry in WalkDir::new(&root).into_iter().filter_map(Result::ok) {
            if !entry.file_type().is_file() {
                continue;
            }
            let path = entry.path();
            let ext = path.extension().and_then(std::ffi::OsStr::to_str);
            if ext != Some("sh") && ext != Some("bash") && ext != Some("zsh") {
                continue;
            }
            if path.starts_with(&allowed_root) {
                continue;
            }
            let content = std::fs::read_to_string(path).unwrap_or_default();
            for (lineno, line) in content.lines().enumerate() {
                let trimmed = line.trim_start();
                if trimmed.starts_with('#') {
                    continue;
                }
                if invoke_re.is_match(line) {
                    offenders.push(format!("{}:{}", path.display(), lineno + 1));
                }
            }
        }
    }

    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "only bijux-dna-dev/containers may invoke docker/apptainer directly. Offenders:\n{}",
        offenders.join("\n")
    );
}
