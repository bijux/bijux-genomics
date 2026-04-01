use super::configuration::GuardrailConfig;

pub(super) fn guardrail_config() -> GuardrailConfig {
    GuardrailConfig {
        max_pub_items_per_file: 50,
        max_pub_use_per_file: 25,
        forbid_pub_use_spam: false,
        forbid_panic_expect: false,
        forbid_stage_id_strings: false,
        allow_panic_expect_paths: Vec::new(),
        allow_stage_id_paths: Vec::new(),
        allow_mod_only_dirs: Vec::new(),
    }
}
