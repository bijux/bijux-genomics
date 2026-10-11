use schemars::{schema_for, Schema};

use super::model::ContainerSecurityEvidencePublication;

#[must_use]
pub fn security_evidence_schema() -> Schema {
    schema_for!(ContainerSecurityEvidencePublication)
}
