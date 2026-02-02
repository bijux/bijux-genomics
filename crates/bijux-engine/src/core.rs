//! Owner: bijux-engine
//! Core planning, validation, and execution types.

pub mod composer;
pub mod errors;
pub mod types;
pub mod validator;

#[allow(dead_code)]
pub(crate) fn module_id() -> &'static str {
    "core"
}
