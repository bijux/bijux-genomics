#![allow(non_snake_case)]
use std::path::{Path, PathBuf};

use crate::cargo_dependency_authority::{
    read_dependency_names as parse_dependency_names, CargoDependencySnapshot,
};

fn repo_root() -> PathBuf {
    bijux_dna_testkit::workspace_root_from_manifest(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn policy__boundaries__benchmark_dependency_policy__benchmark_has_no_planner_or_stage_dependencies()
{
    let root = repo_root();
    let authority = CargoDependencySnapshot::capture(&root.join("Cargo.toml"));
    let manifest = root.join("crates/bijux-dna-bench/Cargo.toml");
    let deps = authority.dependency_names(&manifest);
    let denylist = [
        "bijux-dna-stages-fastq",
        "bijux-dna-stages-bam",
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
        "bijux-dna-bench must not depend on stages/planners:\n{}",
        offenders.join("\n")
    );
}

fn benchmark_dependency_fixture(
    root: &Path,
    section: &str,
    dependency: &str,
) -> anyhow::Result<PathBuf> {
    std::fs::create_dir_all(root.join("src"))?;
    std::fs::create_dir_all(root.join("planner/src"))?;
    std::fs::write(root.join("src/lib.rs"), "pub fn fixture() {}\n")?;
    std::fs::write(root.join("planner/src/lib.rs"), "pub fn fixture() {}\n")?;
    std::fs::write(
        root.join("planner/Cargo.toml"),
        "[package]\nname = \"bijux-dna-planner-bam\"\nversion = \"0.1.0\"\nedition = \"2021\"\n",
    )?;
    let manifest = root.join("Cargo.toml");
    let source = format!("[package]\nname = \"dependency-observer\"\nversion = \"0.1.0\"\nedition = \"2021\"\n[workspace]\nmembers = [\"planner\"]\n[package.metadata.observer]\nlabel = \"permitted selective metadata\"\n[{section}]\n{dependency}\n");
    let _: toml::Value = toml::from_str(&source)?;
    std::fs::write(&manifest, source)?;
    Ok(manifest)
}

#[test]
fn benchmark_dependency_reader_resolves_aliased_package_identity() -> anyhow::Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("benchmark-dependency-authority");
    let manifest = benchmark_dependency_fixture(
        owned.path(),
        "dependencies",
        "bijux-dna-planner-bam = { path = \"planner\" }",
    )?;
    assert!(parse_dependency_names(&manifest).contains(&"bijux-dna-planner-bam".to_string()));
    let manifest = benchmark_dependency_fixture(
        owned.path(),
        "dependencies",
        "planner_alias = { package = \"bijux-dna-planner-bam\", path = \"planner\" }",
    )?;
    assert!(
        parse_dependency_names(&manifest).contains(&"bijux-dna-planner-bam".to_string()),
        "supported Cargo dependency must retain package identity through aliased_package_identity"
    );
    Ok(())
}

#[test]
fn benchmark_dependency_reader_resolves_target_specific_edges() -> anyhow::Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("benchmark-dependency-authority");
    let manifest = benchmark_dependency_fixture(
        owned.path(),
        "dependencies",
        "bijux-dna-planner-bam = { path = \"planner\" }",
    )?;
    assert!(parse_dependency_names(&manifest).contains(&"bijux-dna-planner-bam".to_string()));
    let manifest = benchmark_dependency_fixture(
        owned.path(),
        "target.'cfg(unix)'.dependencies",
        "bijux-dna-planner-bam = { path = \"planner\" }",
    )?;
    assert!(
        parse_dependency_names(&manifest).contains(&"bijux-dna-planner-bam".to_string()),
        "supported Cargo dependency must retain package identity through target_specific_edges"
    );
    Ok(())
}

#[test]
fn dependency_observations_are_request_owned_and_refuse_foreign_manifest_identity(
) -> anyhow::Result<()> {
    let owned = bijux_dna_testkit::tempdir_for("request-owned-dependency-authority");
    let manifest = benchmark_dependency_fixture(
        owned.path(),
        "target.'cfg(unix)'.build-dependencies",
        "planner_alias = { package = \"bijux-dna-planner-bam\", path = \"planner\" }",
    )?;
    let observation = CargoDependencySnapshot::capture(&manifest);
    let dependency = &observation.dependencies(&manifest)[0];
    assert_eq!(dependency.name, "bijux-dna-planner-bam");
    assert_eq!(dependency.rename.as_deref(), Some("planner_alias"));
    assert_eq!(dependency.kind, cargo_metadata::DependencyKind::Build);
    assert_eq!(dependency.target.as_ref().map(ToString::to_string).as_deref(), Some("cfg(unix)"));
    let known = std::collections::BTreeSet::from(["bijux-dna-planner-bam".to_string()]);
    assert_eq!(observation.known_names(&manifest, &known), known);
    assert!(observation.known_names(&manifest, &std::collections::BTreeSet::new()).is_empty());
    assert!(observation.dependency_names(&owned.path().join("planner/Cargo.toml")).is_empty());

    benchmark_dependency_fixture(owned.path(), "dependencies", "")?;
    assert_eq!(observation.dependency_names(&manifest), ["bijux-dna-planner-bam"]);
    let fresh = CargoDependencySnapshot::capture(&manifest);
    assert!(fresh.dependency_names(&manifest).is_empty());

    let workspace = owned.path().join("virtual-workspace");
    let member = benchmark_dependency_fixture(&workspace.join("consumer"), "dependencies", "")?;
    let member_source = std::fs::read_to_string(&member)?;
    std::fs::write(&member, member_source.replace("[workspace]\nmembers = [\"planner\"]\n", ""))?;
    let workspace_manifest = workspace.join("Cargo.toml");
    std::fs::write(
        &workspace_manifest,
        "[workspace]\nmembers = [\"consumer\", \"consumer/planner\"]\nresolver = \"2\"\n",
    )?;
    let workspace_observation = CargoDependencySnapshot::capture(&workspace_manifest);
    assert!(workspace_observation.dependency_names(&member).is_empty());
    assert!(workspace_observation
        .dependency_names(&workspace.join("consumer/planner/Cargo.toml"))
        .is_empty());
    assert!(std::panic::catch_unwind(
        || workspace_observation.dependency_names(&workspace_manifest)
    )
    .is_err());

    let foreign = bijux_dna_testkit::tempdir_for("foreign-dependency-authority");
    let foreign_manifest = benchmark_dependency_fixture(foreign.path(), "dependencies", "")?;
    assert!(std::panic::catch_unwind(|| observation.dependency_names(&foreign_manifest)).is_err());
    assert!(std::panic::catch_unwind(
        || observation.dependency_names(&owned.path().join("missing/Cargo.toml"))
    )
    .is_err());
    std::fs::write(&manifest, "[package\n")?;
    assert!(std::panic::catch_unwind(|| CargoDependencySnapshot::capture(&manifest)).is_err());
    assert_eq!(observation.dependency_names(&manifest), ["bijux-dna-planner-bam"]);
    Ok(())
}
