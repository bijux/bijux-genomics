#![cfg(test)]

use super::{
    admit_metadata_source, read_workspace_crate_manifests, workspace_edges_at,
    write_workspace_dependency_map, Result,
};
use cargo_metadata::MetadataCommand;
use std::collections::BTreeSet;
use std::fmt::Write as _;
use std::path::Path;

const API: &str = "bijux-dna-api";
const CORE: &str = "bijux-dna-core";

fn fixture(root: &Path, dependency: Option<(&str, &str)>) -> Result<()> {
    std::fs::create_dir_all(root)?;
    std::fs::write(
        root.join("Cargo.toml"),
        "[workspace]\nmembers = [\"bijux-dna-core\", \"bijux-dna-api\"]\nresolver = \"2\"\n",
    )?;
    for name in [API, CORE] {
        let member = root.join(name);
        std::fs::create_dir_all(member.join("src"))?;
        let mut manifest =
            format!("[package]\nname = \"{name}\"\nversion = \"0.1.0\"\nedition = \"2021\"\n");
        if let Some((from, to)) = dependency {
            if name == from {
                writeln!(manifest, "[dependencies]\n{to} = {{ path = \"../{to}\" }}")?;
            }
        }
        std::fs::write(member.join("Cargo.toml"), manifest)?;
        std::fs::write(member.join("src/lib.rs"), "pub const VALUE: u8 = 1;\n")?;
    }
    Ok(())
}

fn source_bytes(root: &Path) -> Result<Vec<Vec<u8>>> {
    let mut bytes = vec![std::fs::read(root.join("Cargo.toml"))?];
    for name in [API, CORE] {
        bytes.push(std::fs::read(root.join(name).join("Cargo.toml"))?);
        bytes.push(std::fs::read(root.join(name).join("src/lib.rs"))?);
    }
    Ok(bytes)
}

fn rejection<T>(result: Result<T>) -> Result<anyhow::Error> {
    match result {
        Ok(_) => anyhow::bail!("expected source or manifest rejection"),
        Err(error) => Ok(error),
    }
}

#[test]
fn dependency_map_binds_opposite_graphs_with_identical_crate_names() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "dependency_map_binds_opposite_graphs_with_identical_crate_names",
    );
    let first = temp.path().join("api-depends-core");
    let second = temp.path().join("core-depends-api");
    fixture(&first, Some((API, CORE)))?;
    fixture(&second, Some((CORE, API)))?;
    let first_bytes = source_bytes(&first)?;
    let second_bytes = source_bytes(&second)?;
    let cwd = std::env::current_dir()?;
    assert_eq!(workspace_edges_at(&first)?, BTreeSet::from([(API.into(), CORE.into())]));
    let output = second.join("reports/dependencies.json");
    let report = write_workspace_dependency_map(&second, &output)?;
    assert_eq!(report.schema_version, "bijux.crates.dependency_map.v1");
    assert_eq!(report.workspace_manifest, "Cargo.toml");
    assert_eq!(report.output_path, "reports/dependencies.json");
    assert_eq!(report.crate_count, 2);
    assert_eq!(report.edge_count, 1);
    assert_eq!(report.edges[0].from, CORE);
    assert_eq!(report.edges[0].to, API);
    assert_eq!(report.crates[0].crate_name, API);
    assert_eq!(report.crates[0].manifest_path, "bijux-dna-api/Cargo.toml");
    assert!(report.crates[0].direct_workspace_dependencies.is_empty());
    assert_eq!(report.crates[0].direct_workspace_dependents, [CORE]);
    assert_eq!(report.crates[1].direct_workspace_dependencies, [API]);
    assert!(report.crates[1].direct_workspace_dependents.is_empty());
    let persisted: serde_json::Value = serde_json::from_slice(&std::fs::read(output)?)?;
    assert_eq!(persisted, serde_json::to_value(report)?);
    assert_eq!(source_bytes(&first)?, first_bytes);
    assert_eq!(source_bytes(&second)?, second_bytes);
    assert_eq!(std::env::current_dir()?, cwd);
    Ok(())
}

#[test]
fn dependency_map_retains_isolated_same_name_members_without_foreign_edges() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "dependency_map_retains_isolated_same_name_members_without_foreign_edges",
    );
    fixture(temp.path(), None)?;
    let source = source_bytes(temp.path())?;
    assert!(workspace_edges_at(temp.path())?.is_empty());
    let report = write_workspace_dependency_map(temp.path(), &temp.path().join("graph.json"))?;
    assert_eq!(report.crate_count, 2);
    assert_eq!(report.edge_count, 0);
    assert!(report.edges.is_empty());
    for node in &report.crates {
        assert!(node.direct_workspace_dependencies.is_empty());
        assert!(node.direct_workspace_dependents.is_empty());
    }
    assert_eq!(source_bytes(temp.path())?, source);
    Ok(())
}

#[test]
fn metadata_source_admission_rejects_another_real_workspace() -> Result<()> {
    let temp =
        bijux_dna_testkit::tempdir_for("metadata_source_admission_rejects_another_real_workspace");
    let expected = temp.path().join("expected");
    let actual = temp.path().join("actual");
    fixture(&expected, None)?;
    fixture(&actual, Some((CORE, API)))?;
    let metadata = MetadataCommand::default()
        .manifest_path(actual.join("Cargo.toml"))
        .current_dir(&actual)
        .exec()?;
    let error = rejection(admit_metadata_source(&expected.canonicalize()?, &metadata))?;
    assert!(error.to_string().contains("does not match requested source root"));
    admit_metadata_source(&actual.canonicalize()?, &metadata)?;
    Ok(())
}

#[test]
fn metadata_source_admission_rejects_member_directory_resolved_to_parent() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metadata_source_admission_rejects_member_directory_resolved_to_parent",
    );
    fixture(temp.path(), None)?;
    let error = rejection(workspace_edges_at(&temp.path().join(API)))?;
    assert!(error.to_string().contains("does not match requested source root"));
    Ok(())
}

#[test]
fn dependency_map_preserves_manifest_failure_priority_without_publication() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "dependency_map_preserves_manifest_failure_priority_without_publication",
    );
    let output = temp.path().join("report.json");
    let manifest = temp.path().join("Cargo.toml");
    let error = rejection(write_workspace_dependency_map(temp.path(), &output))?;
    assert_eq!(error.to_string(), format!("read {}", manifest.display()));
    std::fs::write(&manifest, "[workspace]\nmembers = [17]\n")?;
    let error = rejection(write_workspace_dependency_map(temp.path(), &output))?;
    assert_eq!(error.to_string(), "workspace.members must contain only string paths");
    fixture(temp.path(), None)?;
    let member = temp.path().join(CORE).join("Cargo.toml");
    std::fs::write(&member, "[package]\nversion = \"0.1.0\"\n")?;
    let error = rejection(write_workspace_dependency_map(temp.path(), &output))?;
    assert_eq!(error.to_string(), format!("package.name missing from {}", member.display()));
    assert!(!output.exists());
    Ok(())
}

#[test]
fn manifest_reader_preserves_declared_paths_and_sorts_by_crate_name() -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "manifest_reader_preserves_declared_paths_and_sorts_by_crate_name",
    );
    fixture(temp.path(), None)?;
    let members = read_workspace_crate_manifests(temp.path())?;
    assert_eq!(
        members.iter().map(|member| member.crate_name.as_str()).collect::<Vec<_>>(),
        [API, CORE]
    );
    assert_eq!(members[0].manifest_path, temp.path().join(API).join("Cargo.toml"));
    assert_eq!(members[1].manifest_path, temp.path().join(CORE).join("Cargo.toml"));
    assert!(!temp.path().join("Cargo.lock").exists());
    Ok(())
}

#[cfg(unix)]
#[test]
fn metadata_source_admission_accepts_symlink_root_alias_without_rewriting_report_paths(
) -> Result<()> {
    let temp = bijux_dna_testkit::tempdir_for(
        "metadata_source_admission_accepts_symlink_root_alias_without_rewriting_report_paths",
    );
    let physical = temp.path().join("physical");
    let alias = temp.path().join("alias");
    fixture(&physical, Some((CORE, API)))?;
    std::os::unix::fs::symlink(&physical, &alias)?;
    assert_eq!(workspace_edges_at(&alias)?, BTreeSet::from([(CORE.into(), API.into())]));
    let members = read_workspace_crate_manifests(&alias)?;
    assert_eq!(members[0].manifest_path, alias.join(API).join("Cargo.toml"));
    let report = write_workspace_dependency_map(&alias, &alias.join("report.json"))?;
    assert_eq!(report.workspace_manifest, "Cargo.toml");
    assert_eq!(report.output_path, "report.json");
    assert_eq!(report.edges[0].from, CORE);
    Ok(())
}
