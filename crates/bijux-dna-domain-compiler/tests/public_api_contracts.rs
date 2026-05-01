use std::path::Path;

#[test]
fn public_api_docs_cover_registry_bundle_exports() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let public_api = std::fs::read_to_string(root.join("docs/PUBLIC_API.md"))
        .unwrap_or_else(|err| panic!("read docs/PUBLIC_API.md: {err}"));

    for expected in [
        "build_domain_registry_bundle",
        "load_domain_registry_bundle",
        "write_domain_registry_bundle",
        "query_domain_registry_bundle",
        "domain_defaults_snapshot",
        "domain_artifact_contract_snapshots",
        "domain_metric_catalogs",
        "domain_deprecation_catalogs",
        "domain_invariant_catalogs",
        "domain_evidence_catalogs",
        "DomainRegistryReleaseBundle",
        "DomainRegistryQuery",
        "DomainRegistryQueryKind",
        "CompiledDomainRegistry",
    ] {
        assert!(public_api.contains(expected), "docs/PUBLIC_API.md must document `{expected}`");
    }
}

#[test]
fn readme_and_contracts_list_registry_bundle_outputs() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = std::fs::read_to_string(root.join("README.md"))
        .unwrap_or_else(|err| panic!("read README.md: {err}"));
    let contracts = std::fs::read_to_string(root.join("docs/CONTRACTS.md"))
        .unwrap_or_else(|err| panic!("read docs/CONTRACTS.md: {err}"));

    for expected in [
        "ci/registry/domain_registry_release_bundle.json",
        "ci/registry/domain_defaults_snapshot.json",
        "ci/registry/domain_artifact_contract_snapshots.json",
        "ci/registry/domain_metric_catalogs.json",
        "ci/registry/domain_deprecations_snapshot.json",
        "ci/registry/domain_invariant_catalogs.json",
        "ci/registry/domain_evidence_contracts.json",
    ] {
        assert!(readme.contains(expected), "README.md must document `{expected}`");
        assert!(contracts.contains(expected), "docs/CONTRACTS.md must document `{expected}`");
    }
}
