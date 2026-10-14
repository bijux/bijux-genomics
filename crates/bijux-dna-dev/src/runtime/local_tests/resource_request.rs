//! Load an explicit artifact-owned allocation bound to the current source and lane catalog.

use std::path::Path;

use anyhow::{ensure, Context, Result};
use serde_json::Value;

use crate::application::ci::local_tests::{
    admit_resources, AdmittedResources, ArtifactReference, ProvisionedResourceRequest,
    ResourceCapacityObservation, SourceIdentity,
};
use crate::runtime::workspace::Workspace;

use super::reference;

pub(crate) fn read_resource_request(
    workspace: &Workspace,
    path: &Path,
    source: &SourceIdentity,
    observed: &Value,
    worker_budget: usize,
) -> Result<(AdmittedResources, ArtifactReference)> {
    let path = workspace.admit_artifact_path(path)?;
    ensure!(
        path.symlink_metadata()?.is_file() && path.canonicalize()? == path,
        "resource request must be a canonical regular artifact input"
    );
    let seal = reference(&path)?;
    let bytes = bijux_dna_infra::read_to_end_bounded(&path, 256 * 1024)?;
    let request: ProvisionedResourceRequest = serde_json::from_slice(&bytes)?;
    ensure!(
        bijux_dna_infra::hash_bytes_sha256(&bytes) == seal.sha256,
        "resource request changed during admission"
    );
    let capacity = ResourceCapacityObservation {
        physical_ram_bytes: observed["physical_ram_bytes"]
            .as_u64()
            .context("physical RAM observation missing")?,
        kernel_reported_available_memory_bytes: observed["kernel_reported_available_memory_bytes"]
            .as_u64()
            .context("available RAM observation missing")?,
        artifact_volume_free_bytes: observed["artifact_volume_free_bytes"]
            .as_u64()
            .context("free disk observation missing")?,
        cpu_worker_units: worker_budget,
    };
    let catalog =
        bijux_dna_infra::hash_file_sha256(&workspace.path("configs/ci/local-test-lanes.json"))?;
    Ok((admit_resources(request, source, &catalog, &capacity)?, seal))
}

/// Verify the original provisioned input after producers complete, without replacing its seal.
pub(crate) fn verify_resource_request(
    workspace: &Workspace,
    seal: &ArtifactReference,
) -> Result<()> {
    let path = workspace.admit_artifact_path(Path::new(&seal.path))?;
    ensure!(
        path.is_absolute() && path.symlink_metadata()?.is_file() && path.canonicalize()? == path,
        "resource request ownership changed during execution"
    );
    ensure!(reference(&path)?.sha256 == seal.sha256, "resource request changed during execution");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::application::ci::local_tests::REQUIRED_LANES;
    use serde_json::json;

    #[test]
    fn provisioned_input_mutation_cannot_inherit_prior_admission() -> Result<()> {
        let owner = tempfile::tempdir()?;
        let workspace = Workspace { root: owner.path().canonicalize()? };
        std::fs::create_dir_all(workspace.path("artifacts"))?;
        std::fs::create_dir_all(workspace.path("configs/ci"))?;
        std::fs::write(workspace.path("configs/ci/local-test-lanes.json"), b"catalog inputs")?;
        let source = SourceIdentity {
            sha: "a".repeat(40),
            tree: "b".repeat(40),
            patch_sha256: "c".repeat(64),
        };
        let catalog =
            bijux_dna_infra::hash_file_sha256(&workspace.path("configs/ci/local-test-lanes.json"))?;
        let request = json!({
            "schema_version":"bijux.ci.provisioned_resources.v1","source":source,
            "lane_catalog_sha256":catalog,"authority":"explicit constrained runner allocation","measurement_claim":false,
            "cpu_worker_units":8,"memory_capacity_bytes":1200,"artifact_disk_capacity_bytes":6000,
            "foreground_memory_reserve_bytes":200,"foreground_disk_reserve_bytes":500,"retained_growth_reserve_bytes":2000,
            "io_slots":5,"top_level_process_slots":6,"coordinator":{"memory_bytes":100,"scratch_bytes":250,"io_slots":1},
            "lanes":REQUIRED_LANES.into_iter().map(|lane|json!({"lane":lane,"allocation":{"memory_bytes":800,"scratch_bytes":500,"io_slots":1}})).collect::<Vec<_>>()
        });
        let path = workspace.path("artifacts/provisioned-resources.json");
        std::fs::write(&path, serde_json::to_vec(&request)?)?;
        let observed = json!({"physical_ram_bytes":3200,"kernel_reported_available_memory_bytes":1800,"artifact_volume_free_bytes":9000});
        let (admitted, seal) = read_resource_request(&workspace, &path, &source, &observed, 11)?;
        assert_eq!(admitted.worker_scratch_bytes, 3750);
        verify_resource_request(&workspace, &seal)?;
        let mut changed = request.clone();
        changed["cpu_worker_units"] = json!(9);
        std::fs::write(&path, serde_json::to_vec(&changed)?)?;
        assert!(verify_resource_request(&workspace, &seal).is_err());
        std::fs::write(
            workspace.path("configs/ci/local-test-lanes.json"),
            b"changed actual producer configuration",
        )?;
        assert!(read_resource_request(&workspace, &path, &source, &observed, 11).is_err());
        assert!(read_resource_request(&workspace, &path, &source, &json!({}), 11).is_err());
        Ok(())
    }

    #[cfg(unix)]
    #[test]
    fn redirected_provisioned_input_cannot_inherit_identical_bytes() -> Result<()> {
        use std::os::unix::fs::symlink;
        let owner = tempfile::tempdir()?;
        let workspace = Workspace { root: owner.path().canonicalize()? };
        std::fs::create_dir_all(workspace.path("artifacts"))?;
        let original = workspace.path("artifacts/request.json");
        let alternate = workspace.path("artifacts/alternate.json");
        std::fs::write(&original, b"identical declared provision")?;
        std::fs::write(&alternate, b"identical declared provision")?;
        let seal = reference(&original)?;
        verify_resource_request(&workspace, &seal)?;
        std::fs::remove_file(&original)?;
        symlink(&alternate, &original)?;
        assert_eq!(reference(&original)?.sha256, seal.sha256);
        assert!(verify_resource_request(&workspace, &seal).is_err());
        Ok(())
    }
}
