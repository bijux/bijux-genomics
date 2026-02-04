//! Owner: bijux-engine
//! Execution services and IO boundaries.

pub mod composer;
pub mod pipeline;
pub mod run_artifacts;
pub mod telemetry;

#[allow(dead_code)]
pub(crate) fn module_id() -> &'static str {
    "services"
}
