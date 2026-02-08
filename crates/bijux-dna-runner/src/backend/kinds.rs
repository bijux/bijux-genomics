#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BackendKind {
    Docker,
}

impl BackendKind {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            BackendKind::Docker => "docker",
        }
    }
}
