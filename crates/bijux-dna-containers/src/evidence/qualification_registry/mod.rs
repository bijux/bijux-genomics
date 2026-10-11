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
mod admission;
mod allocation;
mod audit;
mod history;
mod identity;
mod legacy;
mod protocol;
mod render;
mod storage;
pub use admission::{validate_container_run_index, validate_qualification_run_registry};
pub use model::{DerivedQualificationRunRegistry, QualificationRegistryDerivationRequest};
pub use render::{
    render_container_run_index_json, render_container_run_index_markdown,
    render_qualification_run_registry,
};
pub use storage::load_qualification_run_registry;
pub use workflow::{derive_container_run_index, derive_qualification_run_registry};
