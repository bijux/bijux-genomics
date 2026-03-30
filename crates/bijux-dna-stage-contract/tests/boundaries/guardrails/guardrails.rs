use bijux_dna_policies::{check, GuardrailConfig};

#[test]
fn guardrails() {
    let crate_root = crate::support::crate_root("bijux-dna-stage-contract")
        .unwrap_or_else(|err| panic!("resolve crate root: {err}"));
    let config = GuardrailConfig::for_crate(env!("CARGO_PKG_NAME"));
    check(&crate_root, &config).unwrap_or_else(|err| panic!("guardrails failed: {err}"));
}
