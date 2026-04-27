use std::path::Path;

#[test]
fn stage_contract_docs_list_every_contract_stage() {
    let docs = stage_contract_docs();

    for stage in bijux_dna_stages_fastq::contract_stage_ids() {
        assert!(
            docs.contains(&format!("`{}`", stage.as_str())),
            "docs/STAGE_CONTRACTS.md must list stage `{}`",
            stage.as_str()
        );
    }
}

#[test]
fn stage_contract_docs_list_fixture_families() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let docs = stage_contract_docs();
    let fixture_root = root.join("tests/fixtures");
    let fixture_families = std::fs::read_dir(&fixture_root)
        .unwrap_or_else(|err| panic!("read {}: {err}", fixture_root.display()))
        .map(|entry| entry.unwrap_or_else(|err| panic!("read fixture entry: {err}")))
        .filter(|entry| entry.path().is_dir())
        .map(|entry| entry.file_name().to_string_lossy().to_string());

    for family in fixture_families {
        assert!(
            docs.contains(&format!("tests/fixtures/{family}")),
            "docs/STAGE_CONTRACTS.md must list fixture family `{family}`"
        );
    }
}

fn stage_contract_docs() -> String {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    match std::fs::read_to_string(root.join("docs/STAGE_CONTRACTS.md")) {
        Ok(docs) => docs,
        Err(error) => panic!("read docs/STAGE_CONTRACTS.md: {error}"),
    }
}
