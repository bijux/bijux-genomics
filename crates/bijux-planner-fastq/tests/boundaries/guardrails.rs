#[path = "../../bijux-policies/tests/guardrails.rs"]
mod policies;

/// Centralized guardrails runner.
#[test]
fn guardrails() {
    policies::guardrails();
}
