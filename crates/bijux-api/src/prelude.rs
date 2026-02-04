//! Convenience re-exports for v1 API consumers.
//!
//! Keep this prelude intentionally small and stable.

pub use crate::v1::bench::{compare_runs, objective_spec, Objective, ObjectiveSpec, RankInput};
pub use crate::v1::plan::{
    plan_run, select_pipeline, select_pipelines, Domain, PipelineProfile, PipelineRegistry,
    PlanRunRequest, PlanRunResult,
};
pub use crate::v1::report::{render_report, RenderReportRequest, RenderReportResult};
pub use crate::v1::run::{execute_run, RunMode, RunRequest, RunResult};
