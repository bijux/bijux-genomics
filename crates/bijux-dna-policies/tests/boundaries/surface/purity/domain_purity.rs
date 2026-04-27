#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

fn scan_dir_for_tokens(root: &Path, tokens: &[&str]) -> Vec<String> {
    let mut offenders = Vec::new();
    for entry in walkdir::WalkDir::new(root).min_depth(1).max_depth(6) {
        let entry =
            entry.unwrap_or_else(|err| panic!("walk entry under {}: {err}", root.display()));
        if !entry.file_type().is_file() {
            continue;
        }
        if entry.path().extension().and_then(|s| s.to_str()) != Some("rs") {
            continue;
        }
        let content = std::fs::read_to_string(entry.path())
            .unwrap_or_else(|err| panic!("read {}: {err}", entry.path().display()));
        for token in tokens {
            if content.contains(token) {
                offenders.push(format!("{} contains `{}`", entry.path().display(), token));
            }
        }
    }
    offenders
}

#[test]
fn policy__boundaries__domain_purity__domains_have_no_registry_logic() {
    let root = repo_root();
    let domains = [
        root.join("crates/bijux-dna-domain-fastq/src"),
        root.join("crates/bijux-dna-domain-bam/src"),
    ];
    let tokens = ["registry", "tool_registry", "stage_registry"];
    let mut offenders = Vec::new();
    for domain in domains {
        offenders.extend(scan_dir_for_tokens(&domain, &tokens));
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Domain crates must not contain registry/selection logic.\n\
Move registry/selection into planners.\n\
See docs/40-policies/STYLE.md for domain purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__domain_purity__domains_have_no_execution_details() {
    let root = repo_root();
    let domains = [
        root.join("crates/bijux-dna-domain-fastq/src"),
        root.join("crates/bijux-dna-domain-bam/src"),
    ];
    let tokens = ["CommandSpec", "ContainerImage", "command_template", "argv", "docker"];
    let mut offenders = Vec::new();
    for domain in domains {
        offenders.extend(scan_dir_for_tokens(&domain, &tokens));
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "Domain crates must not reference execution details.\n\
Move execution wiring into planners/runners.\n\
See docs/40-policies/STYLE.md for domain purity rules.\n\
Offenders:\n{}",
        offenders.join("\n")
    );
}
