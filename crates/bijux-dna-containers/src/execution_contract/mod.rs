//! Typed declarations admitted before container execution.

mod credential;

pub use credential::{
    container_credential_reference_schema, load_container_credential_reference,
    validate_container_credential_reference, write_container_credential_reference_schema,
    ContainerCredentialRecord, ContainerCredentialRedaction, ContainerCredentialReference,
    CONTAINER_CREDENTIAL_REFERENCE_SCHEMA_V1,
};
