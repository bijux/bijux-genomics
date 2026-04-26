#[path = "../support/workspace_paths.rs"]
mod support;

use bijux_dna_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = support::crate_root("bijux-dna-runtime")
        .unwrap_or_else(|err| panic!("resolve runtime crate root: {err}"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(&crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
