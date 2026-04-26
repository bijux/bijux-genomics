use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

#[test]
fn public_api_doc_matches_science_root_surface() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = read(root.join("docs/PUBLIC_API.md"));
    let lib_rs = read(root.join("src/lib.rs"));

    let modules = lib_rs
        .lines()
        .filter_map(|line| line.trim().strip_prefix("pub mod "))
        .map(|module| module.trim_end_matches(';').to_string())
        .collect::<BTreeSet<_>>();

    assert_eq!(
        modules,
        entries(
            ["app", "cli", "compile", "domain", "errors", "io", "release", "render", "schema",]
        ),
        "src/lib.rs must expose the documented science modules"
    );

    for module in &modules {
        assert!(
            public_api.contains(&format!("`{module}`")),
            "docs/PUBLIC_API.md must document public module `{module}`"
        );
    }

    for entrypoint in [
        "app::run",
        "app::validate_workspace",
        "app::build_workspace",
        "app::trace_workspace",
        "app::release_workspace",
        "compile::load_specs",
        "compile::compile_workspace",
        "compile::compile_loaded",
        "release::cut_release",
    ] {
        assert!(
            public_api.contains(entrypoint),
            "docs/PUBLIC_API.md must document stable entrypoint `{entrypoint}`"
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|err| panic!("read {}: {err}", path.display()))
}

fn entries<const N: usize>(items: [&str; N]) -> BTreeSet<String> {
    items.into_iter().map(str::to_string).collect()
}
