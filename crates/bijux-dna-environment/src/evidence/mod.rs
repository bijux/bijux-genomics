//! Owner: bijux-dna-environment
//! Typed admission for immutable container environment evidence.

mod load;
mod model;
mod schema;
mod validation;

pub use load::{load_image_inventory_publication, load_runtime_verification_publication};
pub use model::{
    BuildContextIdentity, BuildEvidenceStatus, ContainerBuildEvidence, ContainerImageCandidate,
    ContainerImageInventoryPublication, ContainerImageInventoryTool, ContainerImageState,
    ContainerRuntimeEvidence, ContainerSupportDisposition, EvidenceSourceIdentity,
    ImageCandidateDisposition, ImageInventoryPublicationIdentity, RuntimeEvidenceStatus,
    RuntimeVerificationPublication, CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
};
pub use schema::{
    image_inventory_publication_schema, runtime_verification_publication_schema,
    write_image_inventory_publication_schema, write_runtime_verification_publication_schema,
};
pub use validation::{
    validate_image_inventory_publication, validate_runtime_verification_publication,
};
