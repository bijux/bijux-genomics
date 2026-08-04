//! Owner: bijux-dna-environment
//! Typed admission for immutable container environment evidence.

mod chain;
mod load;
mod model;
mod schema;
pub mod tool_contract;
mod validation;

pub use chain::{
    load_container_evidence_chain, validate_container_evidence_chain, ContainerEvidenceChain,
};
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
pub use tool_contract::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, validate_container_catalog, ApptainerRecipeIdentity, ContainerCatalog,
    ContainerCatalogTool, ContainerCatalogValidation,
};
pub use validation::{
    validate_image_inventory_publication, validate_runtime_verification_publication,
};
