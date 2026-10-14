#![allow(non_snake_case)]
use std::path::PathBuf;

use crate::cargo_dependency_authority::{
    read_dependency_names as parse_dependency_names, CargoDependencySnapshot,
};

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__boundaries__dependency_boundaries__stages_do_not_depend_on_environment() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifests = [
        root.join("crates/bijux-dna-stages-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-stages-bam/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = authority.dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-dna-containers") {
            offenders.push(format!("{} depends on bijux-dna-containers", manifest.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stages crates must not depend on bijux-dna-containers:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__stages_and_planners_do_not_depend_on_runner_or_engine(
) {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifests = [
        root.join("crates/bijux-dna-stages-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-stages-bam/Cargo.toml"),
        root.join("crates/bijux-dna-planner-fastq/Cargo.toml"),
        root.join("crates/bijux-dna-planner-bam/Cargo.toml"),
    ];
    let denylist = ["bijux-dna-runner", "bijux-dna-engine"];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = authority.dependency_names(&manifest);
        for denied in denylist {
            if deps.iter().any(|dep| dep == denied) {
                offenders.push(format!("{} depends on {}", manifest.display(), denied));
            }
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "stages/planners must not depend on runner or engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__analyze_and_benchmark_do_not_depend_on_engine() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifests = [
        root.join("crates/bijux-dna-analyze/Cargo.toml"),
        root.join("crates/bijux-dna-bench/Cargo.toml"),
        root.join("crates/bijux-dna-bench-model/Cargo.toml"),
    ];
    let mut offenders = Vec::new();
    for manifest in manifests {
        let deps = authority.dependency_names(&manifest);
        if deps.iter().any(|dep| dep == "bijux-dna-engine") {
            offenders.push(format!("{} depends on bijux-dna-engine", manifest.display()));
        }
    }
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "analyze/benchmark crates must not depend on bijux-dna-engine:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__engine_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-engine/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-engine must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__runner_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-runner/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-pipelines",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-runner must not depend on domain/stages/pipelines:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__runner_does_not_depend_on_engine() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-runner/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    bijux_dna_policies::policy_assert!(
        deps.iter().all(|dep| dep != "bijux-dna-engine"),
        "bijux-dna-runner must not depend on bijux-dna-engine"
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__infra_has_no_domain_or_stage_dependencies() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-infra/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = [
        "bijux-dna-domain-fastq",
        "bijux-dna-domain-bam",
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-planner-fastq",
        "bijux-dna-planner-bam",
        "bijux-dna-pipelines",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-infra must not depend on domain/stages/planners:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__pipelines_do_not_depend_on_stages_or_execution() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-pipelines/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = [
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
        "bijux-dna-engine",
        "bijux-dna-runner",
        "bijux-dna-containers",
    ];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-pipelines must not depend on stages/engine/runner:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__environment_has_no_engine_or_runner_dependencies() {
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-containers/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = ["bijux-dna-engine", "bijux-dna-runner"];
    let offenders: Vec<String> = denylist
        .iter()
        .filter(|dep| deps.iter().any(|name| name == **dep))
        .map(|dep| format!("{} depends on {}", manifest.display(), dep))
        .collect();
    bijux_dna_policies::policy_assert!(
        offenders.is_empty(),
        "bijux-dna-containers must not depend on engine/runner:\n{}",
        offenders.join("\n")
    );
}

#[test]
fn policy__boundaries__dependency_boundaries__dependency_authority_resolves_aliases_workspace_and_target_specific_edges(
) -> anyhow::Result<()> {
    let root = bijux_dna_testkit::tempdir_for("dependency-authority");
    let root = root.path();
    for package in ["consumer", "runner-owner", "shared-owner", "build-owner", "test-owner"] {
        let dir = root.join(package);
        std::fs::create_dir_all(dir.join("src"))?;
        std::fs::write(dir.join("src/lib.rs"), "pub fn present() {}\n")?;
        std::fs::write(
            dir.join("Cargo.toml"),
            format!("[package]\nname = \"{package}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n"),
        )?;
    }
    std::fs::write(
        root.join("Cargo.toml"),
        r#"[workspace]
members = ["consumer", "runner-owner", "shared-owner", "build-owner", "test-owner"]
resolver = "2"
[workspace.dependencies]
shared_alias = { package = "shared-owner", path = "shared-owner" }
"#,
    )?;
    let manifest = root.join("consumer/Cargo.toml");
    let mut content = std::fs::read_to_string(&manifest)?;
    content.push_str(
        r#"
[dependencies]
runner_alias = { package = "runner-owner", path = "../runner-owner" }
shared_alias.workspace = true
[target.'cfg(unix)'.build-dependencies]
builder = { package = "build-owner", path = "../build-owner" }
[dev-dependencies.fixture]
package = "test-owner"
path = "../test-owner"
"#,
    );
    std::fs::write(&manifest, content)?;
    let observed =
        parse_dependency_names(&manifest).into_iter().collect::<std::collections::BTreeSet<_>>();
    assert_eq!(
        observed,
        ["runner-owner", "shared-owner", "build-owner", "test-owner"]
            .into_iter()
            .map(str::to_string)
            .collect()
    );
    Ok(())
}
