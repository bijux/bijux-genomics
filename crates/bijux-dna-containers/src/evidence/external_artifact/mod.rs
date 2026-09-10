mod admission;
mod binding;
mod model;
mod probe;
mod request;
mod schema;
mod storage;
mod workflow;

pub use model::{
    external_artifact_image_ref, external_artifact_tool_id, ExternalArtifactAdmission,
    ExternalArtifactAdmissionStatus, ExternalArtifactCommandBinding, ExternalArtifactIdentity,
    ExternalArtifactKind, ExternalArtifactVersionProbe, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1,
    EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V2, EXTERNAL_ARTIFACT_IMAGE_SCHEME,
};
pub use schema::{
    external_artifact_admission_schema, external_artifact_admission_schema_matches,
    write_external_artifact_admission_schema,
};
pub use workflow::{
    admit_external_artifact, admit_external_artifact_with_probe, bind_external_artifact_command,
    load_external_artifact_admission, verify_external_artifact_admission,
};

pub use probe::{
    ExternalArtifactProbeObservation, ExternalArtifactProbeOutcome, ExternalArtifactProbeRequest,
    ExternalArtifactProbeTransport, NativeExternalArtifactProbeTransport,
};
pub use request::{ExternalArtifactAdmissionRequest, ExternalArtifactInput};
