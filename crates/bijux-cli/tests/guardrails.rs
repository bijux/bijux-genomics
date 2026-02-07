#[path = "guardrails/architecture.rs"]
mod architecture;
#[path = "guardrails/architecture_guardrail.rs"]
mod architecture_guardrail;
#[path = "guardrails/ci_contract.rs"]
mod ci_contract;
#[path = "guardrails/deps.rs"]
mod deps;
#[path = "guardrails/no_process_spawn.rs"]
mod no_process_spawn;
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
