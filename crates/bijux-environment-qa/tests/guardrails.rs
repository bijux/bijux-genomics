#[path = "guardrails/guardrails.rs"]
mod guardrails;
#[path = "guardrails/offline_policy.rs"]
mod offline_policy;

use std::path::Path;

use bijux_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
