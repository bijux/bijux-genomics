//! Execution engine for Bijux.
//!
//! Owns: execution services, validation gates, and observability hooks.
//! Must NOT depend on: bijux-domain-* crates or domain semantics.

#![allow(
    clippy::module_name_repetitions,
    clippy::missing_errors_doc,
    clippy::implicit_hasher,
    clippy::must_use_candidate,
    clippy::new_without_default
)]

mod errors;
mod executor;
mod services;

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use anyhow::Result;
use bijux_core::contract::{ExecutionGraph, RetryPolicy, RunRecordV1};
use bijux_core::ids::StepId;
use bijux_runtime::run_layout::RunLayout;
use bijux_runtime::Runner;

#[derive(Debug, Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    #[must_use]
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::SeqCst);
    }

    #[must_use]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::SeqCst)
    }
}

impl Default for CancellationToken {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum EngineEvent {
    StepStart {
        step_id: StepId,
        attempt: u32,
    },
    StepEnd {
        step_id: StepId,
        attempt: u32,
        success: bool,
    },
    Retry {
        step_id: StepId,
        attempt: u32,
        exit_code: i32,
    },
    ArtifactVerified {
        step_id: StepId,
        path: String,
    },
}

pub trait EngineHooks: Send + Sync {
    fn on_event(&self, event: EngineEvent);
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct EngineConfig {
    pub step_timeout_s: Option<u64>,
    pub deterministic_scheduler: bool,
    pub retry_policy: Option<RetryPolicy>,
    pub max_parallelism: Option<usize>,
}

pub struct Engine {
    config: EngineConfig,
}

impl Engine {
    #[must_use]
    pub fn new(config: EngineConfig) -> Self {
        Self { config }
    }
}

impl Engine {
    /// # Errors
    /// Returns an error if validation or execution fails.
    pub fn execute(
        &self,
        graph: &ExecutionGraph,
        runner: &dyn Runner,
        _layout: &RunLayout,
        hooks: Option<&dyn EngineHooks>,
        cancel: Option<&CancellationToken>,
    ) -> Result<RunRecordV1> {
        let mut graph = graph.clone();
        if let Some(timeout) = self.config.step_timeout_s {
            graph = graph.with_step_timeout(Some(timeout));
        }
        if self.config.deterministic_scheduler {
            graph = graph.with_deterministic_scheduler(true);
        }
        if let Some(policy) = self.config.retry_policy.clone() {
            graph = graph.with_retry_policy(policy);
        }
        executor::execute_plan(&graph, runner, hooks, cancel)
    }
}

impl Default for Engine {
    fn default() -> Self {
        Self::new(EngineConfig::default())
    }
}

pub use errors::EngineError;
