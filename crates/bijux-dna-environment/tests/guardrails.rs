use bijux_dna_policies::GuardrailConfig;

#[test]
fn guardrails() {
    let crate_root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let config = GuardrailConfig::for_crate("bijux-dna-environment");

    bijux_dna_policies::check(crate_root, &config)
        .unwrap_or_else(|err| panic!("guardrail policy failed: {err}"));
}
