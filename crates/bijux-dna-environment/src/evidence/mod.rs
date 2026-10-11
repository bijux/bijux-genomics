//! Owner: bijux-dna-environment
//! Typed admission for immutable container environment evidence.

mod chain;
mod image_storage;
mod inventory;
mod load;
mod model;
mod qualification_registry;
mod runtime_verification;
mod schema;
mod selection;
pub mod tool_contract;
mod validation;

pub use chain::{
    load_container_evidence_chain, validate_container_evidence_chain,
    validate_container_inventory_catalog, validate_container_recipe_evidence_chain,
    ContainerEvidenceChain,
};
pub use image_storage::{
    apply_image_storage_plan, create_image_storage_plan, derive_image_storage_report,
    image_storage_plan_schema, image_storage_report_schema, load_image_storage_plan,
    publish_image_storage_report, render_image_storage_report_markdown,
    rollback_image_storage_plan, verify_image_storage_plan, write_image_storage_plan_schema,
    write_image_storage_report_schema, ImageStorageMove, ImageStorageMoveCategory,
    ImageStoragePlan, ImageStoragePlanRequest, ImageStoragePlanState, ImageStorageRecovery,
    ImageStorageReport, ImageStorageReportSource, ImageStorageReportSources, ImageStorageVerdict,
    QuarantinedImageIdentity, IMAGE_STORAGE_PLAN_SCHEMA_V1, IMAGE_STORAGE_REPORT_SCHEMA_V1,
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
    CONTAINER_IMAGE_INVENTORY_PUBLICATION_SCHEMA_V2,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V1,
    CONTAINER_RUNTIME_VERIFICATION_PUBLICATION_SCHEMA_V2,
};
pub use qualification_registry::{
    derive_qualification_run_registry, load_qualification_run_registry,
    qualification_run_id_audit_schema, qualification_run_registry_schema,
    render_qualification_run_registry, validate_qualification_run_registry,
    write_qualification_run_id_audit_schema, write_qualification_run_registry_schema,
    DerivedQualificationRunRegistry, QualificationHistoricalRunIdentity,
    QualificationRegistryDerivationRequest, QualificationRunIdAudit, QualificationRunIdAuditRow,
    QualificationRunRegistry, QualificationRunRegistryEntry, QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V1,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V1,
};
pub use runtime_verification::{
    execute_runtime_verification, render_runtime_verification_markdown,
    verify_runtime_verification_report, RuntimeVerificationRequest, RuntimeVerificationResult,
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
