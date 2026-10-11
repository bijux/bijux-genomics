//! Borrowed operator inputs for external artifact admission.
use super::model::{ExternalArtifactAdmissionStatus, ExternalArtifactKind};
use std::path::Path;

pub struct ExternalArtifactInput<'a> {
    pub kind: ExternalArtifactKind,
    pub path: &'a Path,
    pub expected_sha256: &'a str,
    pub lawful_access_basis: &'a str,
    pub apptainer_executable: Option<&'a Path>,
}

pub struct ExternalArtifactAdmissionRequest<'a> {
    pub repository_root: &'a Path,
    pub distributable_image_root: &'a Path,
    pub repository_commit: &'a str,
    pub tool_id: &'a str,
    pub recorded_at_utc: &'a str,
    pub availability: ExternalArtifactAdmissionStatus,
    pub artifact: Option<ExternalArtifactInput<'a>>,
    pub blocker_reason: Option<&'a str>,
}
