use std::fs;
use std::path::Path;

#[test]
fn dependency_graph_matches_science_boundary() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let cargo_toml = read(root.join("Cargo.toml"));
    let dependencies_doc = read(root.join("docs/DEPENDENCIES.md"));

    for dependency in
        ["anyhow", "bijux-dna-infra", "clap", "serde", "serde_json", "toml", "walkdir"]
    {
        assert!(
            cargo_toml.contains(dependency),
            "Cargo.toml must keep documented science dependency `{dependency}`"
        );
        assert!(
            dependencies_doc.contains(&format!("`{dependency}`")),
            "docs/DEPENDENCIES.md must document dependency `{dependency}`"
        );
    }

    assert!(
        dependencies_doc.contains("`bijux-dna-policies`"),
        "docs/DEPENDENCIES.md must document the guardrail dev-dependency"
    );
    assert!(
        dependencies_doc.contains("`thiserror`") && !cargo_toml.contains("thiserror"),
        "docs/DEPENDENCIES.md must explain why direct thiserror is absent"
    );

    for forbidden in [
        "bijux-dna-runtime",
        "bijux-dna-runner",
        "bijux-dna-engine",
        "bijux-dna-planner",
        "bijux-dna-environment",
    ] {
        assert!(
            !cargo_toml.contains(forbidden),
            "bijux-dna-science must not depend on forbidden execution/planning crate {forbidden}"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}
