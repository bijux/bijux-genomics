use bijux_guardrails::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
