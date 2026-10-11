//! Owner: one byte-bound container catalog observation for request-scoped admission.
use super::{ContainerCatalog, CONTAINER_CATALOG_SCHEMA_V1};
use anyhow::{bail, Context, Result};
use std::{fs, path::Path};
pub(crate) struct CatalogObservation {
    pub(crate) catalog: ContainerCatalog,
    pub(crate) sha256: String,
}
pub(crate) fn observe_container_catalog(root: &Path) -> Result<CatalogObservation> {
    let (catalog, bytes) = load_catalog_source(root)?;
    Ok(CatalogObservation { catalog, sha256: bijux_dna_infra::hash_bytes_sha256(&bytes) })
}
pub(super) fn load_catalog_source(root: &Path) -> Result<(ContainerCatalog, Vec<u8>)> {
    let path = root.join("containers/contracts/catalog.toml");
    let bytes =
        fs::read(&path).with_context(|| format!("read container catalog {}", path.display()))?;
    let catalog: ContainerCatalog = toml::from_slice(&bytes)
        .with_context(|| format!("parse container catalog {}", path.display()))?;
    if catalog.schema_version != CONTAINER_CATALOG_SCHEMA_V1 {
        bail!("unsupported container catalog schema: {}", catalog.schema_version);
    }
    Ok((catalog, bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn catalog_observation_binds_bytes_and_refreshes_after_replacement() -> Result<()> {
        let temp = bijux_dna_testkit::temp::tempdir_for("container-catalog-observation");
        let path = temp.path().join("containers/contracts/catalog.toml");
        fs::create_dir_all(path.parent().context("catalog parent")?)?;
        let catalog: ContainerCatalog = serde_json::from_value(
            serde_json::json!({"schema_version":CONTAINER_CATALOG_SCHEMA_V1,"owner":"owner","contact":"first","repository":"repo","apptainer_architecture":"amd64","docker_architecture":"amd64","evidence_schema":"evidence","recipe_license":"MIT","required_checks":[],"evidence_dimensions":[],"tools":[]}),
        )?;
        let original = toml::to_string(&catalog)?;
        fs::write(&path, &original)?;
        let observed = observe_container_catalog(temp.path())?;
        let mut changed = catalog.clone();
        changed.contact = "second".into();
        let replacement = toml::to_string(&changed)?;
        fs::write(&path, &replacement)?;
        assert_eq!(observed.catalog, catalog);
        assert_eq!(observed.sha256, bijux_dna_infra::hash_bytes_sha256(original.as_bytes()));
        let fresh = observe_container_catalog(temp.path())?;
        assert_eq!(fresh.catalog, changed);
        assert_eq!(fresh.sha256, bijux_dna_infra::hash_bytes_sha256(replacement.as_bytes()));
        assert_ne!(observed.sha256, fresh.sha256);
        fs::write(&path, replacement.replace(CONTAINER_CATALOG_SCHEMA_V1, "unknown.schema"))?;
        assert!(observe_container_catalog(temp.path()).is_err());
        fs::remove_file(&path)?;
        assert!(observe_container_catalog(temp.path()).is_err());
        Ok(())
    }
}
