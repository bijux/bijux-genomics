#![allow(non_snake_case)]

use bijux_policies::GuardrailConfig;

pub fn guardrails() {
    let _config = GuardrailConfig::for_crate("bijux-policies");
}

#[test]
fn policy__root__guardrails__guardrails_smoke() {
    guardrails();
}
