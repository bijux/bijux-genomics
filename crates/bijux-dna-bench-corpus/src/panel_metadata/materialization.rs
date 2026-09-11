//! Materialize only admitted metadata markers; never replace reference asset bytes.
use super::admission::resolve_governed_vcf_panel_asset_contract;
use super::markers::{admit_markers, plan_markers, Marker};
use super::model::LocalVcfPanelFixtureReport;
use anyhow::Result;
use std::path::Path;

/// Materialize metadata-only panel/map fixtures, reusing byte-identical markers.
///
/// # Errors
/// Rejects unsafe paths and existing non-marker assets before writing any marker.
/// Concurrent byte-identical markers are reused after revalidation.
/// Publication does not replace files and is not a multi-file transaction.
pub fn materialize_local_vcf_panel_fixture(
    materialization_root: &Path,
) -> Result<LocalVcfPanelFixtureReport> {
    let contract = resolve_governed_vcf_panel_asset_contract()?;
    let markers = plan_markers(materialization_root, &contract)?;
    let absent = admit_markers(&markers)?;
    for marker in absent {
        publish_marker(marker)?;
    }
    Ok(LocalVcfPanelFixtureReport {
        schema_version: "bijux.local_vcf_panel_fixture.v1",
        fixture_class: "local_smoke",
        panel_id: contract.panel_id,
        map_id: contract.map_id,
        materialized_files: markers
            .iter()
            .map(|marker| marker.path.display().to_string())
            .collect(),
    })
}

fn publish_marker(marker: &Marker) -> Result<()> {
    match bijux_dna_infra::atomic_create_bytes(&marker.path, marker.payload.as_bytes()) {
        Ok(()) => Ok(()),
        Err(error) => {
            if error
                .source
                .as_ref()
                .is_none_or(|source| source.kind() != std::io::ErrorKind::AlreadyExists)
            {
                return Err(error.into());
            }
            // Another process may publish after preflight. Revalidate its ancestry,
            // regular-file identity and exact marker bytes before accepting the result.
            let absent = admit_markers(std::slice::from_ref(marker))?;
            if absent.is_empty() {
                Ok(())
            } else {
                Err(error.into())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{admit_markers, publish_marker, Marker};
    use anyhow::{anyhow, Result};
    use std::sync::Barrier;

    #[test]
    fn concurrent_marker_creation_reuses_exact_bytes_after_shared_preflight() -> Result<()> {
        let root = tempfile::tempdir()?;
        let marker = Marker {
            path: root.path().join("marker"),
            payload: "fixture_class=local_smoke\n".to_owned(),
        };
        let start = Barrier::new(4);
        std::thread::scope(|scope| -> Result<()> {
            let handles = (0..4)
                .map(|_| {
                    scope.spawn(|| -> Result<()> {
                        assert_eq!(admit_markers(std::slice::from_ref(&marker))?.len(), 1);
                        start.wait();
                        publish_marker(&marker)
                    })
                })
                .collect::<Vec<_>>();
            for handle in handles {
                handle.join().map_err(|_| anyhow!("marker publisher panicked"))??;
            }
            Ok(())
        })?;
        assert_eq!(std::fs::read(&marker.path)?, marker.payload.as_bytes());
        Ok(())
    }

    #[test]
    fn foreign_bytes_published_after_preflight_are_preserved_and_refused() -> Result<()> {
        let root = tempfile::tempdir()?;
        let marker =
            Marker { path: root.path().join("marker"), payload: "expected marker".to_owned() };
        assert_eq!(admit_markers(std::slice::from_ref(&marker))?.len(), 1);
        std::fs::write(&marker.path, b"foreign payload")?;
        assert!(publish_marker(&marker).is_err());
        assert_eq!(std::fs::read(&marker.path)?, b"foreign payload");
        Ok(())
    }
}
