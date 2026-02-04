//! Owner: bijux-exec
//! Stage execution orchestration (domain-neutral shell around runners).

#![allow(clippy::missing_errors_doc)]

pub mod observer;
mod plugins;
pub mod stage_exec;

pub mod primitives {
    pub use crate::observer::{hash_file_sha256, write_explain_plan, Observer};
    pub use crate::stage_exec::{execute_stage_plan, StageResultV1};
}
