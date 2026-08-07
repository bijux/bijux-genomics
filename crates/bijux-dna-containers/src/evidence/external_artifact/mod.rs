mod model;
mod schema;
mod workflow;

pub use model::{
    ExternalArtifactAdmission, ExternalArtifactAdmissionStatus, ExternalArtifactCommandBinding,
    ExternalArtifactIdentity, ExternalArtifactKind, ExternalArtifactVersionProbe,
    EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1,
};
pub use schema::{external_artifact_admission_schema, write_external_artifact_admission_schema};
pub use workflow::{
    admit_external_artifact, bind_external_artifact_command, load_external_artifact_admission,
    verify_external_artifact_admission, ExternalArtifactAdmissionRequest, ExternalArtifactInput,
};
