#![allow(non_snake_case)]
#[path = "../../../support/fs.rs"]
mod support;

use walkdir::WalkDir;

fn crate_dependencies(root: &std::path::Path, name: &str) -> std::collections::BTreeSet<String> {
    let metadata = cargo_metadata::MetadataCommand::new()
        .manifest_path(root.join("Cargo.toml"))
        .exec()
        .unwrap_or_else(|err| panic!("cargo metadata: {err}"));
    let pkg = metadata
        .packages
        .iter()
        .find(|p| p.name == name)
        .unwrap_or_else(|| panic!("missing package {name}"));
    pkg.dependencies.iter().map(|dep| dep.name.clone()).collect::<std::collections::BTreeSet<_>>()
}

fn is_domain_effect_allowlisted(path: &std::path::Path) -> bool {
    let path_str = path.to_string_lossy();
    path_str.contains("/crates/bijux-dna-domain-fastq/src/stages/contract/runtime/")
        || path_str.ends_with("/crates/bijux-dna-domain-bam/src/artifacts.rs")
}

fn scan_forbidden_patterns(base: &std::path::Path, forbidden: &[&str]) -> Vec<String> {
    let mut offenders = Vec::new();
    for entry in WalkDir::new(base).into_iter().filter_map(Result::ok) {
        if !entry.file_type().is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }
        if path.to_string_lossy().contains("/tests/") || is_domain_effect_allowlisted(path) {
            continue;
        }
        let content = support::read_to_string(path);
        for needle in forbidden {
            if content.contains(needle) {
                offenders.push(format!("{} contains `{}`", path.display(), needle));
            }
        }
    }
    offenders
}

#[test]
fn policy__contracts__purity_effects_responsibility_policy__domain_crates_have_no_fs_or_network_effects(
) {
    let root = support::workspace_root();
    let forbidden = [
        "std::fs::write",
        "std::fs::File::create",
        "std::fs::create_dir",
        "std::fs::create_dir_all",
        "std::fs::remove_file",
        "std::fs::remove_dir",
        "std::fs::remove_dir_all",
        "tokio::fs::write",
        "tokio::fs::File::create",
        "tokio::fs::create_dir",
        "tokio::fs::create_dir_all",
        "tokio::fs::remove_file",
        "tokio::fs::remove_dir",
        "tokio::fs::remove_dir_all",
        "std::net::",
        "tokio::net::",
        "reqwest::",
        "ureq::",
        "hyper::",
        "std::process::Command",
        "Command::new(",
    ];
    let mut offenders = Vec::new();
    for rel in ["crates/bijux-dna-domain-fastq/src", "crates/bijux-dna-domain-bam/src"] {
        offenders.extend(scan_forbidden_patterns(&root.join(rel), &forbidden));
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "domain crates must remain pure (no filesystem mutation/network/process effects):\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__purity_effects_responsibility_policy__stages_crates_define_invocations_and_parsers_only(
) {
    let root = support::workspace_root();
    let forbidden = [
        "std::fs::write",
        "std::fs::File::create",
        "std::fs::create_dir",
        "std::fs::create_dir_all",
        "std::fs::remove_file",
        "std::fs::remove_dir",
        "std::fs::remove_dir_all",
        "tokio::fs::write",
        "tokio::fs::File::create",
        "tokio::fs::create_dir",
        "tokio::fs::create_dir_all",
        "tokio::fs::remove_file",
        "tokio::fs::remove_dir",
        "tokio::fs::remove_dir_all",
        "std::net::",
        "tokio::net::",
        "reqwest::",
        "ureq::",
        "hyper::",
        "std::process::Command",
        "Command::new(",
        "docker ",
        "apptainer ",
    ];
    let mut offenders = Vec::new();
    for rel in ["crates/bijux-dna-stages-fastq/src", "crates/bijux-dna-stages-bam/src"] {
        offenders.extend(scan_forbidden_patterns(&root.join(rel), &forbidden));
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stage crates must not execute mutation/network/process effects; only invocation contracts/parsers are allowed:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__contracts__purity_effects_responsibility_policy__planner_engine_runner_environment_split_is_enforced(
) {
    let root = support::workspace_root();
    let planner_fastq = crate_dependencies(&root, "bijux-dna-planner-fastq");
    let planner_bam = crate_dependencies(&root, "bijux-dna-planner-bam");
    let engine = crate_dependencies(&root, "bijux-dna-engine");
    let runner = crate_dependencies(&root, "bijux-dna-runner");

    for dep in ["bijux-dna-runner", "bijux-dna-environment"] {
        bijux_dna_policies::policy_assert!(
            !planner_fastq.contains(dep),
            "planner-fastq must not depend on {dep}"
        );
        bijux_dna_policies::policy_assert!(
            !planner_bam.contains(dep),
            "planner-bam must not depend on {dep}"
        );
    }

    for dep in ["bijux-dna-planner-fastq", "bijux-dna-planner-bam"] {
        bijux_dna_policies::policy_assert!(
            !engine.contains(dep),
            "engine must not depend on planning crates ({dep})"
        );
    }

    bijux_dna_policies::policy_assert!(
        runner.contains("bijux-dna-environment"),
        "runner must depend on environment for image/runtime resolution"
    );
}
