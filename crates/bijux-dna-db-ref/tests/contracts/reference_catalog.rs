use anyhow::{Context, Result};
use bijux_dna_db_ref::{find_reference_bundle_identity_at, find_species_authority_at};

fn repository(build: &str) -> Result<tempfile::TempDir> {
    let root = tempfile::tempdir()?;
    let runtime = root.path().join("configs/runtime");
    std::fs::create_dir_all(&runtime)?;
    std::fs::write(
        runtime.join("species.toml"),
        format!(
            "[[species]]\nspecies_id = 'fixture'\ndefault_build_id = '{build}'\ncontig_naming = 'bare'\nsex_chromosomes = 'XY'\nmitochondrion_id = 'MT'\nploidy_model = 'diploid_xy'\n"
        ),
    )?;
    std::fs::write(
        runtime.join("reference_bundles.toml"),
        format!(
            "[[bundle]]\nbundle_id = 'bundle'\nspecies_id = 'fixture'\nbuild_id = '{build}'\nfasta = 'ref.fa'\nfai = 'ref.fa.fai'\ndict = 'ref.dict'\ncontig_set_digest = 'metadata-only'\nsource_lock_sha256 = 'metadata-only'\nbundle_lock_sha256 = 'metadata-only'\nnormalization_policy = 'strict_only'\nsex_system = 'xy'\npar_policy = 'none'\ncontigs = [{{name = '1', length_bp = 10}}]\n"
        ),
    )?;
    Ok(root)
}

#[test]
fn explicit_roots_resolve_their_own_species_and_bundle_identities() -> Result<()> {
    for build in ["alpha", "beta"] {
        let root = repository(build)?;
        let species = find_species_authority_at(root.path(), "fixture")?.context("species")?;
        let bundle = find_reference_bundle_identity_at(root.path(), "bundle")?.context("bundle")?;
        assert_eq!(species.default_build_id, build);
        assert_eq!(species.contig_naming, "bare");
        assert_eq!(bundle.build_id, build);
        assert_eq!(bundle.species_id, species.species_id);
        assert_eq!(bundle.bundle_id, "bundle");
        assert!(find_species_authority_at(root.path(), "unknown")?.is_none());
        assert!(find_reference_bundle_identity_at(root.path(), "unknown")?.is_none());
    }
    Ok(())
}

#[test]
fn missing_selected_catalogs_do_not_fall_back_to_the_compiled_workspace() -> Result<()> {
    let root = tempfile::tempdir()?;
    assert!(find_species_authority_at(root.path(), "Homo sapiens").is_err());
    assert!(find_reference_bundle_identity_at(root.path(), "hsapiens_grch38_primary").is_err());
    assert!(!root.path().join("configs").exists(), "queries must not create catalog files");
    Ok(())
}

#[test]
fn identity_queries_consume_complete_owner_schemas() -> Result<()> {
    let root = repository("alpha")?;
    std::fs::write(
        root.path().join("configs/runtime/species.toml"),
        "[[species]]\nspecies_id = 'fixture'\ndefault_build_id = 'alpha'\n",
    )?;
    assert!(find_species_authority_at(root.path(), "fixture").is_err());
    std::fs::write(
        root.path().join("configs/runtime/reference_bundles.toml"),
        "[[bundle]]\nbundle_id = 'bundle'\nspecies_id = 'fixture'\nbuild_id = 'alpha'\n",
    )?;
    assert!(find_reference_bundle_identity_at(root.path(), "bundle").is_err());
    Ok(())
}

#[test]
fn matching_duplicate_catalog_identities_are_rejected() -> Result<()> {
    let root = repository("alpha")?;
    for filename in ["species.toml", "reference_bundles.toml"] {
        let path = root.path().join("configs/runtime").join(filename);
        let original = std::fs::read_to_string(&path)?;
        std::fs::write(&path, format!("{original}\n{original}"))?;
    }
    let species = find_species_authority_at(root.path(), "fixture")
        .err()
        .context("duplicate species must fail")?;
    let bundle = find_reference_bundle_identity_at(root.path(), "bundle")
        .err()
        .context("duplicate bundle must fail")?;
    assert!(species.to_string().contains("identity `fixture` is ambiguous"));
    assert!(bundle.to_string().contains("identity `bundle` is ambiguous"));
    Ok(())
}
