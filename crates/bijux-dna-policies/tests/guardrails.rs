#![allow(non_snake_case)]

use bijux_dna_policies::GuardrailConfig;

pub fn guardrails() {
    let _config = GuardrailConfig::for_crate("bijux-dna-policies");
}

#[test]
fn policy__root__guardrails__guardrails_smoke() {
    guardrails();
}
