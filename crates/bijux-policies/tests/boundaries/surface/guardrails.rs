#![allow(non_snake_case)]
use std::path::Path;

use bijux_policies::{check, GuardrailConfig};

#[test]
fn policy__boundaries__guardrails__guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config)
        .unwrap_or_else(|err| bijux_policies::policy_panic!("guardrails failed: {err}"));
}
