use serde::{Deserialize, Serialize};

use super::{baseline, presets};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuardrailConfig {
    pub max_pub_items_per_file: usize,
    pub max_pub_use_per_file: usize,
    pub forbid_pub_use_spam: bool,
    pub forbid_panic_expect: bool,
    pub forbid_stage_id_strings: bool,
    pub allow_panic_expect_paths: Vec<String>,
    pub allow_stage_id_paths: Vec<String>,
    pub allow_mod_only_dirs: Vec<String>,
}

impl Default for GuardrailConfig {
    fn default() -> Self {
        baseline::guardrail_config()
    }
}

impl GuardrailConfig {
    #[must_use]
    pub fn for_crate(name: &str) -> Self {
        presets::for_crate(name)
    }
}
