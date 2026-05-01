use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct CacheKey {
    pub input_fingerprint: String,
    pub parameters_fingerprint: String,
    pub tool_version: String,
    pub env_digest: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub stage_contract_version: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub backend_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub input_artifact_identities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reference_identities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_versions: Vec<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub environment_compatibility: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReproducibilityIdentityV1 {
    pub image_digest: String,
    pub tool_version: String,
    pub params_hash: String,
    pub input_hash: String,
    pub bank_hashes: serde_json::Value,
}

impl ReproducibilityIdentityV1 {
    #[must_use]
    pub fn as_string(&self) -> String {
        let bank_hashes =
            serde_json::to_string(&self.bank_hashes).unwrap_or_else(|_| "{}".to_string());
        format!(
            "{}|{}|{}|{}|{}",
            self.image_digest, self.tool_version, self.params_hash, self.input_hash, bank_hashes
        )
    }
}

impl CacheKey {
    #[must_use]
    pub fn new(
        input_fingerprint: impl Into<String>,
        parameters_fingerprint: impl Into<String>,
        tool_version: impl Into<String>,
        env_digest: impl Into<String>,
    ) -> Self {
        Self {
            input_fingerprint: input_fingerprint.into(),
            parameters_fingerprint: parameters_fingerprint.into(),
            tool_version: tool_version.into(),
            env_digest: env_digest.into(),
            stage_contract_version: String::new(),
            backend_identity: String::new(),
            input_artifact_identities: Vec::new(),
            reference_identities: Vec::new(),
            policy_versions: Vec::new(),
            environment_compatibility: String::new(),
        }
    }

    #[allow(clippy::too_many_arguments)]
    #[must_use]
    pub fn with_governance(
        input_fingerprint: impl Into<String>,
        parameters_fingerprint: impl Into<String>,
        tool_version: impl Into<String>,
        env_digest: impl Into<String>,
        stage_contract_version: impl Into<String>,
        backend_identity: impl Into<String>,
        input_artifact_identities: Vec<String>,
        reference_identities: Vec<String>,
        policy_versions: Vec<String>,
        environment_compatibility: impl Into<String>,
    ) -> Self {
        Self {
            input_fingerprint: input_fingerprint.into(),
            parameters_fingerprint: parameters_fingerprint.into(),
            tool_version: tool_version.into(),
            env_digest: env_digest.into(),
            stage_contract_version: stage_contract_version.into(),
            backend_identity: backend_identity.into(),
            input_artifact_identities,
            reference_identities,
            policy_versions,
            environment_compatibility: environment_compatibility.into(),
        }
    }

    #[must_use]
    pub fn as_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.input_fingerprint, self.parameters_fingerprint, self.tool_version, self.env_digest
        )
    }

    #[must_use]
    pub fn governed_identity_string(&self) -> String {
        let input_artifact_identities = self.input_artifact_identities.join(",");
        let reference_identities = self.reference_identities.join(",");
        let policy_versions = self.policy_versions.join(",");
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
            self.input_fingerprint,
            self.parameters_fingerprint,
            self.tool_version,
            self.env_digest,
            self.stage_contract_version,
            self.backend_identity,
            input_artifact_identities,
            reference_identities,
            policy_versions,
            self.environment_compatibility
        )
    }
}
