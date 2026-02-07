use std::path::{Path, PathBuf};

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf()
}

fn scan_dir_for_tokens(root: &Path, tokens: &[&str]) -> Vec<String> {
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1).max_depth(6) {
        let entry = entry.expect("walk entry");
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path()).expect("read source");
        for token in tokens {
            if content.contains(token) {
                offenders.push(format!("{} contains `{}`", entry.path().display(), token));
            }
        }
    }
    offenders
}

#[test]
fn domains_have_no_registry_logic() {
    let root = workspace_root();
    let domains = [
        root.join("crates/bijux-domain-fastq/src"),
        root.join("crates/bijux-domain-bam/src"),
    ];
    let tokens = ["registry", "tool_registry", "stage_registry"];
    let mut offenders = Vec::new();
    for domain in domains {
        offenders.extend(scan_dir_for_tokens(&domain, &tokens));
    }
    assert!(
        offenders.is_empty(),
        "Domain crates must not contain registry/selection logic.\n\
Move registry/selection into planners.\n\
See docs/STYLE.md for domain purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn domains_have_no_execution_details() {
    let root = workspace_root();
    let domains = [
        root.join("crates/bijux-domain-fastq/src"),
        root.join("crates/bijux-domain-bam/src"),
    ];
    let tokens = [
        "CommandSpec",
        "ContainerImage",
        "command_template",
        "argv",
        "docker",
    ];
    let mut offenders = Vec::new();
    for domain in domains {
        offenders.extend(scan_dir_for_tokens(&domain, &tokens));
    }
    assert!(
        offenders.is_empty(),
        "Domain crates must not reference execution details.\n\
Move execution wiring into planners/runners.\n\
See docs/STYLE.md for domain purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
