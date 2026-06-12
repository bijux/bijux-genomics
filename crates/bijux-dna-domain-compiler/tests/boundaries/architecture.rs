use std::collections::BTreeSet;
use std::path::Path;

#[test]
#[allow(clippy::too_many_lines)]
fn crate_tree_matches_domain_compiler_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(dir_entries(root), expected_root, "domain-compiler root must stay minimal");

    let expected_docs: BTreeSet<_> = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "COMMANDS.md",
        "CONTRACTS.md",
        "DEPENDENCIES.md",
        "EFFECTS.md",
        "INDEX.md",
        "PUBLIC_API.md",
        "SCOPE.md",
        "TESTS.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let docs_entries = dir_entries(&root.join("docs"));
    assert_eq!(docs_entries.len(), 10, "domain-compiler docs allowance is 10 Markdown files");
    assert_eq!(docs_entries, expected_docs, "domain-compiler docs spine must stay explicit");

    let expected_src: BTreeSet<_> =
        ["bin/", "compiler/", "lib.rs"].into_iter().map(str::to_string).collect();
    assert_eq!(dir_entries(&root.join("src")), expected_src, "source tree changed");

    let expected_compiler: BTreeSet<_> = [
        "bundle.rs",
        "compile.rs",
        "coverage.rs",
        "loading/",
        "mod.rs",
        "models.rs",
        "support/",
        "validation/",
        "vcf_emit.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        dir_entries(&root.join("src/compiler")),
        expected_compiler,
        "compiler module tree changed"
    );

    let expected_loading: BTreeSet<_> = [
        "image_registries.rs",
        "index_catalogs.rs",
        "index_defaults.rs",
        "load_and_collect.rs",
        "mod.rs",
        "stage_loading.rs",
        "stage_registries.rs",
        "tool_loading.rs",
        "tool_registries.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        dir_entries(&root.join("src/compiler/loading")),
        expected_loading,
        "loading/ must stay split by source family"
    );

    let expected_support: BTreeSet<_> =
        ["mod.rs", "placeholders.rs", "render.rs", "repository.rs", "status.rs", "tooling.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        dir_entries(&root.join("src/compiler/support")),
        expected_support,
        "support/ must stay limited to compiler helpers"
    );

    let expected_validation: BTreeSet<_> = [
        "catalog_coverage.rs",
        "catalog_validation.rs",
        "deprecations.rs",
        "fixture_consistency.rs",
        "index_rules/",
        "mod.rs",
        "stage_files.rs",
        "strict_stage_schemas.rs",
        "tool_files.rs",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(
        dir_entries(&root.join("src/compiler/validation")),
        expected_validation,
        "validation/ must stay split by validation family"
    );

    let expected_index_rules: BTreeSet<_> =
        ["compatibility_matrix.rs", "domain_inventory.rs", "domain_versions.rs", "mod.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        dir_entries(&root.join("src/compiler/validation/index_rules")),
        expected_index_rules,
        "index_rules/ must keep reference-index checks grouped"
    );
}

#[test]
fn markdown_docs_stay_in_root_readme_or_docs_dir() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut offenders = Vec::new();
    collect_markdown_outside_docs(root, root, &mut offenders);

    assert!(
        offenders.is_empty(),
        "crate markdown must be root README.md or live under docs/: {}",
        offenders.join(", ")
    );
}

fn dir_entries(path: &Path) -> BTreeSet<String> {
    std::fs::read_dir(path)
        .unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display())))
        .map(|entry| {
            let path = entry.path();
            let name = entry.file_name().to_string_lossy().to_string();
            if path.is_dir() {
                format!("{name}/")
            } else {
                name
            }
        })
        .collect()
}

fn collect_markdown_outside_docs(root: &Path, path: &Path, offenders: &mut Vec<String>) {
    for entry in
        std::fs::read_dir(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
    {
        let entry = entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", path.display()));
        let path = entry.path();
        let rel = path.strip_prefix(root).unwrap_or_else(|err| {
            panic!("strip {} from {}: {err}", root.display(), path.display())
        });
        let rel_text = rel.to_string_lossy().replace('\\', "/");

        if path.is_dir() {
            if rel_text != "docs" {
                collect_markdown_outside_docs(root, &path, offenders);
            }
            continue;
        }

        if path.extension().is_some_and(|extension| extension == "md")
            && rel_text != "README.md"
            && !rel_text.starts_with("docs/")
        {
            offenders.push(rel_text);
        }
    }
}
