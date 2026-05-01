use std::collections::BTreeSet;
use std::path::Path;

#[test]
fn crate_tree_matches_domain_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let expected_root: BTreeSet<_> = ["Cargo.toml", "README.md", "docs/", "src/", "tests/"]
        .into_iter()
        .map(str::to_string)
        .collect();
    assert_eq!(dir_entries(root), expected_root, "domain-bam root must stay minimal");

    let expected_docs: BTreeSet<_> = [
        "ARCHITECTURE.md",
        "BOUNDARY.md",
        "COMMANDS.md",
        "DOMAIN_MODEL.md",
        "FIXTURES.md",
        "INDEX.md",
        "METRICS.md",
        "PUBLIC_API.md",
        "SCOPE.md",
        "TESTS.md",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    let docs_entries = dir_entries(&root.join("docs"));
    assert_eq!(docs_entries.len(), 10, "domain-bam docs allowance is 10 Markdown files");
    assert_eq!(docs_entries, expected_docs, "domain-bam docs spine must stay explicit");

    let expected_src: BTreeSet<_> = [
        "alignment.rs",
        "artifacts.rs",
        "defaults.rs",
        "invariants/",
        "lib.rs",
        "metrics/",
        "params/",
        "pipeline_contract.rs",
        "prelude.rs",
        "stage_specs/",
        "types/",
    ]
    .into_iter()
    .map(str::to_string)
    .collect();
    assert_eq!(dir_entries(&root.join("src")), expected_src, "source tree changed");

    let expected_boundaries: BTreeSet<_> =
        ["architecture.rs", "commands.rs", "dependencies.rs", "guardrails.rs", "purity.rs"]
            .into_iter()
            .map(str::to_string)
            .collect();
    assert_eq!(
        dir_entries(&root.join("tests/boundaries")),
        expected_boundaries,
        "boundary tests must stay focused on architecture, guardrails, and purity"
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
