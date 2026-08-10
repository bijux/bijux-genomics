//! Owner: bijux-dna-containers
//! Stable container qualification-run identity registry.

mod model;
mod schema;
mod workflow;

pub use model::{
    ContainerRunIndex, ContainerRunIndexSource, QualificationHistoricalRunIdentity,
    QualificationRunCitationRecord, QualificationRunDefinition, QualificationRunIdAudit,
    QualificationRunIdAuditRow, QualificationRunIdTombstone, QualificationRunNamespacePolicy,
    QualificationRunRegistry, QualificationRunRegistryEntry, QualificationRunRegistrySummary,
    QualificationRunState, CONTAINER_RUN_INDEX_SCHEMA_V1, QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V3,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V3,
};
pub use schema::{
    container_run_index_schema, qualification_run_id_audit_schema,
    qualification_run_registry_schema, write_container_run_index_schema,
    write_qualification_run_id_audit_schema, write_qualification_run_registry_schema,
};
pub use workflow::{
    derive_container_run_index, derive_qualification_run_registry, load_qualification_run_registry,
    render_container_run_index_json, render_container_run_index_markdown,
    render_qualification_run_registry, validate_container_run_index,
    validate_qualification_run_registry, DerivedQualificationRunRegistry,
    QualificationRegistryDerivationRequest,
};
