use std::collections::BTreeMap;
use std::fmt::Write as _;

use sha2::Digest;

use crate::{PipelineProfile, ProfileManifestV1};

impl PipelineProfile {
    #[must_use]
    pub fn profile_manifest(&self) -> ProfileManifestV1 {
        let mut stage_list: Vec<String> =
            self.defaults.tools.keys().map(|stage| stage.as_str().to_string()).collect();
        stage_list.sort();
        stage_list.dedup();
        for stage in self.defaults.params.keys() {
            let stage_id = stage.as_str().to_string();
            if !stage_list.contains(&stage_id) {
                stage_list.push(stage_id);
            }
        }
        stage_list.sort();
        let tool_ids = self
            .defaults
            .tools
            .iter()
            .map(|(stage, tool)| (stage.as_str().to_string(), tool.as_str().to_string()))
            .collect();
        let param_hashes =
            self.defaults
                .params
                .iter()
                .map(|(stage, params)| {
                    let canonical = bijux_dna_core::contract::canonical::to_canonical_json_bytes(
                        &params.to_json(),
                    )
                    .unwrap_or_else(|err| {
                        panic!("failed to canonicalize params for stage {}: {err}", stage.as_str())
                    });
                    let mut hasher = sha2::Sha256::new();
                    hasher.update(canonical);
                    (stage.as_str().to_string(), sha256_hex(hasher.finalize()))
                })
                .collect();
        let schema_versions = BTreeMap::from([
            ("profile_manifest".to_string(), "bijux.profile_manifest.v1".to_string()),
            ("defaults_ledger".to_string(), "bijux.defaults_ledger.v1".to_string()),
            ("params".to_string(), "by_stage".to_string()),
        ]);
        ProfileManifestV1 {
            schema_version: "bijux.profile_manifest.v1",
            pipeline_id: self.id.as_str().to_string(),
            invariants_preset: self.invariants_preset.map(|preset| preset.as_str().to_string()),
            library_model: self.library_model,
            stage_list,
            tool_ids,
            param_hashes,
            schema_versions,
        }
    }

    #[must_use]
    pub fn profile_hash(&self) -> String {
        let manifest = self.profile_manifest();
        let bytes = bijux_dna_core::contract::canonical::to_canonical_json_bytes(&manifest)
            .unwrap_or_else(|err| panic!("failed to canonicalize profile manifest: {err}"));
        let mut hasher = sha2::Sha256::new();
        hasher.update(bytes);
        sha256_hex(hasher.finalize())
    }
}

fn sha256_hex(digest: impl AsRef<[u8]>) -> String {
    let bytes = digest.as_ref();
    let mut hex = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        let _ = write!(&mut hex, "{byte:02x}");
    }
    hex
}
