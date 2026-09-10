//! Metadata fixtures must remain distinguishable from, and never overwrite, reference assets.
use anyhow::Result;
use bijux_dna_bench_corpus::panel_metadata::{
    materialize_local_vcf_panel_fixture, resolve_governed_vcf_panel_asset_contract,
};
use std::fs;
use std::path::Path;

#[test]
fn metadata_markers_are_reusable_without_changing_observed_bytes() -> Result<()> {
    let root = tempfile::tempdir()?;
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    assert_eq!(contract.fixture_class, "local_smoke_metadata_only");
    let first = materialize_local_vcf_panel_fixture(root.path())?;
    assert_eq!(first.fixture_class, "local_smoke");
    let snapshots =
        first.materialized_files.iter().map(fs::read).collect::<std::io::Result<Vec<_>>>()?;
    assert!(!snapshots.is_empty());
    assert!(snapshots.iter().all(|bytes| bytes.starts_with(b"fixture_class=local_smoke\n")));
    let second = materialize_local_vcf_panel_fixture(root.path())?;
    assert_eq!(first, second);
    for (path, before) in second.materialized_files.iter().zip(snapshots) {
        assert_eq!(fs::read(path)?, before);
    }
    Ok(())
}

#[test]
fn unknown_map_bytes_reject_before_any_panel_marker_is_written() -> Result<()> {
    let root = tempfile::tempdir()?;
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    let path =
        root.path().join("fixtures/maps").join(&contract.map_id).join(&contract.map_files[0]);
    fs::create_dir_all(path.parent().expect("map parent"))?;
    let native = b"chromosome position rate map\n22 10510000 1.2 0.1\n";
    fs::write(&path, native)?;
    assert!(materialize_local_vcf_panel_fixture(root.path()).is_err());
    assert_eq!(fs::read(&path)?, native);
    assert!(!root.path().join("fixtures/panels").exists());
    Ok(())
}

#[test]
fn stale_marker_bytes_are_preserved_and_rejected() -> Result<()> {
    let root = tempfile::tempdir()?;
    let report = materialize_local_vcf_panel_fixture(root.path())?;
    let victim = &report.materialized_files[0];
    let unknown = b"fixture_class=local_smoke\nasset_kind=foreign\n";
    fs::write(victim, unknown)?;
    let removed = &report.materialized_files[report.materialized_files.len() - 1];
    assert_ne!(victim, removed);
    fs::remove_file(removed)?;
    assert!(materialize_local_vcf_panel_fixture(root.path()).is_err());
    assert_eq!(fs::read(victim)?, unknown);
    assert!(!Path::new(removed).exists());
    Ok(())
}

#[test]
fn blocked_parent_rejects_without_partial_materialization() -> Result<()> {
    let root = tempfile::tempdir()?;
    fs::write(root.path().join("fixtures"), b"preserve unrelated file")?;
    assert!(materialize_local_vcf_panel_fixture(root.path()).is_err());
    assert_eq!(fs::read(root.path().join("fixtures"))?, b"preserve unrelated file");
    Ok(())
}

#[cfg(unix)]
#[test]
fn symbolic_link_destinations_cannot_redirect_panel_markers() -> Result<()> {
    let root = tempfile::tempdir()?;
    let external = tempfile::tempdir()?;
    std::os::unix::fs::symlink(external.path(), root.path().join("fixtures"))?;
    assert!(materialize_local_vcf_panel_fixture(root.path()).is_err());
    assert_eq!(fs::read_dir(external.path())?.count(), 0);
    Ok(())
}
