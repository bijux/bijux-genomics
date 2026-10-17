use super::*;
use bijux_dna_containers::api::{
    admit_external_artifact_with_probe, ExternalArtifactProbeObservation,
    ExternalArtifactProbeOutcome, ExternalArtifactProbeRequest, ExternalArtifactProbeTransport,
};

#[derive(Clone, Copy, Debug)]
enum Drift {
    Unchanged,
    SameSizeContent,
    Size,
    Missing,
    Directory,
    Redirected,
}

struct Observer {
    drift: Drift,
    artifact: std::path::PathBuf,
    calls: usize,
}

impl ExternalArtifactProbeTransport for Observer {
    fn observe(
        &mut self,
        request: &ExternalArtifactProbeRequest<'_>,
    ) -> Result<ExternalArtifactProbeObservation> {
        self.calls += 1;
        assert!(request.command.iter().any(|arg| Path::new(arg) == self.artifact));
        match self.drift {
            Drift::Unchanged => {}
            Drift::SameSizeContent => std::fs::write(&self.artifact, b"changed")?,
            Drift::Size => std::fs::write(&self.artifact, b"changed size")?,
            Drift::Missing => std::fs::remove_file(&self.artifact)?,
            Drift::Directory => {
                std::fs::remove_file(&self.artifact)?;
                std::fs::create_dir(&self.artifact)?;
            }
            Drift::Redirected => {
                let retained = self.artifact.with_extension("retained");
                std::fs::rename(&self.artifact, &retained)?;
                std::os::unix::fs::symlink(retained, &self.artifact)?;
            }
        }
        Ok(ExternalArtifactProbeObservation {
            outcome: ExternalArtifactProbeOutcome::Exited { code: Some(0) },
            stdout: b"angsd version 0.941\n".to_vec(),
            stderr: Vec::new(),
        })
    }
}

#[test]
fn version_evidence_is_published_only_for_an_unchanged_external_artifact() -> Result<()> {
    let (fixture, repository) = external_artifact_fixture()?;
    let root = repository.as_path();
    let commit = repository_head_commit(root)?;
    for kind in [ExternalArtifactKind::Binary, ExternalArtifactKind::ApptainerSif] {
        for drift in [
            Drift::Unchanged,
            Drift::SameSizeContent,
            Drift::Size,
            Drift::Missing,
            Drift::Directory,
            Drift::Redirected,
        ] {
            let scratch = tempfile::Builder::new()
                .prefix("version-observation-")
                .tempdir_in(fixture.path())?;
            let artifact = scratch.path().join("operator artifact");
            std::fs::write(&artifact, b"initial")?;
            let artifact = artifact.canonicalize()?;
            let checksum = bijux_dna_infra::hash_file_sha256(&artifact)?;
            let active = scratch.path().join("active");
            std::fs::create_dir(&active)?;
            let output = scratch.path().join("admission.json");
            let request = ExternalArtifactAdmissionRequest {
                repository_root: root,
                distributable_image_root: &active,
                repository_commit: &commit,
                tool_id: "angsd",
                recorded_at_utc: "2026-08-06T12:00:00Z",
                availability: ExternalArtifactAdmissionStatus::Admitted,
                artifact: Some(ExternalArtifactInput {
                    kind,
                    path: &artifact,
                    expected_sha256: &checksum,
                    lawful_access_basis: "local contract fixture",
                    apptainer_executable: None,
                }),
                blocker_reason: None,
            };
            let mut observer = Observer { drift, artifact: artifact.clone(), calls: 0 };
            let result = admit_external_artifact_with_probe(&request, &output, &mut observer);
            assert_eq!(observer.calls, 1);
            if matches!(drift, Drift::Unchanged) {
                let admitted = result?;
                assert_eq!(admitted, load_external_artifact_admission(&output)?);
                assert_eq!(admitted, verify_external_artifact_admission(root, &active, &output)?);
            } else {
                let error = result.err().context("changed artifact must not be admitted")?;
                assert!(error.to_string().contains("changed during version observation"));
                assert!(!output.exists(), "published stale evidence for {kind:?}/{drift:?}");
            }
        }
    }
    Ok(())
}
