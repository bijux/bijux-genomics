//! Deterministic watchdog policy simulations over admitted pipeline DAG evidence.
mod admission;
mod completion_rules;
mod declarations;
mod failure_isolation;
mod model;
mod no_global_wait;
mod partial_resume;
mod paths;
mod scenario;
mod service;
mod status;
pub use declarations::{
    DEFAULT_COMPLETION_RULES_REPORT_PATH, DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
    DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH, DEFAULT_PARTIAL_RESUME_REPORT_PATH,
};
pub use model::{
    LocalDagWatchdogCompletionCheck, LocalDagWatchdogSimulationNode,
    LocalDagWatchdogSimulationReport,
};
pub use scenario::LocalDagWatchdogScenario;
pub use service::simulate_dag_watchdog_path;
