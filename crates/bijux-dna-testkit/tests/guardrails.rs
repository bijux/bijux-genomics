#[test]
fn guardrails() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = bijux_dna_policies::GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    bijux_dna_policies::check(crate_root, &config)
        .unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
