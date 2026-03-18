use std::fmt::Display;

pub const WHY: &str =
    "Policies protect architectural boundaries, ownership, and determinism across the workspace.";
pub const HOW: &str =
    "Fix the offending code or docs, then re-run the policy suite to confirm the change.";
pub const MORE: &str = "crates/bijux-dna-policies/docs/POLICY_DIAGNOSTICS.md";

pub fn message(what: impl Display) -> String {
    format!("WHAT: {what}\nWHY: {WHY}\nHOW: {HOW}\nMORE: {MORE}")
}
