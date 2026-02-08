#[path = "guardrails/canonical_owner.rs"]
mod canonical_owner;
#[path = "guardrails/no_generic_helpers.rs"]
mod no_generic_helpers;
#[path = "guardrails/policies.rs"]
mod policies;
#[path = "guardrails/public_surface.rs"]
mod public_surface;

use std::path::Path;

use bijux_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
