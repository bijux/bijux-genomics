use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeKind {
    Local,
    Docker,
    Singularity,
    Apptainer,
}

impl fmt::Display for RuntimeKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            RuntimeKind::Local => "local",
            RuntimeKind::Docker => "docker",
            RuntimeKind::Singularity => "singularity",
            RuntimeKind::Apptainer => "apptainer",
        };
        write!(f, "{value}")
    }
}
