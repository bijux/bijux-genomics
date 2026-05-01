#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunnerContractKind {
    Local,
    Docker,
    Apptainer,
}

impl std::fmt::Display for RunnerContractKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Local => f.write_str("local"),
            Self::Docker => f.write_str("docker"),
            Self::Apptainer => f.write_str("apptainer"),
        }
    }
}
