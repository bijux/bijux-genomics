//! Owner: bijux-dna-containers
//! Typed admission for immutable container environment evidence.

mod chain;
mod external_artifact;
mod image_activation;
mod image_storage;
mod inventory;
mod load;
mod model;
mod qualification_registry;
mod run_evidence;
mod runtime_verification;
mod schema;
mod security_database;
mod security_probe;
mod security_toolchain;
mod selection;
pub mod tool_contract;
mod validation;

pub use chain::{
    load_container_evidence_chain, resolve_current_apptainer_contract_commit,
    validate_container_evidence_chain, validate_container_inventory_catalog,
    validate_container_recipe_evidence_chain, ContainerEvidenceChain,
};
pub use external_artifact::{
    admit_external_artifact, bind_external_artifact_command, external_artifact_admission_schema,
    external_artifact_image_ref, external_artifact_tool_id, load_external_artifact_admission,
    verify_external_artifact_admission, write_external_artifact_admission_schema,
    ExternalArtifactAdmission, ExternalArtifactAdmissionRequest, ExternalArtifactAdmissionStatus,
    ExternalArtifactCommandBinding, ExternalArtifactIdentity, ExternalArtifactInput,
    ExternalArtifactKind, ExternalArtifactVersionProbe, EXTERNAL_ARTIFACT_ADMISSION_SCHEMA_V1,
    EXTERNAL_ARTIFACT_IMAGE_SCHEME,
};
pub use image_activation::{
    apply_image_activation_plan, create_image_activation_plan, image_activation_plan_schema,
    load_image_activation_plan, rollback_image_activation_plan, verify_image_activation_plan,
    write_image_activation_plan_schema, ImageActivationPlan, ImageActivationPlanRequest,
    ImageActivationPlanState, ImageActivationPublication, IMAGE_ACTIVATION_PLAN_SCHEMA_V1,
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
    derive_image_inventory_evidence, derive_image_inventory_publication,
    render_image_inventory_markdown, DerivedImageInventory, ImageInventoryDerivationRequest,
    ImageInventoryEvidenceRoots, ImageInventoryScan, ImageInventoryScannedImage,
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
    QualificationRunIdTombstone, QualificationRunRegistry, QualificationRunRegistryEntry,
    QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V2, QUALIFICATION_RUN_REGISTRY_SCHEMA_V1,
};
pub use run_evidence::{
    apply_final_run_evidence_plan, apply_result_layout, apply_superseded_result_archive,
    container_attempt_layout_schema, derive_result_layout_plan, derive_run_evidence_index,
    derive_superseded_result_archive_plan, ensure_container_attempt_layout, import_run_evidence,
    load_run_evidence_index, plan_final_run_evidence, render_run_evidence_markdown,
    result_layout_plan_schema, rollback_result_layout, rollback_superseded_result_archive,
    run_evidence_index_schema, superseded_result_archive_plan_schema, validate_run_attempt_ledger,
    validate_run_evidence_index, validate_run_evidence_root, verify_result_layout,
    verify_run_evidence_publication, verify_superseded_result_archive,
    write_result_movement_schemas, write_run_evidence_index_schema, ArchivedResultCategory,
    AttemptExecutionStatus, ContainerAttemptIdentity, FinalRunEvidencePlan, ResultFileIdentity,
    ResultLayoutMove, ResultLayoutPlan, ResultLayoutPlanRequest, ResultMovementState,
    RunAttemptLedger, RunAttemptLedgerRow, RunAttemptLedgerSummary, RunEvidenceFileIdentity,
    RunEvidenceImportRequest, RunEvidenceIndex, RunEvidenceLock, RunEvidenceRecord,
    RunEvidenceSummary, ScientificAcceptanceStatus, SupersededResultArchivePlan,
    SupersededResultArchivePlanRequest, SupersededResultMove,
    CONTAINER_RESULT_LAYOUT_PLAN_SCHEMA_V1, CONTAINER_RUN_ATTEMPT_LEDGER_SCHEMA_V1,
    CONTAINER_RUN_EVIDENCE_INDEX_SCHEMA_V2, CONTAINER_RUN_EVIDENCE_LOCK_SCHEMA_V1,
    CONTAINER_SUPERSEDED_RESULT_ARCHIVE_SCHEMA_V1,
};
pub use runtime_verification::{
    render_runtime_verification_markdown, verify_runtime_verification_report,
};
pub use schema::{
    image_inventory_publication_schema, image_inventory_scan_schema,
    runtime_verification_publication_schema, write_image_inventory_publication_schema,
    write_image_inventory_scan_schema, write_runtime_verification_publication_schema,
};
pub use security_database::{
    load_grype_database_contract, verify_grype_database, AdmittedGrypeDatabase,
    GrypeDatabaseCompletion, GrypeDatabaseContract, GrypeDatabaseStatus,
    GRYPE_DATABASE_COMPLETION_SCHEMA_V1, GRYPE_DATABASE_CONTRACT_SCHEMA_V1,
};
pub use security_probe::{
    admit_security_probe_publication, security_probe_publication_schema,
    write_security_probe_publication_schema, AdmittedSecurityProbePublication,
    AdmittedSecurityProbeReports, SecurityFilesystemKind, SecurityFilesystemObservation,
    SecurityProbeArtifact, SecurityProbeAuthority, SecurityProbePublication, SecurityProbeStatus,
    SecurityRuntimeProbe, SecurityRuntimeReport, SecurityStructuralReport, ToolSecurityProbe,
    SECURITY_PROBE_PUBLICATION_SCHEMA_V1,
};
pub use security_toolchain::{
    load_security_scanner_toolchain_contract, verify_security_scanner_toolchain,
    SecurityScannerBinary, SecurityScannerTool, SecurityScannerToolchainCompletion,
    SecurityScannerToolchainContract, SECURITY_SCANNER_TOOLCHAIN_COMPLETION_SCHEMA_V1,
    SECURITY_SCANNER_TOOLCHAIN_CONTRACT_SCHEMA_V1,
};
pub use selection::{resolve_catalog_locked_apptainer_image, CurrentApptainerImage};
pub use tool_contract::{
    apptainer_recipe_identity_at_commit, container_build_evidence_matches_recipe,
    container_catalog_lock_schema, container_catalog_schema, container_version_lock_schema,
    container_version_map_schema, derive_container_catalog_lock,
    derive_container_registry_projection_report, derive_container_version_lock,
    derive_container_version_map, load_container_catalog, load_container_catalog_lock,
    load_container_version_lock, render_container_catalog_lock, render_container_version_index,
    render_container_version_lock, render_container_version_map, repository_head_commit,
    validate_container_catalog, validate_container_catalog_lock, validate_container_version_lock,
    validate_container_version_lock_model, write_container_catalog_lock_schema,
    write_container_catalog_schema, write_container_version_lock_schema,
    write_container_version_map_schema, ApptainerRecipeIdentity, ContainerCatalog,
    ContainerCatalogLock, ContainerCatalogLockTool, ContainerCatalogTool,
    ContainerCatalogValidation, ContainerRegistryProjectionReport,
    ContainerRegistryProjectionSource, ContainerVersionLock, ContainerVersionLockItem,
    ContainerVersionMap, ContainerVersionMapItem, CONTAINER_CATALOG_LOCK_PATH,
    CONTAINER_CATALOG_LOCK_SCHEMA_V1, CONTAINER_CATALOG_PATH, CONTAINER_CATALOG_SCHEMA_V1,
    CONTAINER_REGISTRY_PROJECTION_SCHEMA_V1, CONTAINER_VERSION_CATALOG_PATH,
    CONTAINER_VERSION_INDEX_PATH, CONTAINER_VERSION_LOCK_COMMAND,
    CONTAINER_VERSION_LOCK_GENERATOR_PATH, CONTAINER_VERSION_LOCK_PATH,
    CONTAINER_VERSION_LOCK_SCHEMA_V4, CONTAINER_VERSION_MAP_SCHEMA_V2,
};
pub use validation::{
    validate_image_inventory_evidence, validate_image_inventory_publication,
    validate_image_inventory_scan, validate_runtime_verification_publication,
};
