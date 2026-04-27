use std::path::Path;

#[test]
fn stage_contract_docs_list_every_implemented_stage() {
    let docs = stage_contract_docs();

    for stage in bijux_dna_stages_bam::implemented_stages() {
        assert!(
            docs.contains(&format!("`{}`", stage.as_str())),
            "docs/STAGE_CONTRACTS.md must list stage `{}`",
            stage.as_str()
        );
    }
}

#[test]
fn stage_contract_docs_list_observer_fixture_files() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = stage_contract_docs();
    let fixture_root = root.join("tests/fixtures/observer/default");
    let fixtures = std::fs::read_dir(&fixture_root)
        .unwrap_or_else(|err| panic!("read {}: {err}", fixture_root.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read fixture entry: {err}")))
        .filter(|entry| entry.path().is_file())
        .map(|entry| entry.file_name().to_string_lossy().to_string());

    for fixture in fixtures {
        assert!(
            docs.contains(&fixture),
            "docs/STAGE_CONTRACTS.md must list observer fixture `{fixture}`"
        );
    }
}

fn stage_contract_docs() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(root.join("docs/STAGE_CONTRACTS.md"))
        .unwrap_or_else(|err| panic!("read docs/STAGE_CONTRACTS.md: {err}"))
}
