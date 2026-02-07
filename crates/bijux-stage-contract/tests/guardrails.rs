// Owner: bijux-stage-contract
// Guardrails suite spine.
#[path = "guardrails/guardrails.rs"]
mod guardrails;
#[path = "guardrails/no_execution_scan.rs"]
mod no_execution_scan;
#[path = "guardrails/tree_contract.rs"]
mod tree_contract;

use std::path::Path;

use bijux_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
