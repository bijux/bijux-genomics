//! Owner: bijux-dna-containers
//! Stable container qualification-run identity registry.

mod model;
mod schema;
mod workflow;

pub use model::{
    QualificationHistoricalRunIdentity, QualificationRunIdAudit, QualificationRunIdAuditRow,
    QualificationRunRegistry, QualificationRunRegistryEntry, QUALIFICATION_RUN_ID_AUDIT_SCHEMA_V1,
    QUALIFICATION_RUN_REGISTRY_SCHEMA_V1,
};
pub use schema::{
    qualification_run_id_audit_schema, qualification_run_registry_schema,
    write_qualification_run_id_audit_schema, write_qualification_run_registry_schema,
};
pub use workflow::{
    derive_qualification_run_registry, load_qualification_run_registry,
    render_qualification_run_registry, validate_qualification_run_registry,
    DerivedQualificationRunRegistry, QualificationRegistryDerivationRequest,
};
