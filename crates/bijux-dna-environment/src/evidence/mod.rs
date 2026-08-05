//! Owner: bijux-dna-environment
//! Typed admission for immutable container environment evidence.

mod chain;
mod inventory;
mod load;
mod model;
mod schema;
mod selection;
pub mod tool_contract;
mod validation;

pub use chain::{
    load_container_evidence_chain, validate_container_evidence_chain,
    validate_container_inventory_catalog, validate_container_recipe_evidence_chain,
    ContainerEvidenceChain,
};
pub use inventory::{
    derive_image_inventory_evidence, derive_image_inventory_publication, DerivedImageInventory,
    ImageInventoryDerivationRequest, ImageInventoryScan, ImageInventoryScannedImage,
    ImageInventorySourceFile,
};
pub use load::{
    load_image_inventory_publication, load_image_inventory_scan,
    load_runtime_verification_publication,
};
pub use model::{
    BuildContextIdentity, BuildEvidenceStatus, ContainerBuildEvidence, ContainerDetachedImage,
    ContainerImageCandidate, ContainerImageInventoryPublication, ContainerImageInventorySummary,
    ContainerImageInventoryTool, ContainerImageState, ContainerRuntimeEvidence,
    ContainerStagedImage, ContainerSupportDisposition, DetachedImageDisposition,
    EvidenceSourceIdentity, ImageCandidateDisposition, ImageInventoryPublicationIdentity,
    RuntimeEvidenceStatus, RuntimeVerificationPublication,
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
};
pub use schema::{
    image_inventory_publication_schema, image_inventory_scan_schema,
    runtime_verification_publication_schema, write_image_inventory_publication_schema,
    write_image_inventory_scan_schema, write_runtime_verification_publication_schema,
};
pub use selection::{resolve_current_apptainer_image, CurrentApptainerImage};
pub use tool_contract::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    load_container_catalog, repository_head_commit, validate_container_catalog,
    ApptainerRecipeIdentity, ContainerCatalog, ContainerCatalogTool, ContainerCatalogValidation,
};
pub use validation::{
    validate_image_inventory_evidence, validate_image_inventory_publication,
    validate_image_inventory_scan, validate_runtime_verification_publication,
};
