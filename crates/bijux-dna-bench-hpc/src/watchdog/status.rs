#[derive(Debug, Clone, Copy)]
pub(super) enum LocalDagWatchdogNodeStatus {
    Completed,
    Failed,
    Blocked,
    Reused,
    Planned,
    Incomplete,
}

impl LocalDagWatchdogNodeStatus {
    pub(super) fn as_str(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
            Self::Reused => "reused",
            Self::Planned => "planned",
            Self::Incomplete => "incomplete",
        }
    }
}
