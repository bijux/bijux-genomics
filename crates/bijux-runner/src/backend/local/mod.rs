//! Owner: bijux-runner
//! Local runner placeholder (no docker).

pub mod executor;
pub mod execution_spec;
pub mod replay;

#[must_use]
pub fn module_id() -> &'static str {
    "bijux-runner-local"
}
