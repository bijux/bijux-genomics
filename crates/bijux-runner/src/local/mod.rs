//! Owner: bijux-runner
//! Local runner placeholder (no docker).

pub mod executor;
pub mod replay;
pub mod support;

#[must_use]
pub fn module_id() -> &'static str {
    "bijux-runner-local"
}
