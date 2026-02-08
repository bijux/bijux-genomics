#[path = "../guardrails.rs"]
mod policies;

/// Centralized guardrails runner.
#[test]
fn guardrails() {
    policies::guardrails();
}
