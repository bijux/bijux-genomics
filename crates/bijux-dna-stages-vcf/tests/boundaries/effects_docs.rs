use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

#[test]
fn production_environment_overrides_are_documented() {
    let root = crate_root();
    let source_vars = env_vars_in_source(&root.join("src"));
    let effects = std::fs::read_to_string(root.join("docs/EFFECTS.md"))
        .unwrap_or_else(|err| panic!("read docs/EFFECTS.md: {err}"));

    for var in &source_vars {
        assert!(
            effects.contains(var),
            "docs/EFFECTS.md must document production environment override {var}"
        );
    }

    assert!(!source_vars.is_empty(), "stages-vcf should have explicit env override coverage");
}

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn env_vars_in_source(root: &Path) -> BTreeSet<String> {
    let mut vars = BTreeSet::new();
    collect_env_vars(root, &mut vars);
    vars
}

fn collect_env_vars(current: &Path, vars: &mut BTreeSet<String>) {
    for entry in
        std::fs::read_dir(current).unwrap_or_else(|err| panic!("read {}: {err}", current.display()))
    {
        let entry =
            entry.unwrap_or_else(|err| panic!("read entry in {}: {err}", current.display()));
        let path = entry.path();

        if path.is_dir() {
            collect_env_vars(&path, vars);
            continue;
        }

        if path.extension().and_then(|ext| ext.to_str()) != Some("rs") {
            continue;
        }

        let source = std::fs::read_to_string(&path)
            .unwrap_or_else(|err| panic!("read {}: {err}", path.display()));
        for token in source.split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '_')) {
            if token.starts_with("BIJUX_") {
                vars.insert(token.to_string());
            }
        }
    }
}
