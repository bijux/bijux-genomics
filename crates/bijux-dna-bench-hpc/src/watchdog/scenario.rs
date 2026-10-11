use super::declarations::{
    DEFAULT_COMPLETION_RULES_REPORT_PATH, DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
    DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH, DEFAULT_PARTIAL_RESUME_REPORT_PATH,
};
#[derive(Debug, Clone, Copy)]
pub enum LocalDagWatchdogScenario {
    NoGlobalWait,
    FailureIsolation,
    PartialResume,
    CompletionRules,
}

impl LocalDagWatchdogScenario {
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoGlobalWait => "no_global_wait",
            Self::FailureIsolation => "failure_isolation",
            Self::PartialResume => "partial_resume",
            Self::CompletionRules => "completion_rules",
        }
    }

    #[must_use]
    pub fn default_output_relative_path(self) -> &'static str {
        match self {
            Self::NoGlobalWait => DEFAULT_NO_GLOBAL_WAIT_REPORT_PATH,
            Self::FailureIsolation => DEFAULT_FAILURE_ISOLATION_REPORT_PATH,
            Self::PartialResume => DEFAULT_PARTIAL_RESUME_REPORT_PATH,
            Self::CompletionRules => DEFAULT_COMPLETION_RULES_REPORT_PATH,
        }
    }
}
