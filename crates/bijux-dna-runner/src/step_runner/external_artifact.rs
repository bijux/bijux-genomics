use std::path::Path;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use bijux_dna_containers::api::{
    bind_external_artifact_command, external_artifact_tool_id, verify_external_artifact_admission,
    ExternalArtifactKind, RuntimeKind,
};
use bijux_dna_core::contract::ExecutionStep;
use bijux_dna_core::prelude::CommandSpecV1;

use super::{execute_verified_step_with_runtime_executable, StageResultV1};

/// Execute a restricted-tool step only after reverifying its operator-provided artifact.
///
/// # Errors
/// Returns an error before stage execution when the plan reference, admission record, artifact
/// identity, catalog identity, stage binding, or command binding is invalid.
pub fn execute_external_artifact_step(
    step: &ExecutionStep,
    repository_root: &Path,
    distributable_image_root: &Path,
    admission_path: &Path,
    apptainer_executable: Option<&Path>,
    timeout: Option<Duration>,
) -> Result<StageResultV1> {
    let planned_tool = external_artifact_tool_id(&step.image.image).with_context(|| {
        format!("step {} does not declare an external-artifact image reference", step.step_id)
    })?;
    let admission = verify_external_artifact_admission(
        repository_root,
        distributable_image_root,
        admission_path,
    )?;
    if admission.tool_id != planned_tool {
        bail!(
            "external artifact admission for {} cannot execute plan for {planned_tool}",
            admission.tool_id
        );
    }
    let binding = bind_external_artifact_command(
        repository_root,
        distributable_image_root,
        admission_path,
        step.stage_id.as_str(),
        &step.command.template,
        apptainer_executable,
    )?;
    let artifact = admission.artifact.as_ref().context("external artifact is not admitted")?;

    let mut bound = step.clone();
    bound.image.digest = Some(format!("sha256:{}", artifact.sha256));
    let runtime = match artifact.kind {
        ExternalArtifactKind::Binary => {
            let mut command = vec![binding.executable];
            command.extend(binding.arguments);
            bound.command = CommandSpecV1 { template: command };
            bound.image.image = format!("external-binary://{}", admission.tool_id);
            RuntimeKind::Local
        }
        ExternalArtifactKind::ApptainerSif => {
            bound.image.image.clone_from(&artifact.path);
            RuntimeKind::Apptainer
        }
    };
    let runtime_executable =
        (runtime == RuntimeKind::Apptainer).then_some(apptainer_executable).flatten();
    execute_verified_step_with_runtime_executable(&bound, runtime, timeout, runtime_executable)
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt as _;
    use std::path::PathBuf;

    use bijux_dna_containers::api::{
        admit_external_artifact, load_external_artifact_admission, repository_head_commit,
        ExternalArtifactAdmissionRequest, ExternalArtifactInput,
    };
    use serde_json::json;

    use super::*;

    fn repository_root() -> PathBuf {
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(Path::parent)
            .unwrap_or_else(|| panic!("workspace root"))
            .to_path_buf()
    }

    fn angsd_step(out_dir: &Path, stage_id: &str) -> Result<ExecutionStep> {
        Ok(serde_json::from_value(json!({
            "step_id": "sample-0001.external.angsd",
            "stage_id": stage_id,
            "command": { "template": ["angsd", "-i", "sample.input"] },
            "image": { "image": "external-artifact://angsd", "digest": null },
            "resources": { "runtime": "apptainer", "mem_gb": 1, "tmp_gb": 1, "threads": 1 },
            "io": { "inputs": [], "outputs": [] },
            "out_dir": out_dir,
            "aux_images": {},
            "expected_artifact_ids": [],
            "metrics_schema_ids": []
        }))?)
    }

    #[test]
    fn admitted_sif_uses_the_explicit_apptainer_executable() -> Result<()> {
        let repository_root = repository_root();
        let temporary =
            tempfile::Builder::new().prefix("bijux-dna-runner-external-").tempdir_in("/tmp")?;
        let active_root = temporary.path().join("active");
        let artifact = temporary.path().join("angsd.sif");
        let apptainer = temporary.path().join("operator-apptainer");
        let marker = temporary.path().join("apptainer-invoked");
        let admission_path = temporary.path().join("admission.json");
        std::fs::create_dir(&active_root)?;
        std::fs::write(&artifact, b"operator-provided-sif")?;
        std::fs::write(
            &apptainer,
            format!(
                "#!/bin/sh\nprintf 'invoked\\n' > '{}'\ncase \" $* \" in *' --version '*) printf 'angsd version 0.941\\n';; esac\nexit 0\n",
                marker.display()
            ),
        )?;
        std::fs::set_permissions(&apptainer, std::fs::Permissions::from_mode(0o755))?;
        let checksum = bijux_dna_infra::hash_file_sha256(&artifact)?;
        let commit = repository_head_commit(&repository_root)?;
        admit_external_artifact(
            &ExternalArtifactAdmissionRequest {
                repository_root: &repository_root,
                distributable_image_root: &active_root,
                repository_commit: &commit,
                tool_id: "angsd",
                recorded_at_utc: "2026-08-06T12:00:00Z",
                artifact: Some(ExternalArtifactInput {
                    kind: ExternalArtifactKind::ApptainerSif,
                    path: &artifact,
                    expected_sha256: &checksum,
                    lawful_access_basis: "institutional operator installation",
                    apptainer_executable: Some(&apptainer),
                }),
                blocker_reason: None,
            },
            &admission_path,
        )?;
        let admission = load_external_artifact_admission(&admission_path)?;
        let stage_id = admission.stage_ids.first().context("ANGSD admission has no stage")?;
        std::fs::remove_file(&marker)?;

        let step = angsd_step(&temporary.path().join("out"), stage_id)?;

        execute_external_artifact_step(
            &step,
            &repository_root,
            &active_root,
            &admission_path,
            Some(&apptainer),
            None,
        )?;

        assert!(marker.is_file());
        Ok(())
    }

    #[test]
    fn blocked_external_access_fails_before_output_effects() -> Result<()> {
        let repository_root = repository_root();
        let temporary = tempfile::tempdir()?;
        let active_root = temporary.path().join("active");
        let admission_path = temporary.path().join("blocked.json");
        let out_dir = temporary.path().join("out");
        std::fs::create_dir(&active_root)?;
        let commit = repository_head_commit(&repository_root)?;
        admit_external_artifact(
            &ExternalArtifactAdmissionRequest {
                repository_root: &repository_root,
                distributable_image_root: &active_root,
                repository_commit: &commit,
                tool_id: "angsd",
                recorded_at_utc: "2026-08-06T12:00:00Z",
                artifact: None,
                blocker_reason: Some("institutional artifact was not supplied"),
            },
            &admission_path,
        )?;
        let admission = load_external_artifact_admission(&admission_path)?;
        let stage_id = admission.stage_ids.first().context("ANGSD admission has no stage")?;

        let error = execute_external_artifact_step(
            &angsd_step(&out_dir, stage_id)?,
            &repository_root,
            &active_root,
            &admission_path,
            None,
            None,
        )
        .err()
        .context("blocked external access allowed execution")?;

        assert!(error.to_string().contains("institutional artifact was not supplied"));
        assert!(!out_dir.exists());
        Ok(())
    }
}
