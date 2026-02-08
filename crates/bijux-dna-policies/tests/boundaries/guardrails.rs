#![allow(non_snake_case)]

#[path = "../guardrails.rs"]
mod policies;

/// Centralized guardrails runner.
#[test]
fn policy__boundaries__guardrails__guardrails() {
    policies::guardrails();
}
