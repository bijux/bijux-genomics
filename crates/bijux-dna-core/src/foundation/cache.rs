use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(deny_unknown_fields)]
pub struct CacheKey {
    pub input_fingerprint: String,
    pub parameters_fingerprint: String,
    pub tool_version: String,
    pub env_digest: String,
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
        }
    }

    #[must_use]
    pub fn as_string(&self) -> String {
        format!(
            "{}|{}|{}|{}",
            self.input_fingerprint, self.parameters_fingerprint, self.tool_version, self.env_digest
        )
    }
}
