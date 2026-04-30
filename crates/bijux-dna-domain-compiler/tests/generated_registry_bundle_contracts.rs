use bijux_dna_domain_compiler::{
    load_domain_registry_bundle, CompiledDomainDefaultsSnapshot, DomainDeprecationCatalog,
};

#[path = "support/mod.rs"]
mod support;

#[test]
fn generated_registry_release_bundle_is_checked_in_and_loadable() -> anyhow::Result<()> {
    let root = support::repo_root();
    let bundle = load_domain_registry_bundle(
        &root.join("configs/ci/registry/domain_registry_release_bundle.json"),
    )?;
    assert!(
        bundle
            .domains
            .iter()
            .any(|domain| domain.domain_id == "fastq" && !domain.stages.is_empty()),
        "checked-in release bundle must expose the FASTQ registry surface"
    );
    Ok(())
}

#[test]
fn generated_defaults_snapshot_carries_governance_metadata() -> anyhow::Result<()> {
    let root = support::repo_root();
    let raw = std::fs::read_to_string(root.join("configs/ci/registry/domain_defaults_snapshot.json"))?;
    let defaults: Vec<CompiledDomainDefaultsSnapshot> = serde_json::from_str(&raw)?;
    assert!(
        defaults.iter().any(|domain| {
            domain.domain_id == "vcf"
                && domain.defaults.iter().any(|entry| {
                    entry.stage_id == "vcf.call"
                        && !entry.source.trim().is_empty()
                        && !entry.rationale.trim().is_empty()
                        && !entry.governance_status.trim().is_empty()
                        && !entry.override_policy.trim().is_empty()
                })
        }),
        "checked-in defaults snapshot must preserve non-anonymous VCF default governance"
    );
    Ok(())
}

#[test]
fn generated_deprecations_snapshot_tracks_known_records() -> anyhow::Result<()> {
    let root = support::repo_root();
    let raw =
        std::fs::read_to_string(root.join("configs/ci/registry/domain_deprecations_snapshot.json"))?;
    let deprecations: Vec<DomainDeprecationCatalog> = serde_json::from_str(&raw)?;
    assert!(
        deprecations.iter().any(|domain| {
            domain.domain_id == "bam"
                && domain.deprecations.iter().any(|entry| entry.tool_id.as_deref() == Some("bamtools"))
        }),
        "checked-in deprecations snapshot must preserve known BAM deprecation records"
    );
    Ok(())
}
