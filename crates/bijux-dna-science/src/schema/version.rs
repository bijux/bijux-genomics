//! Authored science document version access shared by authored source consumers.
pub trait SchemaVersion {
    fn schema_version(&self) -> &str;
}

impl SchemaVersion for crate::domain::SourceSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::EvidenceSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ClaimSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::AssumptionSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ReasoningSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::DecisionSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::BindingSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}

impl SchemaVersion for crate::domain::ReleaseManifestSpec {
    fn schema_version(&self) -> &str {
        &self.schema_version
    }
}
